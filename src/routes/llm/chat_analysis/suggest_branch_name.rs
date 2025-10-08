// src/routes/llm/suggest_branch_name.rs
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use crate::models::{AppState, ChatMessage};
use crate::services::llm_service::LlmService;
use crate::services::project_service::ProjectService;
use std::path::Path;
use serde_json::json;

#[derive(Deserialize)]
pub struct SuggestBranchNameRequest {
    project_name: String,
    query_id: String, // Or chat_history_summary: String,
}

#[post("/suggest-branch-name")]
pub async fn suggest_branch_name(
    app_state: web::Data<AppState>,
    data: web::Json<SuggestBranchNameRequest>,
) -> impl Responder {
    let llm_service = LlmService::new();
    let project_service = ProjectService::new();

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project_name);
    let project = project_service.load_project(&project_dir).unwrap();

    // Build chat history for llm prompt
    let chat_history: Vec<ChatMessage> = project_service.chat_manager.get_analysis_chat_history(&project_service.query_manager, &project_dir, &data.query_id);

    let system_prompt = "You are a Git branch name suggestion assistant. Based on the following chat conversation, suggest a short, descriptive Git branch name (e.g., 'feature/add-user-auth', 'bugfix/login-crash', 'refactor/database-connector'). The name should be based on what you assume the current focus is. Respond with only the suggested branch name, nothing else.".to_string();

    // Format messages for LLM.  We only need the chat history
    let mut messages: Vec<ChatMessage> = Vec::new();
    messages.push(ChatMessage { role: "system".to_string(), content: system_prompt, hidden: false, commit_hash: None });
    messages.extend(chat_history);

    let llm_response = llm_service.send_conversation(&messages, &project.provider.clone(), project.specific_model.as_deref()).await;

    HttpResponse::Ok().json(json!({ "branch_name": llm_response }))
}