// src/routes/llm/analyze_query.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use crate::services::template::TemplateService;
use std::path::Path;
use serde_json::Value;

#[derive(serde::Deserialize)]
pub struct AnalyzeQueryForm {
    project: String,
    query: String,
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
    
    // Get relevant files from the latest saved query
    let relevant_files = if let Some(saved_queries) = &project.saved_queries {
        if let Some(last_query) = saved_queries.last() {
            if let Some(vector_results) = last_query.get("vector_results") {
                if let Some(results_array) = vector_results.as_array() {
                    results_array.iter()
                        .filter_map(|result| {
                            if let Some(file_path) = result.get(0)?.as_str() {
                                Some(file_path.to_string())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<String>>()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    
    // Get saved context files from the project if available
    let saved_context_files = if let Some(saved_queries) = &project.saved_queries {
        if let Some(last_query) = saved_queries.last() {
            if let Some(files) = last_query.get("context_files") {
                if let Some(files_array) = files.as_array() {
                    files_array.iter()
                        .filter_map(|f| f.as_str().map(String::from))
                        .collect::<Vec<String>>()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Extract existing chat messages if any
    let existing_chat_html = if let Some(saved_queries) = &project.saved_queries {
        if let Some(last_query) = saved_queries.last() {
            if let Some(chat_history) = last_query.get("analysis_chat_history") {
                if let Some(history_array) = chat_history.as_array() {
                    let mut html = String::new();
                    for msg in history_array {
                        if let (Some(role), Some(content)) = (msg.get("role").and_then(Value::as_str), 
                                                          msg.get("content").and_then(Value::as_str)) {
                            // let formatted_content = format_message(content);
                            html.push_str(&format!(
                                r#"<div class="chat-message {}-message">
                                    <div class="message-content">{}</div>
                                    <div class="message-controls">
                                        <button class="edit-message-btn" title="Edit message">Edit</button>
                                    </div>
                                </div>"#,
                                role,
                                content
                            ));
                        }
                    }
                    html
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    
    // Use the template service to render the HTML
    let html = template_service.render_analyze_query_page(
        &form.project,
        &form.query,
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