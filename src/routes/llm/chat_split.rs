// src/routes/llm/chat_split.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::{AppState, Project};
use crate::services::llm_service::LlmService;
use std::fs::read_to_string;
use std::path::Path;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ChatMessage {
    project: String,
    message: String,
    history: Vec<MessageHistory>,
}

#[derive(Deserialize)]
pub struct MessageHistory {
    role: String,
    content: String,
}

#[post("/chat-split")]
pub async fn chat_split(
    app_state: web::Data<AppState>,
    message: web::Json<ChatMessage>,
) -> impl Responder {
    let project_name = &message.project;
    
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
    
    // Construct the prompt with history context
    let mut prompt = String::new();
    
    // Add previous conversation context
    for hist_msg in &message.history {
        match hist_msg.role.as_str() {
            "system" => prompt.push_str(&format!("Initial context:\n{}\n\n", hist_msg.content)),
            "user" => prompt.push_str(&format!("User: {}\n\n", hist_msg.content)),
            "assistant" => prompt.push_str(&format!("Assistant: {}\n\n", hist_msg.content)),
            _ => {}
        }
    }
    
    // Add current message
    prompt.push_str(&format!("User: {}\n\nAssistant:", message.message));
    
    // Use the LLM service to get the response
    let llm_service = LlmService::new();
    let response = llm_service.get_analysis(&prompt, &project.model).await;
    
    HttpResponse::Ok().body(response)
}
