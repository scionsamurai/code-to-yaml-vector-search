// src/services/agent/architect.rs
use std::collections::HashMap;
use serde_json::Value;
use crate::models::{ChatMessage, Project};
use crate::services::llm_service::{LlmService, LlmServiceConfig};
use crate::services::utils::html_utils::unescape_html;

/// Prompts the LLM to act as an "Architect" to decide the next best action.
/// This is a new, separate LLM call.
pub async fn get_architect_decision(
    llm_service: &LlmService,
    project: &Project,
    initial_query: &str,
    user_message_content: &str,
    current_yaml_maps: &HashMap<String, String>, // File path -> YAML description
    active_code_files: &HashMap<String, String>, // File path -> full content
) -> Result<Value, String> {
    let mut architect_prompt = String::new();
    architect_prompt.push_str("You are an AI architect assistant. Your goal is to decide the best next step to answer a user's request, given the available context.\n");
    architect_prompt.push_str("You have access to high-level YAML summaries of project files ('YAML Maps') and full source code of a few 'Active Code Files'.\n\n");
    architect_prompt.push_str(&format!("Initial User Query: \"{}\"\n", initial_query));
    architect_prompt.push_str(&format!("User's Latest Message: \"{}\"\n\n", user_message_content));

    if !current_yaml_maps.is_empty() {
        architect_prompt.push_str("--- YAML Maps (High-Level Descriptions) ---\n");
        for (path, description) in current_yaml_maps {
            architect_prompt.push_str(&format!("FILE: {}\nDESCRIPTION:\n{}\n\n", path, description));
        }
    }

    if !active_code_files.is_empty() {
        architect_prompt.push_str("--- Active Code Files (Full Source) ---\n");
        for (path, content) in active_code_files {
            architect_prompt.push_str(&format!("FILE: {}\nCONTENT:\n```\n{}\n```\n\n", path, content));
        }
    }

    architect_prompt.push_str("--- Your Decision ---\n");
    architect_prompt.push_str("Based on the above, decide your next action. You MUST respond with a JSON object. No conversational text.\n");
    architect_prompt.push_str("The JSON structure should be:\n");
    architect_prompt.push_str("{\n  \"action\": \"GENERATE\" | \"FETCH_SOURCE\" | \"SEARCH_MORE\",\n");
    architect_prompt.push_str("  \"reason\": \"A brief explanation for your decision.\",\n");
    architect_prompt.push_str("  \"paths\": [\"path/to/file1.rs\", \"path/to/file2.rs\"], // ONLY if action is FETCH_SOURCE\n");
    architect_prompt.push_str("  \"keywords\": \"space separated keywords for new search\" // ONLY if action is SEARCH_MORE\n}\n\n");

    architect_prompt.push_str("Available Actions:\n");
    architect_prompt.push_str("- `GENERATE`: You have sufficient context to directly answer the user's latest message.\n");
    architect_prompt.push_str("- `FETCH_SOURCE`: You need the full source code for specific files whose YAML summaries are available. Provide a list of relative file paths (e.g., 'src/models.rs') in the 'paths' array. Only select files whose YAML maps are available.\n");
    architect_prompt.push_str("- `SEARCH_MORE`: You need to broaden or refine the search for YAML maps. Provide a new set of space-separated keywords in the 'keywords' field.\n\n");
    architect_prompt.push_str("Remember, your response MUST be a single JSON object. Start with '{' and end with '}'.\n");

    let llm_config_option = Some(LlmServiceConfig::new()); // No grounding for architect decision

    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: architect_prompt,
            ..Default::default()
        },
    ];

    let llm_response_content = llm_service
        .send_conversation(
            &messages,
            &project.provider.clone(),
            project.specific_model.as_deref(),
            llm_config_option,
        )
        .await;

    // Try to parse the LLM's response as JSON
    let trimmed_response = llm_response_content.trim();
    // LLM might wrap JSON in markdown block. Try to extract it.
    let json_str = if trimmed_response.starts_with("```json") && trimmed_response.ends_with("```") {
        trimmed_response.strip_prefix("```json").unwrap().strip_suffix("```").unwrap().trim()
    } else if trimmed_response.starts_with("```") && trimmed_response.ends_with("```") {
         trimmed_response.strip_prefix("```").unwrap().strip_suffix("```").unwrap().trim()
    }
    else {
        trimmed_response
    };

    // unescape llm_response_content
    let unescaped_response = unescape_html(json_str.to_string());


    serde_json::from_str(&unescaped_response)
        .map_err(|e| format!("Failed to parse Architect LLM's JSON response: {} - Original response: {}", e, unescaped_response))
}