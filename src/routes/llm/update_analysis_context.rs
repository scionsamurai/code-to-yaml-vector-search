// src/routes/llm/update_analysis_context.rs
use actix_web::{post, web, HttpResponse};
use serde::Deserialize;
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use std::path::Path;

#[derive(Deserialize)]
pub struct UpdateContextRequest {
    project: String,
    files: Vec<String>,
    query_id: Option<String>,
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
    
    let project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to load project: {}", e)
        })),
    };
    let query_id = data.query_id.as_deref().unwrap_or_default();
    let last_query_text = project.get_query_text(&app_state, query_id).unwrap_or_else(|| "No previous query found".to_string());
    
    // Generate a reference prompt without including file contents
    let updated_prompt = format!(
        "You are an AI assistant helping with code analysis for a project. \
        The user's query is: \"{}\"\n\n\
        You have access to the following files:\n{}",
        last_query_text,
        data.files.join("\n")
    );

    
    // Load the most recent query data
    match project_service.get_most_recent_query_file(&project_dir) {
        Ok(Some(file_path)) => {
            let filename = file_path.file_name().unwrap().to_str().unwrap().to_string();
            match project_service.load_query_data(&project_dir, &filename) {
                Ok(mut query_data) => {
                    // Update the context files
                    query_data.context_files = data.files.clone();

                    // Save the updated QueryData
                    match project_service.save_query_data(&project_dir, &query_data, &filename) {
                        Ok(_) => {
                            return HttpResponse::Ok().json(serde_json::json!({
                                "success": true,
                                "prompt": updated_prompt
                            }));
                        }
                        Err(e) => {
                            eprintln!("Failed to save query data: {}", e);
                            return HttpResponse::InternalServerError().json(serde_json::json!({
                                "success": false,
                                "error": format!("Failed to save query data: {}", e)
                            }));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load query data: {}", e);
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to load query data: {}", e)
                    }));
                }
            }
        }
        _ => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": "No query data found"
            }));
        }
    }
}
