// src/routes/llm/chat_analysis/regenerate_chat_message.rs
use super::models::*;
use super::utils::*;
use crate::models::AppState;
use crate::models::ChatMessage;
use crate::services::llm_service::LlmService;
use crate::services::project_service::ProjectService;
use actix_web::{post, web, HttpResponse};
use std::path::Path;
use crate::services::utils::html_utils::unescape_html;
use crate::routes::llm::chat_analysis::ChatMessageMetadata;

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
    let message_index = data.index;

    // Get selected context files and file contents
    let (context_files, file_contents) = get_context_and_contents(&project, &app_state, &query_id);

    let query_text = project_service.query_manager
        .get_query_data_field(&project_dir, &query_id, "query")
        .unwrap_or_else(|| "No previous query found".to_string());

    let include_file_descriptions = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "include_file_descriptions").unwrap_or_else(|| "false".to_string()) == "true";

    let system_prompt = create_system_prompt(&query_text, &context_files, &file_contents, &project, include_file_descriptions);

    let mut full_history = get_full_history(&project, &app_state, &query_id);

    // Ensure the index is valid and it's a model message
    if message_index >= full_history.len() || full_history[message_index].role != "model" {
        return HttpResponse::BadRequest().body("Invalid message index or not a model message for regeneration.");
    }

    // Identify the user message that prompted this model response
    // We assume that a model response is always preceded by a user message.
    let user_message_index = message_index.checked_sub(1);
    if user_message_index.is_none() || full_history[user_message_index.unwrap()].role != "user" {
        return HttpResponse::BadRequest().body("Could not find a preceding user message for regeneration.");
    }

    // Clone the actual user message from history (content is currently escaped)
    let actual_user_message_escaped = full_history[user_message_index.unwrap()].clone();

    // Create an unescaped version of the user message for the LLM
    let actual_user_message_unescaped = ChatMessage {
        content: unescape_html(actual_user_message_escaped.content.clone()),
        ..actual_user_message_escaped.clone() // Copy other fields
    };
    
    // Truncate the history to exclude the model message being regenerated and its preceding user message
    // This allows `format_messages_for_llm` to correctly add the system prompt + existing history + new user message.
    full_history.truncate(user_message_index.unwrap());

    let mut unescaped_history_for_llm: Vec<ChatMessage> = Vec::new();
    let mut hidden_context: Vec<String> = Vec::new();
    for message in full_history.iter() {
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
        let unescaped_content = unescape_html(message.content.clone());
        unescaped_history_for_llm.push(ChatMessage {
            role: message.role.clone(),
            content: unescaped_content,
            hidden: message.hidden,
            commit_hash: message.commit_hash.clone(),
            timestamp: message.timestamp,
            context_files: message.context_files.clone(),
            provider: message.provider.clone(),
            model: message.model.clone(),
            hidden_context: message.hidden_context.clone(),
        });
    }
    
    // Format messages for LLM with system prompt, truncated history, and the user message that led to the response being regenerated
    let messages = format_messages_for_llm(&system_prompt, &unescaped_history_for_llm, &actual_user_message_unescaped);

    let llm_response = llm_service
        .send_conversation(&messages, &project.provider.clone(), project.specific_model.as_deref())
        .await;

    // Capture metadata for the regenerated assistant message
    let assistant_message_metadata = ChatMessageMetadata {
        timestamp: Some(chrono::Utc::now()),
        context_files: Some(context_files.clone()), // Clone the context files
        provider: Some(project.provider.clone()),
        model: project.specific_model.clone(),
        hidden_context: Some(hidden_context),
    };

    // Update the message in the history
    let new_assistant_message = ChatMessage {
        role: "model".to_string(),
        content: llm_response.clone(),
        hidden: false,
        commit_hash: actual_user_message_escaped.commit_hash.clone(),
        timestamp: assistant_message_metadata.timestamp,
        context_files: assistant_message_metadata.context_files,
        provider: assistant_message_metadata.provider,
        model: assistant_message_metadata.model,
        hidden_context: assistant_message_metadata.hidden_context,
    };

    if let Err(e) = project_service.chat_manager.update_message_in_history(&project_service.query_manager, &project_dir, message_index, new_assistant_message, query_id) {
        return HttpResponse::InternalServerError().body(format!("Failed to save regenerated message: {}", e));
    }

    HttpResponse::Ok().body(llm_response)
}
