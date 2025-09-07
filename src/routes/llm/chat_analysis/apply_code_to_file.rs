// src/routes/llm/chat_analysis/apply_code_to_file.rs
use super::models::*;
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use crate::services::file::FileService; // Import FileService
use actix_web::{post, web, HttpResponse};
use std::path::Path;

#[post("/apply-code-to-file")]
pub async fn apply_code_to_file(
    app_state: web::Data<AppState>,
    data: web::Json<ApplyCodeToFileRequest>,
) -> HttpResponse {
    let project_service = ProjectService::new();
    let file_service = FileService; // Create an instance of FileService

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

    // Call the FileService to write the content
    match file_service.write_file_content(&project, &data.file_path, &data.content) {
        Ok(_) => {
            println!("Server Debug: Successfully wrote content to {}", data.file_path); // Added this for explicit success logging
            HttpResponse::Ok().body(format!("Successfully applied code to {}", data.file_path))
        },
        Err(e) => {
            eprintln!("Server Error: Failed to apply code to {}: {}", data.file_path, e); // Changed to eprintln and made more explicit
            HttpResponse::InternalServerError().body(format!("Failed to apply code to {}: {}", data.file_path, e))
        },
    }
}