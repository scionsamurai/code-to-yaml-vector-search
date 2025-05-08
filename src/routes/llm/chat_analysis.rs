// src/routes/llm/chat_analysis.rs
use actix_web::{post, web, HttpResponse};
use crate::models::ChatMessage;
use serde::Deserialize;
use crate::models::AppState;
use crate::services::llm_service::LlmService;
use crate::services::project_service::ProjectService;
use crate::services::file_service::FileService;
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
    let file_service = FileService {};
    
    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project);
    
    let mut project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load project: {}", e)),
    };
    
    // Get selected context files from project settings
    let context_files = if let Some(saved_queries) = &project.saved_queries {
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
    
    // Load file contents for the selected files
    let file_contents = context_files.iter()
        .filter_map(|file_path| {
            if let Some(content) = file_service.read_specific_file(&project, file_path) {
                Some(format!("--- FILE: {} ---\n{}\n\n", file_path, content))
            } else {
                None
            }
        })
        .collect::<String>();
    
    // Create context prompt with the loaded file contents
    let system_prompt = format!(
        "You are an AI assistant helping with code analysis for a project. \
        The user's original query was: \"{}\"\n\n\
        You have access to the following files:\n{}\n\n\
        Here are the contents of these files:\n\n{}",
        data.query,
        context_files.join("\n"),
        file_contents
    );
    
    // Format messages for LLM
    let mut messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: system_prompt,
        },
        ChatMessage {
            role: "model".to_string(),
            content: "I understand.".to_string(),
        }
    ];
    
    // Add history messages 
    messages.extend(
        data.history.iter()
            .cloned()
    );
    
    // Add the current user message
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: data.message.clone(),
    });

    // println!("Sending messages to LLM: {:?}", messages);
    
    // Send to LLM
    let model = project.model.clone();
    let llm_response = llm_service.send_conversation(&messages, &model).await;
    
    // Add response to messages for history tracking (without file contents)
    let assistant_message = ChatMessage {
        role: "model".to_string(),
        content: llm_response.clone(),
    };
    
    
    // Add rest of the conversation
    let mut full_history = vec![];
    full_history.extend(
        data.history.iter()
            .cloned()
    );
    full_history.push(assistant_message);
    
    // Save the updated chat history to the project settings
    if project.saved_queries.is_none() {
        project.saved_queries = Some(Vec::new());
    }
    
    if let Some(saved_queries) = &mut project.saved_queries {
        if let Some(last_query) = saved_queries.last_mut() {
            // Update the last query with the analysis chat history
            last_query["analysis_chat_history"] = serde_json::to_value(&full_history).unwrap_or_default();
            
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

#[derive(Deserialize)]
pub struct UpdateChatMessageRequest {
    project: String,
    history: Vec<ChatMessage>,
}

#[post("/update-chat-message")]
pub async fn update_chat_message(
    app_state: web::Data<AppState>,
    data: web::Json<UpdateChatMessageRequest>,
) -> HttpResponse {
    let project_service = ProjectService::new();
    
    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project);
    
    let mut project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load project: {}", e)),
    };
    
    // Update the chat history with the edited message content
    if project.saved_queries.is_none() {
        project.saved_queries = Some(Vec::new());
    }
    
    if let Some(saved_queries) = &mut project.saved_queries {
        if let Some(last_query) = saved_queries.last_mut() {
            // Update the last query with the updated chat history
            last_query["analysis_chat_history"] = serde_json::to_value(&data.history).unwrap_or_default();
            
            // Save the updated project settings
            if let Err(e) = project_service.save_project(&project, &project_dir) {
                return HttpResponse::InternalServerError().body(format!("Failed to save project: {}", e));
            }
        }
    }
    
    HttpResponse::Ok().body("Message updated successfully")
}