// src/routes/llm/analyze_query.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use crate::services::file_service::FileService;
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
    let file_service = FileService {};
    
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
    
    // Get file contents for display
    let mut file_snippets = String::new();
    for file_path in &relevant_files {
        if let Some(content) = file_service.read_specific_file(&project, file_path) {
            file_snippets.push_str(&format!(
                r#"<div class="code-snippet">
                    <h3>{}</h3>
                    <pre><code>{}</code></pre>
                </div>"#,
                file_path, html_escape::encode_text(&content)
            ));
        }
    }
    
    // Generate the initial system prompt for the chat
    let initial_prompt = format!(
        "You are an AI assistant helping with code analysis for a project. \
        The user's query is: \"{}\"\n\n\
        You have access to the following files that were found through vector search:\n{}\n\n\
        Answer the user's questions about these files and help them understand the code.",
        form.query,
        relevant_files.join("\n")
    );
    
    // Extract existing chat messages if any
    let existing_chat_html = if let Some(saved_queries) = &project.saved_queries {
        if let Some(last_query) = saved_queries.last() {
            if let Some(chat_history) = last_query.get("analysis_chat_history") {
                if let Some(history_array) = chat_history.as_array() {
                    let mut html = String::new();
                    for msg in history_array {
                        if let (Some(role), Some(content)) = (msg.get("role").and_then(Value::as_str), 
                                                          msg.get("content").and_then(Value::as_str)) {
                            if role != "system" {
                                html.push_str(&format!(
                                    r#"<div class="chat-message {}-message">
                                        <div class="message-content">{}</div>
                                    </div>"#,
                                    role,
                                    content.replace("\n", "<br>")
                                ));
                            }
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
    
    // Render the HTML with the chat interface
    let html = format!(
        r#"
        <html>
            <head>
                <title>Code Analysis - {}</title>
                <link rel="stylesheet" href="/static/project.css">
                <link rel="stylesheet" href="/static/split-chat.css">
                <script src="/static/analyze-query.js"></script>
            </head>
            <body>
                <div class="head">
                    <h1>Code Analysis</h1>
                    <p>Project: {}</p>
                    <p>Query: {}</p>
                </div>
                
                <div class="analysis-container">
                    <div class="file-snippets">
                        <h2>Relevant Code Files</h2>
                        {}
                    </div>
                    
                    <div class="chat-interface">
                        <h2>Analysis Chat</h2>
                        <input type="hidden" id="project-name" value="{}">
                        <input type="hidden" id="query-text" value="{}">
                        <input type="hidden" id="analysis-initial-prompt" value="{}">
                        
                        <div id="analysis-chat-container" class="chat-container">
                            {}
                        </div>
                        
                        <div class="chat-input">
                            <input type="text" id="analysis-message-input" placeholder="Ask a question about the code...">
                            <button id="analysis-send-button">Send</button>
                            <button id="analysis-reset-button" class="secondary">Reset Chat</button>
                        </div>
                    </div>
                </div>
                
                <div class="actions">
                    <a href="/projects/{}" class="button">Back to Project</a>
                </div>
                
                <script>
                    document.addEventListener('DOMContentLoaded', function() {{
                        initAnalysisChat();
                    }});
                </script>
            </body>
        </html>
        "#,
        form.project,
        form.project,
        form.query,
        file_snippets,
        form.project,
        form.query,
        html_escape::encode_text(&initial_prompt),
        existing_chat_html,
        form.project
    );
    
    HttpResponse::Ok().body(html)
}