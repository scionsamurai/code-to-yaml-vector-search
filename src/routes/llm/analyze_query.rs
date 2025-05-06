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

    // Generate file lists - relevant files are no longer default checked
    let relevant_files_html = generate_file_list(&relevant_files, &saved_context_files);
    let other_files_html = generate_other_files_list(&project, &relevant_files, &saved_context_files);

    // Extract existing chat messages if any
    let existing_chat_html = if let Some(saved_queries) = &project.saved_queries {
        if let Some(last_query) = saved_queries.last() {
            if let Some(chat_history) = last_query.get("analysis_chat_history") {
                if let Some(history_array) = chat_history.as_array() {
                    let mut html = String::new();
                    for msg in history_array {
                        if let (Some(role), Some(content)) = (msg.get("role").and_then(Value::as_str), 
                                                          msg.get("content").and_then(Value::as_str)) {
                            html.push_str(&format!(
                                r#"<div class="chat-message {}-message">
                                    <div class="message-content">{}</div>
                                </div>"#,
                                role,
                                content.replace("\n", "<br>")
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
    
    // Render the HTML with the chat interface
    let html = format!(
        r#"
        <html>
            <head>
                <title>Code Analysis - {}</title>
                <link rel="stylesheet" href="/static/project.css">
                <link rel="stylesheet" href="/static/analyze-query.css">
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
                        <h2>Files for Analysis</h2>
                        
                        <div id="context-status" style="display: none; margin: 10px 0; padding: 5px; 
                            background-color: #f0f0f0; border-radius: 4px; transition: opacity 0.5s;">
                        </div>
                        
                        <div class="file-list">
                            <h3>
                                Relevant Files 
                                <button id="toggle-relevant-files" class="toggle-button">Toggle All</button>
                            </h3>
                            <div id="relevant-files-list">
                                {}
                            </div>
                        </div>
                        
                        <div class="file-list">
                            <h3>
                                Other Project Files
                                <button id="toggle-other-files" class="toggle-button">Toggle All</button>
                            </h3>
                            <div id="other-files-list">
                                {}
                            </div>
                        </div>
                        
                    </div>
                    
                    <div class="chat-interface">
                        <h2>Analysis Chat</h2>
                        <input type="hidden" id="project-name" value="{}">
                        <input type="hidden" id="query-text" value="{}">
                        
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
        relevant_files_html,
        other_files_html,
        form.project,
        form.query,
        existing_chat_html,
        form.project
    );
    
    HttpResponse::Ok().body(html)
}

fn generate_file_list(files: &[String], selected_files: &[String]) -> String {
    files.iter()
        .map(|file| {
            format!(
                r#"<div class="file-item">
                    <input type="checkbox" class="file-checkbox" value="{}" {}> {}
                </div>"#,
                file,
                if selected_files.contains(file) { "checked" } else { "" },
                file
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_other_files_list(project: &crate::models::Project, exclude_files: &[String], selected_files: &[String]) -> String {
    // Get all project files
    let all_files: Vec<String> = match &project.embeddings {
        embeddings => embeddings.keys().cloned().collect(),
    };

    // Filter out the files that are already in the relevant files list
    let other_files: Vec<String> = all_files.into_iter()
        .filter(|file| !exclude_files.contains(file))
        .collect();
    
    generate_file_list(&other_files, selected_files)
}