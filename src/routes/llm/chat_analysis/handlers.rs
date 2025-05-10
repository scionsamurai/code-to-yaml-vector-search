// src/routes/llm/chat_analysis/handlers.rs
use actix_web::{post, web, HttpResponse};
use crate::models::ChatMessage;
use crate::models::AppState;
use crate::services::llm_service::LlmService;
use crate::services::project_service::ProjectService;
use crate::services::file_service::FileService;
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
    let file_service = FileService {};

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project);

    let mut project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load project: {}", e)),
    };

    // Get selected context files and file contents
    let (context_files, file_contents) = get_context_and_contents(&project, &file_service);

    // Create context prompt with the loaded file contents
    let system_prompt = create_system_prompt(&data.query, &context_files, &file_contents);

    // Get existing chat history from project structure
    let mut full_history = get_full_history(&project);

    // Format messages for LLM with system prompt and existing history
    let messages = format_messages(&system_prompt, &full_history, &data.message);

    // Send to LLM
    let model = project.model.clone();
    let llm_response = llm_service.send_conversation(&messages, &model).await;

    // Create response message
    let assistant_message = ChatMessage {
        role: "model".to_string(),
        content: llm_response.clone(),
    };

    // Add new message pair to history
    full_history.push(ChatMessage {
        role: "user".to_string(),
        content: data.message.clone(),
    });

    full_history.push(assistant_message);

    // Save the updated chat history to the project settings
    update_and_save_history(&mut project, &project_dir, full_history, project_service);

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

    let mut project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load project: {}", e)),
    };

    // Reset the chat history
    reset_chat_history(&mut project);

    // Save the updated project settings
    if let Err(e) = project_service.save_project(&project, &project_dir) {
        return HttpResponse::InternalServerError().body(format!("Failed to save project: {}", e));
    }

    HttpResponse::Ok().body("Chat history reset successfully")
}

#[post("/save-analysis-history")]
pub async fn save_analysis_history(
    app_state: web::Data<AppState>,
    data: web::Json<SaveAnalysisHistoryRequest>,
) -> HttpResponse {
    let project_service = ProjectService::new();

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project);

    let mut project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load project: {}", e)),
    };

    // Save the chat history to the project settings
    save_chat_history(&mut project, &data.history);

    // Save the updated project settings
    if let Err(e) = project_service.save_project(&project, &project_dir) {
        return HttpResponse::InternalServerError().body(format!("Failed to save project: {}", e));
    }

    HttpResponse::Ok().body("Chat history saved successfully")
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

    let mut project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load project: {}", e)),
    };

    // Update the specific message in the chat history
    let result = update_message_in_history(&mut project, &data);

    match result {
        Ok(message) => HttpResponse::Ok().body(message),
        Err(message) => HttpResponse::BadRequest().body(message)
    }
}