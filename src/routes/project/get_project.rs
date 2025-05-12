// src/routes/project/get_project.rs
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use crate::services::search_service::SearchService;
use crate::services::template::TemplateService;
use crate::services::yaml::YamlService;
use actix_web::{get, web, HttpResponse, Responder};
use std::path::Path;
use crate::routes::llm::chat_analysis::utils::escape_html;

#[derive(serde::Deserialize)]
struct QueryParams {
    q: Option<String>,
}

#[get("/projects/{name}")]
pub async fn get_project(
    app_state: web::Data<AppState>,
    name: web::Path<String>,
    query: web::Query<QueryParams>,
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
    yaml_service
        .check_and_update_yaml_files(&mut project, &app_state.output_dir)
        .await;

    // Initialize HTML for search results
    let mut search_results_html = String::new();

    // Check if a new query is provided
    if let Some(query_text) = &query.q {
        let escaped_query_text = escape_html(query_text.clone()).await;
        if !escaped_query_text.is_empty() {
            // Check if we already have this query saved
            let existing_query = project.saved_queries.as_ref()
                .and_then(|queries| queries.iter()
                    .find(|q| q.get("query").and_then(|q_text| q_text.as_str()) == Some(query_text)));
    
            if let Some(saved_query) = existing_query {
                // Use existing saved query results
                let similar_files = extract_vector_results(saved_query);
                let llm_analysis = saved_query.get("llm_analysis")
                    .and_then(|a| a.as_str())
                    .unwrap_or("No analysis available");
                
                // Render search results from saved data
                search_results_html = template_service.render_search_results(
                    &escaped_query_text,
                    &similar_files,
                    llm_analysis,
                    &project.name
                );
            } else {
                // No saved query found, execute new search
                match search_service.search_project(&mut project, &escaped_query_text).await {
                    Ok((similar_files, llm_analysis)) => {
                        // Save updated project with the new query
                        if let Err(e) = project_service.save_project(&project, &output_dir) {
                            eprintln!("Failed to save project: {}", e);
                        }
                        
                        // Render search results
                        search_results_html = template_service.render_search_results(
                            &escaped_query_text,
                            &similar_files,
                            &llm_analysis,
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
        }
    } else if let Some(saved_queries) = &project.saved_queries {
        // Show most recent saved query if no new query
        if !saved_queries.is_empty() {
            if let Some(latest_query) = saved_queries.last() {
                if let Some(query_text) = latest_query.get("query").and_then(|q| q.as_str()) {
                    let similar_files = extract_vector_results(latest_query);
                    let llm_analysis = latest_query.get("llm_analysis")
                        .and_then(|a| a.as_str())
                        .unwrap_or("No analysis available");
                    
                    // Render search results for previous query
                    search_results_html = template_service.render_search_results(
                        query_text,
                        &similar_files,
                        llm_analysis,
                        &project.name
                    );
                }
            }
        }
    }

    // Get YAML files HTML
    let yaml_files = match project_service.get_yaml_files_html(&output_dir, &project.name) {
        Ok(html) => html,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Error getting YAML files: {}", e));
        }
    };

    // Render full page
    let html = template_service.render_project_page(
        &project,
        &search_results_html,
        &yaml_files,
        query.q.as_deref().unwrap_or(""),
    );

    HttpResponse::Ok().body(html)
}

// Helper function to extract vector results from a saved query
fn extract_vector_results(query: &serde_json::Value) -> Vec<(String, String, f32)> {
    let mut similar_files = Vec::new();
    
    if let Some(results) = query.get("vector_results").and_then(|r| r.as_array()) {
        for result in results {
            if let Some(result_array) = result.as_array() {
                if result_array.len() >= 3 {
                    if let (Some(file_path), Some(yaml_content), Some(score)) = (
                        result_array[0].as_str(),
                        result_array[1].as_str(),
                        result_array[2].as_f64()
                    ) {
                        similar_files.push((
                            file_path.to_string(),
                            yaml_content.to_string(),
                            score as f32
                        ));
                    }
                }
            }
        }
    }
    
    similar_files
}