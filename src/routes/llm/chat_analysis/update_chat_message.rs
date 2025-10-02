// src/routes/llm/chat_analysis/update_chat_message.rs
use super::models::*;
use crate::models::AppState;
use crate::models::ChatMessage;
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

    let message = ChatMessage {
        role: data.role.clone(),
        content: data.content.clone(),
        hidden: data.hidden.unwrap_or(false),
    };

    // Update the specific message in the chat history
    let result = project_service.chat_manager.update_message_in_history(
        &project_service.query_manager,
        &project_dir,
        data.index,
        message,
        data.query_id.as_deref().unwrap(),
    );

    match result {
        Ok(()) => HttpResponse::Ok().finish(), // Changed from body(message) to finish()
        Err(message) => HttpResponse::BadRequest().body(message),
    }
}