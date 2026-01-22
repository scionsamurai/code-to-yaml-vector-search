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
    query_id: String,
    pub include_file_descriptions: bool,
    pub grounding_with_search: bool, // ADDED: New field for grounding
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

    // Use the query_id directly from the request to target the correct query
    let query_id_to_update = &data.query_id;

    // Fetch the last query text based on the specific query_id
    let last_query_text = project_service.query_manager.get_query_data_field(&project_dir, query_id_to_update, "query")
        .unwrap_or_else(|| "No previous query found".to_string());
    
    // Generate a reference prompt without including file contents
    // (This prompt is for the success message, not necessarily the LLM prompt itself)
    let updated_prompt = format!(
        "You are an AI assistant helping with code analysis for a project. \
        The user's query is: \"{}\"\n\n\
        You have access to the following files:\n{}",
        last_query_text,
        data.files.join("\n")
    );

    // Load the specific query data using the query_id_to_update
    match project_service.query_manager.load_query_data(&project_dir, query_id_to_update) {
        Ok(mut query_data) => {
            // Update the context files and description flag
            query_data.context_files = data.files.clone();
            query_data.include_file_descriptions = data.include_file_descriptions; 
            query_data.grounding_with_search = data.grounding_with_search; // ADDED: Update grounding flag

            // Save the updated QueryData using the provided query_id_to_update as the filename
            match project_service.query_manager.save_query_data(&project_dir, &query_data, query_id_to_update) {
                Ok(_) => {
                    return HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "prompt": updated_prompt
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