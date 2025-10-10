// src/routes/llm/chat_analysis/update_chat_message.rs
use super::models::*;
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use actix_web::{post, web, HttpResponse};
use std::path::Path;

#[post("/update-chat-message")]
pub async fn update_chat_message(
    app_state: web::Data<AppState>,
    data: web::Json<UpdateChatMessageRequest>,
) -> HttpResponse {
    let project_service = ProjectService::new();

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project);

    let query_id = data.query_id.as_deref().unwrap();
    let index = data.index;

    // Retrieve the existing chat history
    let chat_history = project_service.chat_manager.get_analysis_chat_history(
        &project_service.query_manager,
        &project_dir,
        query_id,
    );

    // Check if the index is valid
    if index >= chat_history.len() {
        return HttpResponse::BadRequest().body("Invalid message index.");
    }

    // Get the existing message
    let mut existing_message = chat_history[index].clone();

    // Update only the content of the existing message
    existing_message.content = data.content.clone();

    // Update the specific message in the chat history
    let result = project_service.chat_manager.update_message_in_history(
        &project_service.query_manager,
        &project_dir,
        index,
        existing_message,
        query_id,
    );

    match result {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(message) => HttpResponse::BadRequest().body(message),
    }
}