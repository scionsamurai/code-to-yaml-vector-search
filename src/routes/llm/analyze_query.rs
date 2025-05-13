// src/routes/llm/analyze_query.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use crate::services::template::TemplateService;
use std::path::Path;

#[derive(serde::Deserialize)]
pub struct AnalyzeQueryForm {
    project: String,
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
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load project: {}", e)),
    };

    // Use the new methods to get the data
    let relevant_files = project.get_vector_results(&app_state);
    let saved_context_files = project.get_context_files(&app_state);
    let existing_chat_html = project.get_analysis_chat_history(&app_state);
    let last_query_text = project.get_query_text(&app_state).unwrap_or_else(|| "No previous query found".to_string());
    
    // Use the template service to render the HTML
    let html = template_service.render_analyze_query_page(
        &form.project,
        &last_query_text,
        &relevant_files,
        &saved_context_files,
        &project,
        &existing_chat_html
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