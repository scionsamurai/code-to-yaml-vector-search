// src/routes/llm/analyze_query.rs
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use crate::services::template::TemplateService;
use actix_web::{post, web, HttpResponse, Responder};
use std::path::Path;

#[derive(serde::Deserialize, Debug)]
pub struct AnalyzeQueryForm {
    project: String,
    query_id: String,
}


#[post("/analyze-query")]
pub async fn analyze_query(
    app_state: web::Data<AppState>,
    form: web::Form<AnalyzeQueryForm>,
) -> impl Responder {
    let project_service = ProjectService::new();
    let template_service = TemplateService::new();

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&form.project);

    let project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to load project: {}", e))
        }
    };

    let query_id = form.query_id.clone();
    let relevant_files = project.get_query_vec_field(&app_state, &query_id, "vector_results").unwrap();
    let saved_context_files = project.get_query_vec_field(&app_state, &query_id, "context_files").unwrap();
    let existing_chat_history = project.get_analysis_chat_history(&app_state, &query_id); // Changed to Vec<ChatMessage>
    let last_query_text = project
        .get_query_data_field(&app_state, &query_id, "query")
        .unwrap_or_else(|| "No previous query found".to_string());
    
    // Get the list of available queries
    let available_queries = match project.get_query_filenames(&app_state) {
        Ok(queries) => queries,
        Err(e) => {
            eprintln!("Failed to get query filenames: {}", e);
            Vec::new()
        }
    };

    let include_file_descriptions = project.get_query_data_field(&app_state, &query_id, "include_file_descriptions").unwrap_or_else(|| "false".to_string()) == "true";

    // Use the template service to render the HTML
    let html = template_service.render_analyze_query_page(
        &form.project,
        &last_query_text,
        &relevant_files,
        &saved_context_files,
        &project,
        &existing_chat_history, // Pass the Vec<ChatMessage>
        &available_queries,
        &query_id,
        include_file_descriptions
    );

    HttpResponse::Ok().body(html)
}

pub fn _format_message(content: &str) -> String {
    // Create a regex to match triple backtick code blocks
    let re = regex::Regex::new(r"```([a-zA-Z]*)([\s\S]*?)```").unwrap();

    // Replace triple backtick code blocks with formatted HTML
    let formatted_content = re.replace_all(&content, |caps: &regex::Captures| {
        let language = &caps[1];
        let code = caps[2].trim();
        format!("<pre class=\"shiki-block\" data-language=\"{}\" data-original-code=\"{}\"><code class=\"language-{}\">{}</code></pre>", language, code, language, code)
    });

    // Replace newlines with <br> tags for normal text (outside of code blocks)
    // let formatted_content = formatted_content.replace("\n", "<br>");

    formatted_content.to_string()
}
