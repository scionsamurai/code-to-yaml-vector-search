// src/routes/llm/chat_analysis/handlers.rs
use super::models::*;
use super::utils::*;
use crate::models::AppState;
use crate::models::ChatMessage;
use crate::services::llm_service::LlmService;
use crate::services::project_service::ProjectService;
use actix_web::{post, web, HttpResponse};
use serde_json::Value;
use std::path::Path;

#[post("/chat-analysis")]
pub async fn chat_analysis(
    app_state: web::Data<AppState>,
    data: web::Json<ChatAnalysisRequest>,
) -> HttpResponse {
    let llm_service = LlmService::new();
    let project_service = ProjectService::new();

    // Load the project
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

    // Get selected context files and file contents
    let (context_files, file_contents) = get_context_and_contents(&project, &app_state, &query_id);

    let query_text = project
        .get_query_data_field(&app_state, query_id, "query")
        .unwrap_or_else(|| "No previous query found".to_string());

    // Create context prompt with the loaded file contents
    let system_prompt = create_system_prompt(&query_text, &context_files, &file_contents);

    // Get existing chat history from project structure
    let full_history = get_full_history(&project, &app_state, &query_id);

    let user_message = ChatMessage {
        role: "user".to_string(),
        content: data.message.clone(),
        hidden: false,
    };

    let mut unescaped_history: Vec<ChatMessage> = Vec::new();
    for message in full_history.iter() {
        let unescaped_content = unescape_html(message.content.clone());
        unescaped_history.push(ChatMessage {
            role: message.role.clone(),
            content: unescaped_content,
            hidden: message.hidden,
        });
    }

    // Format messages for LLM with system prompt and existing history
    let messages = format_messages_for_llm(&system_prompt, &unescaped_history, &user_message);

    // Send to LLM
    let llm_response = llm_service
        .send_conversation(&messages, &project.provider.clone(), project.specific_model.as_deref())
        .await;

    // Escape the user's message
    let escaped_message = escape_html(data.message.clone()).await;

    let user_message_to_save = ChatMessage { // Renamed to avoid shadowing
        role: "user".to_string(),
        content: escaped_message.to_string(),
        hidden: false,
    };

    // Create response message
    let assistant_message = ChatMessage {
        role: "model".to_string(),
        content: llm_response.clone(),
        hidden: false,
    };
    println!("appstate: {:?}", app_state); // This println is probably for debug, keeping it for now
    // Add messages to chat
    project
        .add_chat_message(&app_state, user_message_to_save, query_id) // Use renamed variable
        .unwrap();
    project
        .add_chat_message(&app_state, assistant_message, query_id)
        .unwrap();

    HttpResponse::Ok().body(llm_response)
}

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

    let query_text = project
        .get_query_data_field(&app_state, query_id, "query")
        .unwrap_or_else(|| "No previous query found".to_string());

    let system_prompt = create_system_prompt(&query_text, &context_files, &file_contents);

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
    let actual_user_message = full_history[user_message_index.unwrap()].clone();

    // Truncate the history to exclude the model message being regenerated and its preceding user message
    // This allows `format_messages_for_llm` to correctly add the system prompt + existing history + new user message.
    full_history.truncate(user_message_index.unwrap());

    let mut unescaped_history_for_llm: Vec<ChatMessage> = Vec::new();
    for message in full_history.iter() {
        let unescaped_content = unescape_html(message.content.clone());
        unescaped_history_for_llm.push(ChatMessage {
            role: message.role.clone(),
            content: unescaped_content,
            hidden: message.hidden,
        });
    }
    
    // Format messages for LLM with system prompt, truncated history, and the user message that led to the response being regenerated
    let messages = format_messages_for_llm(&system_prompt, &unescaped_history_for_llm, &actual_user_message);

    let llm_response = llm_service
        .send_conversation(&messages, &project.provider.clone(), project.specific_model.as_deref())
        .await;

    // Update the message in the history
    let new_assistant_message = ChatMessage {
        role: "model".to_string(),
        content: llm_response.clone(),
        hidden: false, // Regeneration implies it's not hidden
    };

    if let Err(e) = project.update_message_in_history(&app_state, message_index, new_assistant_message, query_id) {
        return HttpResponse::InternalServerError().body(format!("Failed to save regenerated message: {}", e));
    }

    HttpResponse::Ok().body(llm_response)
}


#[post("/reset-analysis-chat")]
pub async fn reset_analysis_chat(
    app_state: web::Data<AppState>,
    data: web::Json<ResetAnalysisChatRequest>,
) -> HttpResponse {
    let project_service = ProjectService::new();

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project);

    let project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to load project: {}", e))
        }
    };

    // Reset the chat history
    let new_file_name = project.reset_chat_history(&app_state, data.query_id.as_deref().unwrap());

    // Save the updated project settings
    if let Err(e) = project_service.save_project(&project, &project_dir) {
        return HttpResponse::InternalServerError().body(format!("Failed to save project: {}", e));
    }

    HttpResponse::Ok().body(new_file_name)
}

#[post("/update-chat-message")]
pub async fn update_chat_message(
    app_state: web::Data<AppState>,
    data: web::Json<UpdateChatMessageRequest>,
) -> HttpResponse {
    let project_service = ProjectService::new();

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project);

    let project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to load project: {}", e))
        }
    };

    let message = ChatMessage {
        role: data.role.clone(),
        content: data.content.clone(),
        hidden: data.hidden.unwrap_or(false),
    };

    // Update the specific message in the chat history
    let result = project.update_message_in_history(
        &app_state,
        data.index,
        message,
        data.query_id.as_deref().unwrap(),
    );

    match result {
        Ok(()) => HttpResponse::Ok().finish(), // Changed from body(message) to finish()
        Err(message) => HttpResponse::BadRequest().body(message),
    }
}

#[post("/update-message-visibility")]
pub async fn update_message_visibility(
    app_state: web::Data<AppState>,
    data: web::Json<Value>,
) -> HttpResponse {
    let project_service = ProjectService::new();

    // Parse request JSON
    let req = data.into_inner();

    let project_name = match req.get("project").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return HttpResponse::BadRequest().body("Missing 'project'"),
    };

    let index = match req.get("index").and_then(|v| v.as_u64()) {
        Some(i) => i as usize,
        None => return HttpResponse::BadRequest().body("Missing or invalid 'index'"),
    };

    let hidden = match req.get("hidden").and_then(|v| v.as_bool()) {
        Some(b) => b,
        None => return HttpResponse::BadRequest().body("Missing or invalid 'hidden'"),
    };

    let query_id = match req.get("query_id").and_then(|v| v.as_str()) {
        Some(q) => q,
        None => return HttpResponse::BadRequest().body("Missing 'query_id'"),
    };

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&project_name);

    let project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to load project: {}", e))
        }
    };

    // Update the message visibility
    let result = project.update_message_visibility(&app_state, index, hidden, query_id);

    match result {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(message) => HttpResponse::BadRequest().body(message),
    }
}