// src/services/agent_service.rs

use crate::models::{ChatMessage, Project};
use crate::services::llm_service::{LlmService, LlmServiceConfig};
use crate::services::project_service::query_management::QueryManager; // Import QueryManager
use crate::routes::llm::chat_analysis::utils::{
    get_context_and_contents, create_system_prompt, format_messages_for_llm
};
use actix_web::web;
use std::path::Path;
use crate::models::AppState;

pub struct AgentService;

impl AgentService {
    pub async fn handle_agentic_message(
        project: &Project,
        app_state: &web::Data<AppState>, // Add app_state
        query_id: &str,
        user_message_content: &str,
        enable_grounding: bool,
        include_file_descriptions: bool,
        unescaped_history: &Vec<ChatMessage>,
        commit_hash_for_user_message: Option<String>,
        hidden_context: Vec<String>,
    ) -> Result<ChatMessage, String> {
        let mut thoughts: Vec<String> = Vec::new();
        let llm_service = LlmService::new();

        thoughts.push("User sent a message. Agentic mode is enabled.".to_string());

        // 1. Determine context (agent's choice instead of user's)
        thoughts.push("Determining relevant file context for the query.".to_string());
        // TODO: Implement agentic file selection logic here
        // For now, reuse existing context for simplicity, or even select *all* relevant files.
        let (context_files, file_contents) = get_context_and_contents(project, app_state, query_id);
        thoughts.push(format!("Selected {} context files.", context_files.len()));

        // 2. Create system prompt
        thoughts.push("Creating system prompt based on query and context.".to_string());
        let query_manager = QueryManager::new(); // Initialize QueryManager

        let output_dir = Path::new(&app_state.output_dir);
        let project_dir = output_dir.join(&project.name);

        let query_text = query_manager
            .get_query_data_field(&project_dir, &query_id, "query")
            .unwrap_or_else(|| "No previous query found".to_string());
        let system_prompt = create_system_prompt(&query_text, &context_files, &file_contents, &project, include_file_descriptions);
        

        // 3. Create user message
        let user_message_for_llm = ChatMessage {
            role: "user".to_string(),
            content: user_message_content.to_string(),
            hidden: false,
            commit_hash: commit_hash_for_user_message.clone(),
            timestamp: Some(chrono::Utc::now()),
            context_files: Some(context_files.clone()),
            provider: Some(project.provider.clone()),
            model: project.specific_model.clone(),
            hidden_context: Some(hidden_context.clone()),
            thoughts: None,
            ..Default::default()
        };

        // 4. Format messages for LLM
        thoughts.push("Formatting messages for the LLM conversation.".to_string());
        let messages = format_messages_for_llm(&system_prompt, &unescaped_history, &user_message_for_llm);
        

        // 5. Send to LLM
        thoughts.push("Sending conversation to LLM.".to_string());
        let mut llm_config = LlmServiceConfig::new();
        if enable_grounding {
            llm_config = llm_config.with_grounding_with_search(true);
        }
        let llm_response_content = llm_service
            .send_conversation(&messages, &project.provider.clone(), project.specific_model.as_deref(), Some(llm_config))
            .await;
        thoughts.push("Received response from LLM.".to_string());

        // 6. Create the model's ChatMessage including thoughts
        let model_chat_message = ChatMessage {
            role: "model".to_string(),
            content: llm_response_content,
            hidden: false,
            commit_hash: commit_hash_for_user_message.clone(),
            timestamp: Some(chrono::Utc::now()),
            context_files: Some(context_files.clone()),
            provider: Some(project.provider.clone()),
            model: project.specific_model.clone(),
            hidden_context: Some(hidden_context.clone()),
            thoughts: Some(thoughts), // ATTACH THOUGHTS HERE
            ..Default::default()
        };

        Ok(model_chat_message)
    }
}