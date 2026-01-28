// src/routes/llm/chat_analysis/update_agentic_mode.rs

use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use std::path::Path;

#[derive(Deserialize)]
pub struct UpdateAgenticModeRequest {
    pub project: String,
    pub query_id: String,
    pub enabled: bool,
}

#[post("/llm/chat_analysis/update_agentic_mode")]
pub async fn update_agentic_mode(
    app_state: web::Data<AppState>,
    data: web::Json<UpdateAgenticModeRequest>,
) -> impl Responder {
    let project_service = ProjectService::new();
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project);
    let query_id = &data.query_id;

    match project_service
        .query_manager
        .update_query_data_in_project(&project_dir, query_id, |qd| {
            qd.agentic_mode_enabled = data.enabled;
        }) {
        Ok(_) => HttpResponse::Ok().body("Agentic mode updated successfully"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to update agentic mode: {}", e)),
    }
}