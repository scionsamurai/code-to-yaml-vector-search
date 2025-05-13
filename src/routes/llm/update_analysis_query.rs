// src/routes/llm/update_analysis_query.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::{AppState, QueryData};
use crate::services::project_service::ProjectService;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize)]
pub struct UpdateQueryRequest {
    project: String,
    query: String,
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
            // Try to get the most recent query file
            let most_recent_file = project_service.get_most_recent_query_file(&project_dir);
            let mut query_data: QueryData = QueryData::default();
            let mut filename: Option<String> = None;

            match most_recent_file {
                Ok(most_recent_file) => {
                    match most_recent_file {
                        Some(file_path) => {
                            let file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();

                            // Load the query data from the most recent file
                            match project_service.load_query_data(&project_dir, &file_name) {
                                Ok(qd) => {
                                    query_data = qd;
                                    filename = Some(file_name);
                                },
                                Err(e) => {
                                    eprintln!("Failed to load query data: {}", e);
                                    // If loading fails, start with default QueryData
                                }
                            }
                        },
                        None => {
                            // No recent file, so start with default QueryData
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get most recent query file: {}", e);
                    // If getting the most recent file fails, start with default QueryData
                }
            }
            // Update the query in the QueryData
            query_data.query = req_body.query.clone();

            // Get filename or generate new one if it doesn't exist
            let query_filename = match filename {
                Some(s) => s,
                None => project_service.generate_query_filename(),
            };

            // Save the updated QueryData
            match project_service.save_query_data(&project_dir, &query_data, &query_filename) {
                Ok(_) => {
                    // Save the updated project - we don't need to track filenames anymore.
                    match project_service.save_project(&project, &project_dir) {
                        Ok(_) => {
                            HttpResponse::Ok().json(UpdateQueryResponse {
                                success: true,
                                message: "Query updated successfully".to_string(),
                            })
                        },
                        Err(e) => {
                            HttpResponse::InternalServerError().json(UpdateQueryResponse {
                                success: false,
                                message: format!("Failed to save project: {}", e),
                            })
                        }
                    }
                },
                Err(e) => {
                    HttpResponse::InternalServerError().json(UpdateQueryResponse {
                        success: false,
                        message: format!("Failed to save query data: {}", e),
                    })
                }
            }
        },
        Err(e) => {
            HttpResponse::NotFound().json(UpdateQueryResponse {
                success: false,
                message: format!("Project not found: {}", e),
            })
        }
    }
}