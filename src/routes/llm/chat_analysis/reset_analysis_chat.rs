// src/routes/llm/chat_analysis/reset_analysis_chat.rs
use super::models::*;
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use actix_web::{post, web, HttpResponse};
// Removed: use actix_web::http::header;
use std::path::Path;
use uuid::Uuid;

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

    let query_id_str = data.query_id.as_deref().unwrap();

    // Reset the chat history. This function returns the ID of the new root message.
    let new_root_message_id: Result<(), String> = project_service.chat_manager.reset_chat_history(&project_service.query_manager, &project_dir, query_id_str);

    // Load the updated project to get the full (reset) chat history for the frontend.
    // Re-load the query data to get the complete chat graph for this query.
    let updated_query_data = match project_service.query_manager.load_query_data(&project_dir, query_id_str) {
        Ok(qd) => qd,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to reload query data after reset: {}", e)),
    };

    // Assuming new_root_message_id is the new current_node_id.
    // If reset_chat_history correctly sets current_node_id, then we just need to retrieve it.
    let initial_chat_history = project_service.chat_manager.get_analysis_chat_history(&project_service.query_manager, &project_dir, query_id_str);

    // Return JSON response instead of redirect
    let new_node_id = match Uuid::parse_str(query_id_str) {
        Ok(id) => id,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Invalid UUID format: {}", e)),
    };
    
    HttpResponse::Ok().json(ResetAnalysisChatResponse {
        success: true,
        initial_chat_history,
        new_current_node_id: new_node_id, // Return the ID of the new root message
    })
}