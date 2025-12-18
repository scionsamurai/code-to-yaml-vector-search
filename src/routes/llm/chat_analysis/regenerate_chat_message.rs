// src/routes/llm/chat_analysis/regenerate_chat_message.rs
use super::models::*;
use super::utils::*;
use crate::models::AppState;
use crate::models::ChatMessage;
use crate::services::llm_service::LlmService;
use crate::services::project_service::ProjectService;
use actix_web::{post, web, HttpResponse};
// Removed: use actix_web::http::header;
use std::path::Path;
// Removed: use crate::services::utils::html_utils::unescape_html;

#[post("/regenerate-chat-message")]
pub async fn regenerate_chat_message(
    app_state: web::Data<AppState>,
    data: web::Json<RegenerateChatMessageRequest>,
) -> HttpResponse {
    let llm_service = LlmService::new();
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

    // Now, we need the actual user message content and history *up to that user message* for the LLM prompt.
    let user_message_index = full_history.iter().position(|msg| msg.id == user_message_id);
    if user_message_index.is_none() {
        return HttpResponse::BadRequest().body("Could not find the user message linked to the regeneration target.");
    }
    let user_message_index = user_message_index.unwrap();

    let actual_user_message_from_history = full_history[user_message_index].clone();
    // Content is already assumed to be raw markdown, no unescape needed.
    let actual_user_message_for_llm = ChatMessage {
        content: actual_user_message_from_history.content.clone(), // Raw markdown
        ..actual_user_message_from_history.clone()
    };

    // Truncate the history for the LLM prompt to include only messages *before* the user message
    let history_for_llm_context: Vec<ChatMessage> = full_history[0..user_message_index].to_vec();

    let (context_files, file_contents) = get_context_and_contents(&project, &app_state, &query_id);
    let query_text = project_service.query_manager
        .get_query_data_field(&project_dir, &query_id, "query")
        .unwrap_or_else(|| "No previous query found".to_string());
    let include_file_descriptions = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "include_file_descriptions").unwrap_or_else(|| "false".to_string()) == "true";
    let system_prompt = create_system_prompt(&query_text, &context_files, &file_contents, &project, include_file_descriptions);

    let mut unescaped_history_for_llm: Vec<ChatMessage> = Vec::new();
    let mut hidden_context: Vec<String> = Vec::new();
    for message in history_for_llm_context.iter() {
        if message.role == "git-flag" {
            continue;
        }
        let code = match (message.role.as_str(), message.hidden) {
            ("user", false) => "P",
            ("user", true) => "p",
            ("model", false) => "R",
            ("model", true) => "r",
            _ => "",
        };
        if !code.is_empty() {
            hidden_context.push(code.to_string());
        }
        // Content is assumed to be raw markdown, no unescape needed.
        unescaped_history_for_llm.push(ChatMessage {
            role: message.role.clone(),
            content: message.content.clone(), // Raw markdown
            hidden: message.hidden,
            commit_hash: message.commit_hash.clone(),
            timestamp: message.timestamp,
            context_files: message.context_files.clone(),
            provider: message.provider.clone(),
            model: message.model.clone(),
            hidden_context: message.hidden_context.clone(),
            ..Default::default()
        });
    }

    let messages = format_messages_for_llm(&system_prompt, &unescaped_history_for_llm, &actual_user_message_for_llm);

    let llm_response = llm_service
        .send_conversation(&messages, &project.provider.clone(), project.specific_model.as_deref())
        .await;

    // Create a NEW assistant message for the regenerated response (raw markdown)
    let new_assistant_message = ChatMessage {
        role: "model".to_string(),
        content: llm_response.clone(), // Raw markdown
        hidden: false,
        commit_hash: actual_user_message_from_history.commit_hash.clone(),
        timestamp: Some(chrono::Utc::now()),
        context_files: actual_user_message_from_history.context_files.clone(),
        provider: actual_user_message_from_history.provider.clone(),
        model: actual_user_message_from_history.model.clone(),
        hidden_context: actual_user_message_from_history.hidden_context.clone(),
        ..Default::default()
    };

    // Add the new message, explicitly setting its parent_id to the user message
    // This creates a new branch point from the user message.
    let new_assistant_message_id = match project_service.chat_manager.add_chat_message(
        &project_service.query_manager,
        &project_dir,
        new_assistant_message,
        query_id,
        Some(user_message_id) // The parent is the user message
    ) {
        Ok(id) => id,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to save regenerated message as new branch: {}", e)),
    };

    // --- NEW: Update the QueryData's current_node_id to point to the new message ---
    let mut query_data = match project_service.query_manager.load_query_data(&project_dir, query_id) {
        Ok(qd) => qd,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load query data for current_node_id update: {}", e)),
    };
    query_data.current_node_id = Some(new_assistant_message_id);
    if let Err(e) = project_service.query_manager.save_query_data(&project_dir, &query_data, query_id) {
        eprintln!("Failed to save query data after regenerating message: {}", e);
        return HttpResponse::InternalServerError().body(format!("Failed to update query's current node: {}", e));
    }
    // --- END NEW ---

    // Fetch the newly created message to return its full data
    let new_model_message_to_return = project_service.query_manager.get_chat_node(&project_dir, query_id, &new_assistant_message_id)
        .ok_or_else(|| HttpResponse::InternalServerError().body("Failed to retrieve new regenerated model message."));


    // Return JSON response instead of redirect
    HttpResponse::Ok().json(RegenerateChatMessageResponse {
        success: true,
        new_model_message: new_model_message_to_return.unwrap(),
        new_current_node_id: new_assistant_message_id,
        user_message_id, // Pass the ID of the user message that was the parent
    })
}