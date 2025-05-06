// src/routes/llm/update_analysis_context.rs
use actix_web::{post, web, HttpResponse};
use serde::Deserialize;
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use std::path::Path;

#[derive(Deserialize)]
pub struct UpdateContextRequest {
    project: String,
    query: String,
    files: Vec<String>,
}

#[post("/update-analysis-context")]
pub async fn update_analysis_context(
    app_state: web::Data<AppState>,
    data: web::Json<UpdateContextRequest>,
) -> HttpResponse {
    let project_service = ProjectService::new();
    
    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project);
    
    let mut project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to load project: {}", e)
        })),
    };
    
    // Generate a reference prompt without including file contents
    let updated_prompt = format!(
        "You are an AI assistant helping with code analysis for a project. \
        The user's query is: \"{}\"\n\n\
        You have access to the following files:\n{}",
        data.query,
        data.files.join("\n")
    );
    
    // Update the saved context in the project - only store file references
    if let Some(saved_queries) = &mut project.saved_queries {
        if let Some(last_query) = saved_queries.last_mut() {
            // Store only the file references
            last_query["context_files"] = serde_json::to_value(&data.files).unwrap_or_default();
        }
    }
    
    // Save the updated project settings
    if let Err(e) = project_service.save_project(&project, &project_dir) {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to save project: {}", e)
        }));
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "prompt": updated_prompt
    }))
}