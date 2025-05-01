// src/routes/project/get_project.rs
use actix_web::{get, web, HttpResponse, Responder};
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use crate::services::search_service::SearchService;
use crate::services::template_service::TemplateService;
use crate::services::yaml::YamlService;
use std::path::Path;

#[derive(serde::Deserialize)]
struct QueryParams {
    q: Option<String>,
}

#[get("/projects/{name}")]
pub async fn get_project(
    app_state: web::Data<AppState>, 
    name: web::Path<String>,
    query: web::Query<QueryParams>
) -> impl Responder {
    let name = name.into_inner();
    let output_dir = Path::new(&app_state.output_dir).join(&name);
    
    // Initialize services
    let project_service = ProjectService::new();
    let search_service = SearchService::new();
    let template_service = TemplateService::new();
    let yaml_service = YamlService::new();
    
    // Load project
    let mut project = match project_service.load_project(&output_dir) {
        Ok(project) => project,
        Err(e) => return HttpResponse::NotFound().body(format!("Project not found: {}", e)),
    };
    
    // Check for manually updated YAML files and update embeddings if needed
    yaml_service.check_and_update_yaml_files(&mut project, &app_state.output_dir).await;
    
    // Initialize HTML for search results
    let mut search_results_html = String::new();
    
    // Handle search query if present
    if let Some(query_text) = &query.q {
        if !query_text.is_empty() {
            // Execute search
            match search_service.search_project(&mut project, query_text).await {
                Ok(similar_files) => {
                    // Save updated project with the new query
                    if let Err(e) = project_service.save_project(&project, &output_dir) {
                        eprintln!("Failed to save project: {}", e);
                    }
                    
                    // Render search results
                    search_results_html = template_service.render_search_results(
                        query_text,
                        &similar_files,
                        &project.name
                    );
                },
                Err(e) => {
                    search_results_html = format!(
                        r#"<div class="search-results">
                            <h2>Error searching: {}</h2>
                        </div>"#,
                        e
                    );
                }
            }
        }
    } else if let Some(saved_queries) = &project.saved_queries {
        // Show most recent saved query if no new query
        if !saved_queries.is_empty() {
            if let Some(latest_query) = saved_queries.last() {
                if let Some(query_text) = latest_query.get("query").and_then(|q| q.as_str()) {
                    if let Some(results) = latest_query.get("results").and_then(|r| r.as_array()) {
                        let mut similar_files = Vec::new();
                        
                        for result in results {
                            if let (Some(file_path), Some(yaml_content), Some(score)) = (
                                result[0].as_str(),
                                result[1].as_str(),
                                result[2].as_f64()
                            ) {
                                similar_files.push((
                                    file_path.to_string(),
                                    yaml_content.to_string(),
                                    score as f32
                                ));
                            }
                        }
                        
                        // Render search results for previous query
                        search_results_html = template_service.render_search_results(
                            query_text,
                            &similar_files,
                            &project.name
                        );
                    }
                }
            }
        }
    }
    
    // Get YAML files HTML
    let yaml_files = match project_service.get_yaml_files_html(&output_dir, &project.name) {
        Ok(html) => html,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Error getting YAML files: {}", e));
        }
    };
    
    // Render full page
    let html = template_service.render_project_page(
        &project,
        &search_results_html,
        &yaml_files,
        query.q.as_deref().unwrap_or("")
    );
    
    HttpResponse::Ok().body(html)
}