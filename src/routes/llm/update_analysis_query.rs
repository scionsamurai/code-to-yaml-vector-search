// src/routes/llm/update_analysis_query.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::{AppState, QueryData};
use crate::services::project_service::ProjectService;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct UpdateQueryRequest {
    project: String,
    query: String,
    query_id: String,
}

#[derive(Serialize)]
pub struct UpdateQueryResponse {
    success: bool,
    message: String,
}

#[post("/update-analysis-query")]
pub async fn update_analysis_query(
    app_state: web::Data<AppState>,
    req_body: web::Json<UpdateQueryRequest>,
) -> impl Responder {
    let project_service = ProjectService::new();

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&req_body.project);

    match project_service.load_project(&project_dir) {
        Ok(project) => {

            let query_filename = match Uuid::parse_str(&req_body.query_id.trim_end_matches(".json")) {
                Ok(_) => req_body.query_id.clone(), // It's a valid UUID, assume it's filename
                Err(_) => {
                    // It's not a UUID, try to match with title
                    let available_queries = project.get_query_filenames(&app_state).unwrap_or_default();
                    let mut matched_filename: Option<String> = None;

                    for (filename, _) in available_queries {
                        if filename == req_body.query_id {
                            matched_filename = Some(filename);
                            break; // Assuming titles are unique, break on first match
                        }
                    }

                    match matched_filename {
                        Some(filename) => filename,
                        None => project_service.generate_query_filename(), // If title not found, create new query
                    }
                }
            };

            // Try to load existing query data or create new
            let (mut query_data, filename) =
                match project.load_query_data_by_filename(&app_state, &query_filename) {
                    Ok(Some(qd)) => (qd, query_filename.to_string()),
                    _ => (
                        QueryData::default(),
                        project_service.generate_query_filename(),
                    ),
                };

            // Update the query in the QueryData
            query_data.query = req_body.query.clone();

            // Save the updated QueryData
            match project_service.save_query_data(&project_dir, &query_data, &filename) {
                Ok(_) => {
                    // Save the updated project - we don't need to track filenames anymore.
                    match project_service.save_project(&project, &project_dir) {
                        Ok(_) => {
                            HttpResponse::Ok().json(UpdateQueryResponse {
                                success: true,
                                message: "Query updated successfully".to_string(),
                            })
                        }
                        Err(e) => {
                            HttpResponse::InternalServerError().json(UpdateQueryResponse {
                                success: false,
                                message: format!("Failed to save project: {}", e),
                            })
                        }
                    }
                }
                Err(e) => {
                    HttpResponse::InternalServerError().json(UpdateQueryResponse {
                        success: false,
                        message: format!("Failed to save query data: {}", e),
                    })
                }
            }
        }
        Err(e) => {
            HttpResponse::NotFound().json(UpdateQueryResponse {
                success: false,
                message: format!("Project not found: {}", e),
            })
        }
    }
}