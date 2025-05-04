// src/routes/llm/chat_analysis.rs
use actix_web::{post, web, HttpResponse};
use crate::models::ChatMessage;
use serde::{Deserialize, Serialize};
use crate::models::AppState;
use crate::services::llm_service::LlmService;
use crate::services::file_service::FileService;
use crate::services::project_service::ProjectService;
use std::path::Path;

#[derive(Deserialize)]
pub struct ChatAnalysisRequest {
    project: String,
    query: String,
    message: String,
    history: Vec<ChatMessage>,
}


#[post("/chat-analysis")]
pub async fn chat_analysis(
    app_state: web::Data<AppState>,
    data: web::Json<ChatAnalysisRequest>,
) -> HttpResponse {
    let llm_service = LlmService::new();
    let project_service = ProjectService::new();
    
    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project);
    
    let mut project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load project: {}", e)),
    };
    
    // Create context prompt
    let system_prompt = match data.history.first() {
        Some(first_msg) if first_msg.role == "system" => first_msg.content.clone(),
        _ => {
            // If there's no system message, create one
            format!(
                "You are an AI assistant helping with code analysis for a project. \
                The user's original query was: \"{}\"\n\n\
                Answer the user's questions and help them understand the code.",
                data.query
            )
        }
    };
    
    // Format messages for LLM
    let mut messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        }
    ];
    
    // Add history messages, excluding any previous system messages
    messages.extend(
        data.history.iter()
            .filter(|msg| msg.role != "system")
            .cloned()
    );
    
    // Add the current user message
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: data.message.clone(),
    });
    
    // Send to LLM
    let model = project.model.clone();
    let llm_response = llm_service.send_conversation(&messages, &model).await;
    
    // Add response to messages
    let assistant_message = ChatMessage {
        role: "assistant".to_string(),
        content: llm_response.clone(),
    };
    messages.push(assistant_message);
    
    // Save the updated chat history to the project settings
    if project.saved_queries.is_none() {
        project.saved_queries = Some(Vec::new());
    }
    
    if let Some(saved_queries) = &mut project.saved_queries {
        if let Some(last_query) = saved_queries.last_mut() {
            // Update the last query with the analysis chat history
            last_query["analysis_chat_history"] = serde_json::to_value(&messages).unwrap_or_default();
            
            // Save the updated project settings
            if let Err(e) = project_service.save_project(&project, &project_dir) {
                eprintln!("Failed to save project: {}", e);
            }
        }
    }
    
    HttpResponse::Ok().body(llm_response)
}

#[derive(Deserialize)]
pub struct ResetAnalysisChatRequest {
    project: String,
}

#[post("/reset-analysis-chat")]
pub async fn reset_analysis_chat(
    app_state: web::Data<AppState>,
    data: web::Json<ResetAnalysisChatRequest>,
) -> HttpResponse {
    let project_service = ProjectService::new();
    
    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project);
    
    let mut project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load project: {}", e)),
    };
    
    // Reset the chat history
    if let Some(saved_queries) = &mut project.saved_queries {
        if let Some(last_query) = saved_queries.last_mut() {
            // Remove the analysis chat history
            last_query.as_object_mut().unwrap().remove("analysis_chat_history");
        }
    }
    
    // Save the updated project settings
    if let Err(e) = project_service.save_project(&project, &project_dir) {
        return HttpResponse::InternalServerError().body(format!("Failed to save project: {}", e));
    }
    
    HttpResponse::Ok().body("Chat history reset successfully")
}

#[derive(Deserialize)]
pub struct SaveAnalysisHistoryRequest {
    project: String,
    history: Vec<ChatMessage>,
}

#[post("/save-analysis-history")]
pub async fn save_analysis_history(
    app_state: web::Data<AppState>,
    data: web::Json<SaveAnalysisHistoryRequest>,
) -> HttpResponse {
    let project_service = ProjectService::new();
    
    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project);
    
    let mut project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load project: {}", e)),
    };
    
    // Save the chat history to the project settings
    if project.saved_queries.is_none() {
        project.saved_queries = Some(Vec::new());
    }
    
    if let Some(saved_queries) = &mut project.saved_queries {
        if let Some(last_query) = saved_queries.last_mut() {
            // Update the last query with the analysis chat history
            last_query["analysis_chat_history"] = serde_json::to_value(&data.history).unwrap_or_default();
        }
    }
    
    // Save the updated project settings
    if let Err(e) = project_service.save_project(&project, &project_dir) {
        return HttpResponse::InternalServerError().body(format!("Failed to save project: {}", e));
    }
    
    HttpResponse::Ok().body("Chat history saved successfully")
}
