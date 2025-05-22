// src/routes/project/path_comment.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::AppState;
use std::path::Path;
use crate::services::project_service::ProjectService;
use crate::services::file::FileService;

#[post("/projects/{name}/validate_paths")]
pub async fn validate_paths(
    app_state: web::Data<AppState>,
    name: web::Path<String>,
) -> impl Responder {
    let name = name.into_inner();
    let output_dir = Path::new(&app_state.output_dir).join(&name);

    // Initialize services
    let project_service = ProjectService::new();
    let file_service = FileService {};

    // Load project
    let project = match project_service.load_project(&output_dir) {
        Ok(project) => project,
        Err(e) => return HttpResponse::NotFound().body(format!("Project not found: {}", e)),
    };

    let results = file_service.validate_file_paths(&project);

    HttpResponse::Ok().json(results)
}