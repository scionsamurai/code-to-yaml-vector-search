// src/routes/suggest_split.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::{AppState, Project};
use crate::services::llm_service::LlmService;
use std::fs::read_to_string;
use std::path::Path;

#[post("/suggest-split")]
pub async fn suggest_split(
    app_state: web::Data<AppState>,
    query: web::Query<SplitParams>,
) -> impl Responder {
    let project_name = &query.project;
    let file_path = &query.file_path;
    
    // Get project details
    let project_settings_path = Path::new(&app_state.output_dir)
        .join(project_name)
        .join("project_settings.json");
    
    let project = match read_to_string(&project_settings_path) {
        Ok(json) => match serde_json::from_str::<Project>(&json) {
            Ok(project) => project,
            Err(e) => return HttpResponse::BadRequest().body(format!("Invalid project settings: {}", e)),
        },
        Err(e) => return HttpResponse::NotFound().body(format!("Project settings not found: {}", e)),
    };
    
    // Read the file content
    let file_path_full = Path::new(&project.source_dir).join(file_path);
    let file_content = match read_to_string(&file_path_full) {
        Ok(content) => content,
        Err(e) => return HttpResponse::NotFound().body(format!("File not found: {}", e)),
    };
    
    // Create a prompt for the LLM with file descriptions
    let file_descriptions_text = project.file_descriptions.iter()
        .map(|(path, desc)| format!("{}: {}", path, desc))
        .collect::<Vec<String>>()
        .join("\n");
    
    // Create a prompt for the LLM
    let initial_prompt = format!(
        "File Descriptions:\n{}\n\nI need to split a large file into smaller, more manageable pieces. Please suggest how to split this file into multiple smaller files.\n\n\
        File path: {}\n\n\
        Please provide a detailed plan for splitting this file, including:\n\
        1. The new files that should be created\n\
        2. What content should go in each file\n\
        3. How the files should reference each other\n\
        4. Any refactoring needed to maintain functionality\n\
        Be specific and provide actual code examples when appropriate.\n\n\
        File content:\n```\n{}\n```\n\n",
        file_descriptions_text,
        file_path,
        file_content
    );
    
    // Use the LLM service to get the analysis
    let llm_service = LlmService::new();
    let analysis = llm_service.get_analysis(&initial_prompt, &project.model).await;
    
    // Create HTML response with chat interface
    let html = format!(
        r#"
        <div class="modal-content split-chat-modal">
            <span class="close" onclick="this.parentElement.parentElement.remove()">&times;</span>
            <div class="chat-header">
                <h2>Split File: {}</h2>
                <p>Project: {}</p>
            </div>
            
            <input type="hidden" id="project-name" value="{}">
            <input type="hidden" id="file-path" value="{}">
            <input type="hidden" id="initial-prompt" value="{}">
            
            <div id="chat-container">
                <div class="chat-message assistant-message">
                    <div class="message-content">{}</div>
                </div>
            </div>
            
            <div class="chat-input">
                <input type="text" id="message-input" placeholder="Type your follow-up question here...">
                <button id="send-button">Send</button>
            </div>
        </div>
        "#,
        file_path,
        project_name,
        project_name,
        file_path,
        html_escape::encode_text(&initial_prompt), // Escape the initial prompt to prevent HTML issues
        analysis.replace("\n", "<br>")
    );
    
    HttpResponse::Ok().content_type("text/html").body(html)
}

#[derive(serde::Deserialize)]
struct SplitParams {
    project: String,
    file_path: String,
}