// src/routes/llm/update_analysis_query.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use serde::{Deserialize, Serialize};
use std::path::Path;
use serde_json::json;

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
        Ok(mut project) => {
            // Update the most recent saved query or create a new one if none exists
            if let Some(saved_queries) = &mut project.saved_queries {
                if let Some(last_query) = saved_queries.last_mut() {
                    // Update the query in the last saved query
                    last_query["query"] = json!(req_body.query);
                } else {
                    // Create a new query entry if list exists but is empty
                    saved_queries.push(json!({
                        "query": req_body.query,
                        "vector_results": [],
                        "context_files": [],
                        "analysis_chat_history": []
                    }));
                }
            } else {
                // Create a new saved_queries list with the current query
                project.saved_queries = Some(vec![json!({
                    "query": req_body.query,
                    "vector_results": [],
                    "context_files": [],
                    "analysis_chat_history": []
                })]);
            }
            
            // Save the updated project
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
            HttpResponse::NotFound().json(UpdateQueryResponse {
                success: false,
                message: format!("Project not found: {}", e),
            })
        }
    }
}