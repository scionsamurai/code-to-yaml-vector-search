// src/routes/project/submit_query.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use crate::services::search_service::SearchService;
use std::path::Path;
use crate::services::utils::html_utils::escape_html;

#[derive(serde::Deserialize)]
pub struct SubmitQueryForm {
    q: String,
}

#[post("/projects/{name}")]
pub async fn submit_query(
    app_state: web::Data<AppState>,
    name: web::Path<String>,
    form: web::Form<SubmitQueryForm>,
) -> impl Responder {
    let name = name.into_inner();
    let output_dir = Path::new(&app_state.output_dir).join(&name);

    // Initialize services
    let project_service = ProjectService::new();
    let search_service = SearchService::new();

    // Load project
    let mut project = match project_service.load_project(&output_dir) {
        Ok(project) => project,
        Err(e) => return HttpResponse::NotFound().body(format!("Project not found: {}", e)),
    };

    let query_text = &form.q;
    let escaped_query_text = escape_html(query_text.clone()).await;

    if !escaped_query_text.is_empty() {
        // Execute new search. The results are saved within search_project.
        let num_search_results = 5;
        if let Err(e) = search_service.search_project(&mut project, &escaped_query_text, Some(&output_dir), num_search_results, true).await {
            // Log the error, but still redirect. A more advanced implementation might use flash messages.
            eprintln!("Error during search for project '{}': {}", name, e);
        }
    }

    // Redirect back to the project page. The GET handler will then display the results we just saved.
    HttpResponse::SeeOther()
        .append_header(("Location", format!("/projects/{}", name)))
        .finish()
}