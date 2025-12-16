// src/routes/llm/chat_analysis/update_chat_message.rs
use super::models::*;
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use actix_web::{post, web, HttpResponse};
use std::path::Path;
use serde_json::json; // <--- ADD THIS for JSON response if branching

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
    let updated_content = data.content.clone();
    let create_new_branch = data.create_new_branch;

    if create_new_branch {
        // Load the existing query data to get the original message and its parent
        let query_data = match project_service.query_manager.load_query_data(&project_dir, query_id) {
            Ok(qd) => qd,
            Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load query data: {}", e)),
        };

        let original_message = match query_data.chat_nodes.get(&message_id) {
            Some(msg) => msg,
            None => return HttpResponse::BadRequest().body("Original message not found for branching edit."),
        };

        // Create a new message with the updated content
        let new_message = crate::models::ChatMessage {
            role: original_message.role.clone(),
            content: updated_content.clone(),
            hidden: original_message.hidden,
            commit_hash: original_message.commit_hash.clone(),
            timestamp: Some(chrono::Utc::now()),
            context_files: original_message.context_files.clone(),
            provider: original_message.provider.clone(),
            model: original_message.model.clone(),
            hidden_context: original_message.hidden_context.clone(),
            ..Default::default() // id and parent_id will be set by add_chat_message
        };

        // Add the new message, making its parent the same as the original message's parent.
        // This effectively creates a new branch from the original message's parent.
        let new_message_id = match project_service.chat_manager.add_chat_message(
            &project_service.query_manager,
            &project_dir,
            new_message,
            query_id,
            original_message.parent_id, // New message branches from the same point as the original
        ) {
            Ok(id) => id,
            Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to save edited message as new branch: {}", e)),
        };

        // Return the new message ID and content for the frontend to update its display
        HttpResponse::Ok().json(json!({
            "message_id": new_message_id,
            "content": updated_content
        }))

    } else {
        // Existing logic for in-place update
        let result = project_service.chat_manager.update_message_in_history(
            &project_service.query_manager,
            &project_dir,
            message_id,
            updated_content.clone(),
            query_id,
        );

        match result {
            Ok(()) => HttpResponse::Ok().json(json!({
                "message_id": message_id, // Return original ID for in-place update
                "content": updated_content
            })),
            Err(message) => HttpResponse::BadRequest().body(message),
        }
    }
}