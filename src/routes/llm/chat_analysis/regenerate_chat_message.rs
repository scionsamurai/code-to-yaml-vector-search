// src/routes/llm/chat_analysis/regenerate_chat_message.rs
use super::models::*;
use super::utils::*;
use crate::models::AppState;
use crate::models::ChatMessage;
use crate::services::project_service::ProjectService;
use actix_web::{post, web, HttpResponse};
use crate::services::utils::html_utils::unescape_html;
use std::path::Path;


#[post("/regenerate-chat-message")]
pub async fn regenerate_chat_message(
    app_state: web::Data<AppState>,
    data: web::Json<RegenerateChatMessageRequest>,
) -> HttpResponse {
    let project_service = ProjectService::new();

    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project);

    let project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to load project: {}", e))
        }
    };

    let query_id = data.query_id.as_deref().unwrap_or_default();
    let message_id_to_regenerate = data.message_id; // Uuid of the model message to "regenerate from"

    // Load query data to get grounding_with_search setting
    let query_data = match project_service.query_manager.load_query_data(&project_dir, query_id) {
        Ok(qd) => qd,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load query data: {}", e)),
    };
    let enable_grounding = query_data.grounding_with_search; // Get grounding setting
    let agentic_mode_enabled = query_data.agentic_mode_enabled; // Get agentic_mode setting

    let include_file_descriptions = query_data.include_file_descriptions;

    // Get the *full* active branch history
    let full_history = get_full_history(&project, &app_state, &query_id);

    // Find the message to regenerate by ID
    let message_to_regenerate = full_history.iter().find(|msg| msg.id == message_id_to_regenerate);

    // Ensure it's a model message and get its preceding user message's ID for branching
    let user_message_id_for_parent = if let Some(model_msg) = message_to_regenerate {
        if model_msg.role != "model" {
            return HttpResponse::BadRequest().body("Can only regenerate model messages.");
        }
        model_msg.parent_id
    } else {
        return HttpResponse::BadRequest().body("Message to regenerate not found.");
    };

    if user_message_id_for_parent.is_none() {
        return HttpResponse::BadRequest().body("Could not find a preceding user message for regeneration. Cannot branch without a parent.");
    }
    let user_message_id = user_message_id_for_parent.unwrap();

    // Find the actual user message that is the parent of the message being regenerated.
    let user_message_index = full_history.iter().position(|msg| msg.id == user_message_id);
    if user_message_index.is_none() {
        return HttpResponse::BadRequest().body("Could not find the user message linked to the regeneration target.");
    }
    let user_message_index = user_message_index.unwrap();

    let actual_user_message_from_history = full_history[user_message_index].clone();
    // Get the raw content of the user message that the LLM needs to respond to.
    let user_message_content_raw = unescape_html(actual_user_message_from_history.content.clone());

    // --- CRITICAL CHANGE START: Prepare `previous_history_for_llm` ---
    // This history should include all messages *before* the current user message (user_msg_A in our example).
    // The `user_message_content_raw` will be treated as the *current* user input.
    let mut previous_history_for_llm: Vec<ChatMessage> = Vec::new();
    let mut hidden_context: Vec<String> = Vec::new();

    // Iterate up to (but *not including*) the user message that is the parent of the regenerated message.
    for message in full_history.iter().take(user_message_index) { // `take(user_message_index)` gets messages from 0 to index-1
        if message.role == "git-flag" {
            continue;
        }

        let code = match (message.role.as_str(), message.hidden) {
            ("user", false) => "P",
            ("user", true) => "p",
            ("model", false) => "R",
            ("model", true) => "r",
            _ => "", // Handle unexpected roles
        };
        if !code.is_empty() {
            hidden_context.push(code.to_string());
        }

        // Clone the message and only unescape its content, preserving other fields like id, parent_id, thoughts.
        let mut msg_clone = message.clone();
        msg_clone.content = unescape_html(message.content.clone());
        previous_history_for_llm.push(msg_clone);
    }
    // --- CRITICAL CHANGE END ---

    // Choose logic based on agentic_mode_enabled
    let new_assistant_message = match handle_chat_message(
        &project,
        &app_state,
        query_id,
        &user_message_content_raw, // The content of the user message to respond to
        enable_grounding,
        include_file_descriptions,
        &previous_history_for_llm, // The history *leading up to* that user message
        actual_user_message_from_history.commit_hash.clone(),
        hidden_context.clone(),
        agentic_mode_enabled,
    ).await {
        Ok(model_message) => model_message,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Chat message handling failed: {}", e));
        }
    };

    // Add the new message, explicitly setting its parent_id to the user message
    // This creates a new branch point from the user message.
    let new_assistant_message_id = match project_service.chat_manager.add_chat_message(
        &project_service.query_manager,
        &project_dir,
        new_assistant_message,
        query_id,
        Some(user_message_id) // The parent is the user message (user_msg_A)
    ) {
        Ok(id) => id,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to save regenerated message as new branch: {}", e)),
    };

    // Update the QueryData's current_node_id to point to the new message
    let mut query_data_for_node_update = match project_service.query_manager.load_query_data(&project_dir, query_id) {
        Ok(qd) => qd,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load query data for current_node_id update: {}", e)),
    };
    query_data_for_node_update.current_node_id = Some(new_assistant_message_id);
    if let Err(e) = project_service.query_manager.save_query_data(&project_dir, &query_data_for_node_update, query_id) {
        eprintln!("Failed to save query data after regenerating message: {}", e);
        return HttpResponse::InternalServerError().body(format!("Failed to update query's current node: {}", e));
    }

    // Fetch the newly created message to return its full data
    let new_model_message_to_return = project_service.query_manager.get_chat_node(&project_dir, query_id, &new_assistant_message_id)
        .ok_or_else(|| HttpResponse::InternalServerError().body("Failed to retrieve new regenerated model message.".to_string()));


    // Return JSON response instead of redirect
    HttpResponse::Ok().json(RegenerateChatMessageResponse {
        success: true,
        new_model_message: new_model_message_to_return.unwrap(),
        new_current_node_id: new_assistant_message_id,
        user_message_id, // The user message this model message is a child of
    })
}