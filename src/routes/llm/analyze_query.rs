// src/routes/llm/analyze_query.rs
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use crate::services::template::TemplateService;
use actix_web::{post, web, HttpResponse, Responder};
use std::path::Path;
use serde_json::Value;
use crate::services::utils::html_utils::unescape_html;

#[derive(serde::Deserialize, Debug)]
pub struct AnalyzeQueryForm {
    project: String,
    query_id: String,
}

// New helper function to encapsulate the LLM analysis parsing logic
fn process_llm_analysis_suggestions(
    project_service: &ProjectService,
    project_dir: &Path,
    query_id: &str,
    original_relevant_files: Vec<String>,
) -> (Vec<String>, Vec<String>) {
    let mut llm_suggested_files: Vec<String> = Vec::new();
    let mut actual_relevant_files: Vec<String> = Vec::new();

    if let Some(llm_analysis_str) = project_service.query_manager.get_query_data_field(project_dir, query_id, "llm_analysis") {
        let llm_analysis_unescaped = unescape_html(llm_analysis_str);
        let llm_analysis_json_str = llm_analysis_unescaped
            .split("```json")
            .nth(1)
            .and_then(|s| s.split("```").next())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "".to_string());

        match serde_json::from_str::<Value>(&llm_analysis_json_str) {
            Ok(json_value) => {
                if let Some(files_array) = json_value["suggested_files"].as_array() {
                    for file_val in files_array {
                        if let Some(file_path) = file_val.as_str() {
                            llm_suggested_files.push(file_path.to_string());
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("Failed to parse LLM analysis JSON for query {}: {}", query_id, e);
                // Continue without LLM suggested files
            }
        }
    }

    // Filter out any llm_suggested_files from the original_relevant_files
    for file in original_relevant_files {
        if !llm_suggested_files.contains(&file) {
            actual_relevant_files.push(file);
        }
    }

    (llm_suggested_files, actual_relevant_files)
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
    // Fetch the original vector_results from the query data
    let original_relevant_files = project_service.query_manager.get_query_vec_field(&project_dir, &query_id, "vector_results").unwrap_or_default();
    let saved_context_files = project_service.query_manager.get_query_vec_field(&project_dir, &query_id, "context_files").unwrap_or_default();
    let existing_chat_history = project_service.chat_manager.get_analysis_chat_history(&project_service.query_manager, &project_dir, &query_id);
    let last_query_text = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "query").unwrap_or_else(|| "No previous query found".to_string());
    let include_file_descriptions = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "include_file_descriptions").unwrap_or_else(|| "false".to_string()) == "true";

    // Call the new helper function
    let (llm_suggested_files, actual_relevant_files) = process_llm_analysis_suggestions(
        &project_service,
        &project_dir,
        &query_id,
        original_relevant_files,
    );

    // Get the list of available queries
    let available_queries = match project_service.query_manager.get_query_filenames(&project_dir) {
        Ok(queries) => queries,
        Err(e) => {
            eprintln!("Failed to get query filenames: {}", e);
            Vec::new()
        }
    };

    // Use the template service to render the HTML
    let html = template_service.render_analyze_query_page(
        &form.project,
        &last_query_text,
        &actual_relevant_files, // Pass the filtered relevant files
        &saved_context_files,
        &project,
        &existing_chat_history, // Pass the Vec<ChatMessage>
        &available_queries,
        &query_id,
        include_file_descriptions,
        &llm_suggested_files, // PASS THE NEW LLM SUGGESTED FILES
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