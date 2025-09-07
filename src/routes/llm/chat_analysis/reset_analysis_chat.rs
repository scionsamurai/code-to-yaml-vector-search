// src/routes/llm/chat_analysis/reset_analysis_chat.rs
use super::models::*;
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use actix_web::{post, web, HttpResponse};
use std::path::Path;

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