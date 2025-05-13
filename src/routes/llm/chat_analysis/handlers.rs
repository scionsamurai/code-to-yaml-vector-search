// src/routes/llm/chat_analysis/handlers.rs
use actix_web::{post, web, HttpResponse};
use crate::models::ChatMessage;
use crate::models::AppState;
use crate::services::llm_service::LlmService;
use crate::services::project_service::ProjectService;
use std::path::Path;
use super::models::*;
use super::utils::*;

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
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load project: {}", e)),
    };

    // Get selected context files and file contents
    let (context_files, file_contents) = get_context_and_contents(&project, &app_state);

    // Escape the user's message
    let escaped_message = escape_html(data.message.clone()).await;
    
    let last_query_text = project.get_query_text(&app_state).unwrap_or_else(|| "No previous query found".to_string());
     
    // Create context prompt with the loaded file contents
    let system_prompt = create_system_prompt(&last_query_text, &context_files, &file_contents);

    // Get existing chat history from project structure
    let full_history = get_full_history(&project, &app_state);

    let user_message = ChatMessage {
        role: "user".to_string(),
        content: escaped_message.to_string(),
    };

    // Format messages for LLM with system prompt and existing history
    let messages = format_messages(&system_prompt, &full_history, &user_message);

    // Send to LLM
    let llm_response = llm_service.send_conversation(&messages, &project.model.clone()).await;

    // Create response message
    let assistant_message = ChatMessage {
        role: "model".to_string(),
        content: llm_response.clone(),
    };
    // Add messages to chat
    project.add_chat_message(&app_state, user_message).unwrap();
    project.add_chat_message(&app_state, assistant_message).unwrap();

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
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load project: {}", e)),
    };

    // Reset the chat history
    project.reset_chat_history(&app_state).unwrap();

    // Save the updated project settings
    if let Err(e) = project_service.save_project(&project, &project_dir) {
        return HttpResponse::InternalServerError().body(format!("Failed to save project: {}", e));
    }

    HttpResponse::Ok().body("Chat history reset successfully")
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
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load project: {}", e)),
    };

    let message = ChatMessage {
        role: data.role.clone(),
        content: data.content.clone()
    };

    // Update the specific message in the chat history
    let result = project.update_message_in_history(&app_state, data.index, message);

    match result {
        Ok(message) => HttpResponse::Ok().body(message),
        Err(message) => HttpResponse::BadRequest().body(message)
    }
}