// src/routes/llm/optimize_prompt.rs
use crate::models::AppState;
use crate::services::llm_service::LlmService;
use crate::services::project_service::ProjectService;
use actix_web::{post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::routes::llm::chat_analysis::utils as chat_utils;
use crate::services::utils::html_utils::{escape_html, unescape_html};

#[derive(Deserialize)]
pub struct OptimizePromptRequest {
    project: String,
    query_id: String, // To fetch context
    original_prompt: String,
    optimization_direction: String,
    #[serde(default)]
    include_chat_history: bool,
    #[serde(default)]
    include_context_files: bool,
}

#[derive(Serialize)]
struct OptimizePromptResponse {
    success: bool,
    optimized_prompt: String,
    error: Option<String>,
}

#[post("/optimize-prompt")]
pub async fn optimize_prompt_route(
    app_state: web::Data<AppState>,
    req: web::Json<OptimizePromptRequest>,
) -> Result<HttpResponse> {
    let project_service = ProjectService::new();
    let llm_service = LlmService::new();

    let project_name = &req.project;
    let output_dir = Path::new(&app_state.output_dir).join(project_name);

    let project = match project_service.load_project(&output_dir) {
        Ok(project) => project,
        Err(e) => return Ok(HttpResponse::InternalServerError().json(OptimizePromptResponse {
            success: false,
            optimized_prompt: String::new(),
            error: Some(format!("Failed to load project: {}", e)),
        })),
    };

    let mut chat_history_for_prompt = None;
    if req.include_chat_history {
        let mut history = chat_utils::get_full_history(&project, &app_state, &req.query_id);
        chat_utils::replace_hidden_messages(&mut history); // Reuse helper to handle hidden messages
        let formatted_history = history.iter()
            .map(|msg| format!("{}: {}", msg.role, unescape_html(msg.content.clone())))
            .collect::<Vec<String>>()
            .join("\n");
        chat_history_for_prompt = Some(formatted_history);
    }

    let mut file_context_for_prompt = None;
    if req.include_context_files {
        let (_, file_contents) = chat_utils::get_context_and_contents(&project, &app_state, &req.query_id);
        if !file_contents.is_empty() {
            file_context_for_prompt = Some(file_contents);
        }
    }

    let provider = &project.provider;
    let specific_model = project.specific_model.as_deref();

    let escaped_original_prompt = escape_html(req.original_prompt.clone()).await;
    let escaped_optimization_direction = escape_html(req.optimization_direction.clone()).await;
    
    let optimized_prompt_result = llm_service.get_optimized_prompt(
        &escaped_original_prompt,
        if escaped_optimization_direction.is_empty() { None } else { Some(&escaped_optimization_direction) },
        chat_history_for_prompt.as_deref(),
        file_context_for_prompt.as_deref(),
        provider,
        specific_model,
    ).await;

    match optimized_prompt_result {
        Ok(optimized_prompt) => {
            Ok(HttpResponse::Ok().json(OptimizePromptResponse {
                success: true,
                optimized_prompt,
                error: None,
            }))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(OptimizePromptResponse {
                success: false,
                optimized_prompt: String::new(),
                error: Some(format!("Failed to optimize prompt with LLM: {}", e)),
            }))
        }
    }
}