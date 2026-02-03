// src/services/agent/response_generation.rs

use std::collections::HashMap;
use std::path::Path;

use crate::models::{ChatMessage, Project};
use crate::services::llm_service::{LlmService, LlmServiceConfig};
use crate::services::project_service::query_management::QueryManager;
use crate::routes::llm::chat_analysis::utils::{create_system_prompt, format_messages_for_llm}; // Import necessary utilities

/// Orchestrates the final LLM generation, including context preparation,
/// prompt construction, and sending the conversation to the LLM.
#[allow(clippy::too_many_arguments)] // This function intentionally takes many arguments to manage generation state
pub async fn generate_llm_response(
    llm_service: &LlmService,
    project: &Project,
    query_id: &str,
    user_message_content_raw: &str,
    llm_config: LlmServiceConfig, // Pass the already configured LLM config
    include_file_descriptions: bool,
    previous_history_for_llm: &Vec<ChatMessage>,
    commit_hash_for_user_message: Option<String>,
    hidden_context: Vec<String>,
    current_file_contents_map: &HashMap<String, String>,
    current_initial_proactive_yaml_summaries: &HashMap<String, String>,
    project_dir: &Path,
    thoughts: &mut Vec<String>,
) -> Result<ChatMessage, String> {
    // Populate context_files (for full content) - ensure uniqueness and consistent ordering
    let mut context_files: Vec<String> = current_file_contents_map.keys().cloned().collect();
    context_files.sort(); // Consistent order
    thoughts.push(format!("Final list of files with full source content: {}.", context_files.len()));

    // Construct file contents string for the main LLM prompt
    let mut file_contents_for_llm = String::new();
    if !context_files.is_empty() {
        thoughts.push("Constructing file content string for the main LLM prompt.".to_string());
        for file_path in &context_files {
            if let Some(content) = current_file_contents_map.get(file_path) {
                file_contents_for_llm.push_str(&format!(
                    "--- FILE: {} ---\n{}\n\n",
                    file_path, content
                ));
            }
        }
    } else {
        thoughts.push("No files selected for full content.".to_string());
    }

    // Integrate proactively fetched descriptions into a clone of project's descriptions
    // Only include YAMLs for files that are *not* included as full source.
    let mut combined_project_file_descriptions = project.file_descriptions.clone();
    let max_proactive_descriptions_for_final_llm = 10; // Tune this number

    let mut sorted_final_yamls: Vec<(&String, &String)> = current_initial_proactive_yaml_summaries.iter().collect();
    sorted_final_yamls.sort_by_key(|(path, _)| path.as_str()); // Sort for stable selection

    let mut final_proactive_descriptions_for_llm_map: HashMap<String, String> = HashMap::new();
    for (path, desc) in sorted_final_yamls.into_iter() {
        if !current_file_contents_map.contains_key(path) && final_proactive_descriptions_for_llm_map.len() < max_proactive_descriptions_for_final_llm {
            final_proactive_descriptions_for_llm_map.insert(path.clone(), desc.clone());
        }
    }

    for (path, desc) in final_proactive_descriptions_for_llm_map.clone() {
        combined_project_file_descriptions.insert(path, desc);
    }
    thoughts.push(format!("Combined project descriptions with {} filtered proactive descriptions for final LLM.", final_proactive_descriptions_for_llm_map.len()));

    // Create system prompt
    thoughts.push("Creating system prompt based on initial query and selected context.".to_string());
    let query_manager = QueryManager::new(); // Local instance needed here
    let query_text = query_manager
        .get_query_data_field(project_dir, query_id, "query")
        .unwrap_or_else(|| "No previous query found".to_string());
    let system_prompt = create_system_prompt(
        &query_text,
        &context_files, // These are the full content files
        &file_contents_for_llm, // Full file contents
        &Project { file_descriptions: combined_project_file_descriptions, ..project.clone() }, // Pass a clone of project with augmented descriptions
        include_file_descriptions,
    );
    thoughts.push("System prompt created.".to_string());

    // Create the current user message (the one the LLM is responding to)
    let current_user_message_for_llm = ChatMessage {
        role: "user".to_string(),
        content: user_message_content_raw.to_string(),
        hidden: false,
        commit_hash: commit_hash_for_user_message.clone(),
        timestamp: Some(chrono::Utc::now()),
        context_files: Some(context_files.clone()),
        provider: Some(project.provider.clone()),
        model: project.specific_model.clone(),
        hidden_context: Some(hidden_context.clone()),
        thoughts: None, // Thoughts are for the model's message
        ..Default::default()
    };

    // Construct the full conversational messages for the LLM.
    // This includes the previous history AND the current user message.
    let conversational_messages: Vec<ChatMessage> = previous_history_for_llm.clone(); // Clone the history *before* this user message

    // Format messages for LLM
    thoughts.push("Formatting messages for the LLM conversation.".to_string());
    let messages = format_messages_for_llm(&system_prompt, &conversational_messages, &current_user_message_for_llm);
    thoughts.push("Messages formatted.".to_string());

    // Send to LLM
    thoughts.push("Sending final conversation to LLM for response generation.".to_string());

    let llm_response_content = llm_service
        .send_conversation(
            &messages,
            &project.provider.clone(),
            project.specific_model.as_deref(),
            Some(llm_config), // Use the passed-in llm_config
        )
        .await;
    thoughts.push("Received final response from LLM.".to_string());

    // Create the model's ChatMessage including thoughts
    let model_chat_message = ChatMessage {
        role: "model".to_string(),
        content: llm_response_content,
        hidden: false,
        commit_hash: commit_hash_for_user_message.clone(),
        timestamp: Some(chrono::Utc::now()),
        context_files: Some(context_files.clone()), // The files whose full content was sent
        provider: Some(project.provider.clone()),
        model: project.specific_model.clone(),
        hidden_context: Some(hidden_context.clone()),
        thoughts: Some(thoughts.clone()), // ATTACH ALL COLLECTED THOUGHTS HERE
        ..Default::default()
    };

    Ok(model_chat_message)
}