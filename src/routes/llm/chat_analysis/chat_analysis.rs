// src/routes/llm/chat_analysis/chat_analysis.rs
use super::models::*;
use super::utils::*;
use crate::models::AppState;
use crate::models::ChatMessage;
use crate::services::llm_service::LlmService;
use crate::services::project_service::ProjectService;
use actix_web::{post, web, HttpResponse};
use std::path::Path;
use crate::services::utils::html_utils::{escape_html, unescape_html};

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

    let project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to load project: {}", e))
        }
    };

    let query_id = data.query_id.as_deref().unwrap();

    let include_file_descriptions = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "include_file_descriptions").unwrap_or_else(|| "false".to_string()) == "true";

    // Get selected context files and file contents
    let (context_files, file_contents) = get_context_and_contents(&project, &app_state, &query_id);

    let query_text = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "query").unwrap_or_else(|| "No previous query found".to_string());

    // Create context prompt with the loaded file contents, project, and description flag
    let system_prompt = create_system_prompt(&query_text, &context_files, &file_contents, &project, include_file_descriptions); // Update this line

    // Get existing chat history from project structure
    let full_history = get_full_history(&project, &app_state, &query_id);

    let user_message = ChatMessage {
        role: "user".to_string(),
        content: data.message.clone(),
        hidden: false,
    };

    let mut unescaped_history: Vec<ChatMessage> = Vec::new();
    for message in full_history.iter() {
        let unescaped_content = unescape_html(message.content.clone());
        unescaped_history.push(ChatMessage {
            role: message.role.clone(),
            content: unescaped_content,
            hidden: message.hidden,
        });
    }

    // Format messages for LLM with system prompt and existing history
    let messages = format_messages_for_llm(&system_prompt, &unescaped_history, &user_message);

    // Send to LLM
    let llm_response = llm_service
        .send_conversation(&messages, &project.provider.clone(), project.specific_model.as_deref())
        .await;

    // Escape the user's message
    let escaped_message = escape_html(data.message.clone()).await;

    let user_message_to_save = ChatMessage { // Renamed to avoid shadowing
        role: "user".to_string(),
        content: escaped_message.to_string(),
        hidden: false,
    };

    // Create response message
    let assistant_message = ChatMessage {
        role: "model".to_string(),
        content: llm_response.clone(),
        hidden: false,
    };
    
    // Add messages to chat
    project_service.chat_manager
        .add_chat_message(&project_service.query_manager,&project_dir, user_message_to_save, query_id)
        .unwrap();
    project_service.chat_manager
        .add_chat_message(&project_service.query_manager,&project_dir, assistant_message, query_id)
        .unwrap();

    HttpResponse::Ok().body(llm_response)
}
