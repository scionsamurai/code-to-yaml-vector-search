// src/routes/query/update_auto_commit.rs
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use std::path::Path;

#[derive(Deserialize)]
pub struct UpdateAutoCommitRequest {
    project_name: String,
    query_id: String,
    auto_commit: bool,
}

#[post("/update-query-auto-commit")]
pub async fn update_auto_commit(
    app_state: web::Data<AppState>,
    data: web::Json<UpdateAutoCommitRequest>,
) -> impl Responder {
    let project_service = ProjectService::new();

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project_name);

    // Get the query_id to update from the request
    let query_id_to_update = &data.query_id;

    // Load the specific query data using the query_id_to_update
    match project_service.query_manager.load_query_data(&project_dir, query_id_to_update) {
        Ok(mut query_data) => {
            // Update the auto_commit flag
            query_data.auto_commit = data.auto_commit;

            // Save the updated QueryData using the provided query_id_to_update as the filename
            match project_service.query_manager.save_query_data(&project_dir, &query_data, query_id_to_update) {
                Ok(_) => {
                    return HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                    }));
                }
                Err(e) => {
                    eprintln!("Failed to save query data for query_id {}: {}", query_id_to_update, e);
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to save query data: {}", e)
                    }));
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to load query data for query_id {}: {}", query_id_to_update, e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to load query data for query_id {}: {}", query_id_to_update, e)
            }));
        }
    }
}