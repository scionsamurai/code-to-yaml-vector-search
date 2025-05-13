// src/routes/llm/chat_split.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::AppState;
use crate::services::llm_service::LlmService;
use crate::services::project_service::ProjectService;
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
    let project_path = Path::new(&app_state.output_dir).join(project_name);
    
    let project_service = ProjectService::new();
    let project = match project_service.load_project(&project_path) {
        Ok(project) => project,
        Err(e) => return HttpResponse::NotFound().body(format!("Error loading project: {}", e)),
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
