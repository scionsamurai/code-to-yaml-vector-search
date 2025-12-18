// src/routes/llm/chat_analysis/update_chat_message.rs
use super::models::*;
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use actix_web::{post, web, HttpResponse};
// Removed: use actix_web::http::header;
use std::path::Path;

#[post("/update-chat-message")]
pub async fn update_chat_message(
    app_state: web::Data<AppState>,
    data: web::Json<UpdateChatMessageRequest>,
) -> HttpResponse {
    let project_service = ProjectService::new();

    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project);

    let query_id = data.query_id.as_deref().unwrap();
    let message_id = data.message_id;
    let updated_content = data.content.clone(); // Content is now raw markdown
    let create_new_branch = data.create_new_branch;

    if create_new_branch {
        // Load the existing query data to get the original message and its parent
        let query_data_for_branch_check = match project_service.query_manager.load_query_data(&project_dir, query_id) {
            Ok(qd) => qd,
            Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load query data: {}", e)),
        };

        let original_message = match query_data_for_branch_check.chat_nodes.get(&message_id) {
            Some(msg) => msg,
            None => return HttpResponse::BadRequest().body("Original message not found for branching edit."),
        };

        // Create a new message with the updated content (raw markdown)
        let new_message = crate::models::ChatMessage {
            role: original_message.role.clone(),
            content: updated_content.clone(), // Raw markdown
            hidden: original_message.hidden,
            commit_hash: original_message.commit_hash.clone(),
            timestamp: Some(chrono::Utc::now()),
            context_files: original_message.context_files.clone(),
            provider: original_message.provider.clone(),
            model: original_message.model.clone(),
            hidden_context: original_message.hidden_context.clone(),
            ..Default::default()
        };

        // Add the new message, making its parent the same as the original message's parent.
        // This effectively creates a new branch from the original message's parent.
        let new_edited_message_id = match project_service.chat_manager.add_chat_message(
            &project_service.query_manager,
            &project_dir,
            new_message,
            query_id,
            original_message.parent_id, // New message branches from the same point as the original
        ) {
            Ok(id) => id,
            Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to save edited message as new branch: {}", e)),
        };

        // --- NEW: Update the QueryData's current_node_id to point to the new edited message ---
        let mut query_data = match project_service.query_manager.load_query_data(&project_dir, query_id) {
            Ok(qd) => qd,
            Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load query data for current_node_id update: {}", e)),
        };
        query_data.current_node_id = Some(new_edited_message_id);
        if let Err(e) = project_service.query_manager.save_query_data(&project_dir, &query_data, query_id) {
            eprintln!("Failed to save query data after branching edit: {}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to update query's current node after edit branch: {}", e));
        }
        // --- END NEW ---

        // Fetch the newly created message to return its full data
        let new_message_to_return = project_service.query_manager.get_chat_node(&project_dir, query_id, &new_edited_message_id)
            .ok_or_else(|| HttpResponse::InternalServerError().body("Failed to retrieve new edited message.")).unwrap();

        HttpResponse::Ok().json(UpdateChatMessageResponse {
            success: true,
            message: new_message_to_return,
            new_current_node_id: new_edited_message_id,
            parent_message_id: original_message.parent_id, // Pass parent_id for potential UI updates
        })

    } else {
        // Existing logic for in-place update
        let result = project_service.chat_manager.update_message_in_history(
            &project_service.query_manager,
            &project_dir,
            message_id,
            updated_content.clone(), // Raw markdown
            query_id,
        );

        match result {
            Ok(()) => {
                // For in-place update, current_node_id does not change, as it's the same message node.
                // Fetch the updated message to return its full data
                let updated_message_to_return = project_service.query_manager.get_chat_node(&project_dir, query_id, &message_id)
                    .ok_or_else(|| HttpResponse::InternalServerError().body("Failed to retrieve updated message.")).unwrap();

                let query_data = match project_service.query_manager.load_query_data(&project_dir, query_id) {
                    Ok(qd) => qd,
                    Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load query data: {}", e)),
                };

                HttpResponse::Ok().json(UpdateChatMessageResponse {
                    success: true,
                    message: updated_message_to_return,
                    new_current_node_id: query_data.current_node_id.unwrap_or(message_id), // Current node ID remains the same
                    parent_message_id: None, // Not relevant for in-place updates usually
                })
            },
            Err(message) => HttpResponse::BadRequest().body(message),
        }
    }
}