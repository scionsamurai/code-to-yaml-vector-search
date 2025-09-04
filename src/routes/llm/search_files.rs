// src/routes/llm/search_files.rs
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use crate::services::search_service::SearchService;
use crate::services::template::TemplateService;
use crate::services::yaml::YamlService;
use actix_web::{post, web, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::routes::llm::chat_analysis::utils::escape_html;

#[derive(Deserialize)]
pub struct SearchRequest {
    project: String,
    query: String,
}

#[derive(Serialize)]
struct SearchResponse {
    success: bool,
    html: String, // Return the HTML string
    error: Option<String>,
}

#[post("/search-related-files")]
pub async fn search_related_files(
    app_state: web::Data<AppState>,
    req: web::Json<SearchRequest>,
) -> Result<HttpResponse> {
    // Initialize services
    let project_service = ProjectService::new();
    let search_service = SearchService::new();
    let template_service = TemplateService::new();
    let yaml_service = YamlService::new();

    let project_name = &req.project;
    let output_dir = Path::new(&app_state.output_dir).join(project_name);

    // Load project
    let mut project = match project_service.load_project(&output_dir) {
        Ok(project) => project,
        Err(e) => return Ok(HttpResponse::InternalServerError().json(SearchResponse {
            success: false,
            html: String::new(),
            error: Some(format!("Failed to load project: {}", e)),
        })),
    };

    // Check for manually updated YAML files and update embeddings if needed
    yaml_service
        .check_and_update_yaml_files(&mut project, &app_state.output_dir)
        .await;

    let escaped_query_text = escape_html(req.query.clone()).await;

    // Execute search
    let search_result = search_service.search_project(&mut project, &escaped_query_text, None).await;

    let search_results_html = match search_result {
        Ok((similar_files, llm_analysis)) => {
            // Render search results
            template_service.render_search_results(
                &escaped_query_text,
                &similar_files,
                &llm_analysis,
                &project.name,
                "transient_query_id", // A temporary query ID
            )
        }
        Err(e) => {
            format!(
                r#"<div class="search-results"><h2>Error searching: {}</h2></div>"#,
                e
            )
        }
    };

    println!("Search results HTML: {}", search_results_html);

    let response = SearchResponse {
        success: true,
        html: search_results_html,
        error: None,
    };

    Ok(HttpResponse::Ok().json(response))
}