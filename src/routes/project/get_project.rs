// src/routes/project/get_project.rs
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use crate::services::search_service::{ SearchService, SearchResult };
use crate::services::template::TemplateService;
use crate::services::yaml::YamlService;
use actix_web::{get, web, HttpResponse, Responder};
use std::path::Path;
use crate::services::utils::html_utils::escape_html;

#[derive(serde::Deserialize)]
struct QueryParams {
    q: Option<String>,
    id: Option<String>,
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

    // if query.id is None, load the most recent query
    let q_id = if let Some(id) = &query.id {
        id.clone()
    } else {
        project_service.query_manager
            .get_recent_query_id(&output_dir)
            .unwrap_or_default()
    };

    // Check if a new query is provided
    if let Some(query_text) = &query.q {
        let escaped_query_text = escape_html(query_text.clone()).await;
        if !escaped_query_text.is_empty() {
            // Execute new search
            let num_search_results = 5;
            match search_service.search_project(&mut project, &escaped_query_text, Some(&output_dir), num_search_results, true).await {
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
                        &project.name,
                        &q_id
                    );
                },
                Err(e) => {
                    search_results_html = format!(
                        r#"<div class=&"search-results&">;
                            <h2>;Error searching: {}</h2>;
                        </div>;"#,
                        e
                    );
                }
            }
        }
    } else {
        // Load most recent query data
        let most_recent_query = project_service.query_manager
            .load_most_recent_query_data(&output_dir);
        match most_recent_query {
            Ok(Some(latest_query)) => {
                let query_text = latest_query.query.clone();
                let similar_files: Vec<SearchResult> = latest_query.vector_results
                    .iter()
                    .map(|(path, score)| SearchResult { file_path: path.clone(), file_description: None, score: *score, file_content: "".to_string(), embedding: None })
                    .collect();

                let llm_analysis = latest_query.llm_analysis.clone();

                // Render search results for previous query
                search_results_html = template_service.render_search_results(
                    &query_text,
                    &similar_files,
                    &llm_analysis,
                    &project.name,
                    &q_id
                );
            }
            Ok(None) => {
                // No saved query found, display a default message
                search_results_html = r#"<div class="search-results"><p>No previous queries found.</p></div>"#.to_string();
            }
            Err(e) => {
                // Error loading query data, display an error message
                search_results_html = format!(
                    r#"<div class="search-results">
                        <h2>Error loading previous query: {}</h2>
                    </div>"#,
                    e
                );
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

    project_service.cleanup_embeddings_on_load(&mut project, &output_dir);

    // Render full page
    let html = template_service.render_project_page(
        &project,
        &search_results_html,
        &yaml_files,
        query.q.as_deref().unwrap_or(""),
        q_id.as_str(),
    );

    HttpResponse::Ok().body(html)
}