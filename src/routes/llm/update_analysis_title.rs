// src/routes/llm/update_analysis_title.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize)]
pub struct UpdateTitleRequest {
    project: String,
    title: String,
    query_id: String,
}

#[derive(Serialize)]
pub struct UpdateTitleResponse {
    success: bool,
    message: String,
}

#[post("/update-analysis-title")]
pub async fn update_analysis_title(
    app_state: web::Data<AppState>,
    req_body: web::Json<UpdateTitleRequest>,
) -> impl Responder {
    let project_service = ProjectService::new();

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&req_body.project);

    match project_service.load_project(&project_dir) {
        Ok(project) => {
            // Update the title
            match project_service.query_manager.update_query_title(&project_dir, &req_body.query_id, &req_body.title) { // Updated line
                Ok(_) => {
                    // Save the project after the update (important!)
                    if let Err(e) = project_service.save_project(&project, &project_dir) {
                        eprintln!("Error saving project after title update: {}", e);
                        return HttpResponse::InternalServerError().json(UpdateTitleResponse {
                            success: false,
                            message: format!("Failed to save project after title update: {}", e),
                        });
                    }
                    HttpResponse::Ok().json(UpdateTitleResponse {
                        success: true,
                        message: "Title updated successfully".to_string(),
                    })
                }
                Err(e) => {
                    HttpResponse::InternalServerError().json(UpdateTitleResponse {
                        success: false,
                        message: format!("Failed to update title: {}", e),
                    })
                }
            }
        }
        Err(e) => {
            HttpResponse::NotFound().json(UpdateTitleResponse {
                success: false,
                message: format!("Project not found: {}", e),
            })
        }
    }
}