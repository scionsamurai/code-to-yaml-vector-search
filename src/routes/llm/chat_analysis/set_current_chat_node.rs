// src/routes/llm/chat_analysis/set_current_chat_node.rs
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::path::Path;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SetCurrentChatNodeRequest {
    project_name: String,
    query_id: String,
    new_current_node_id: Uuid,
}

#[post("/set-current-chat-node")]
pub async fn set_current_chat_node(
    app_state: web::Data<AppState>,
    data: web::Json<SetCurrentChatNodeRequest>,
) -> impl Responder {
    let project_service = ProjectService::new();

    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project_name);

    let result = project_service.chat_manager.set_current_node(
        &project_service.query_manager,
        &project_dir,
        &data.query_id,
        data.new_current_node_id,
    );

    match result {
        Ok(()) => {
            // After setting the current node, we might want to tell the frontend
            // to reload the chat history for the current query to reflect the new branch.
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            eprintln!("Failed to set current chat node: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to set current chat node: {}", e))
        }
    }
}