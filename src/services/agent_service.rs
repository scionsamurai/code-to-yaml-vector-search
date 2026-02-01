// src/services/agent_service.rs
use crate::models::{ChatMessage, Project};
use crate::services::llm_service::{LlmService, LlmServiceConfig};
use crate::routes::llm::chat_analysis::utils::{
    create_system_prompt, format_messages_for_llm
};
use crate::services::project_service::query_management::QueryManager;
use std::path::Path;
use actix_web::web;
use crate::models::AppState;
use crate::services::search_service::{SearchService, SearchResult};
use crate::services::file::FileService;
use crate::services::yaml::{YamlService, FileYamlData};
use crate::services::path_utils::PathUtils; // NEW: Import PathUtils
use std::collections::{HashMap, HashSet};
use serde_json::Value; // To parse Architect LLM's JSON response
use crate::services::utils::html_utils::unescape_html;

pub struct AgentService;

impl AgentService {
    // Helper function to create a concise search query for the embedding model
    fn create_agentic_search_query(
        initial_query: &str,
        unescaped_history: &Vec<ChatMessage>,
        user_message_content: &str,
    ) -> String {
        let mut query = String::new();
        query.push_str("Consider the following initial task:\n");
        query.push_str(initial_query);
        query.push_str("\n\nReview the recent conversation history to understand the current context and goal:\n");

        // Take the last few (e.g., 4) messages from history for conciseness
        let recent_history_len = unescaped_history.len().min(4);
        for msg in unescaped_history.iter().rev().take(recent_history_len).rev() {
            query.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }
        
        query.push_str("\nBased on this, and the user's latest message:\n");
        query.push_str(user_message_content);
        query.push_str("\n\nWhat are the most relevant code files for addressing the current request and conversation state?");
        query
    }

    /// Prompts the LLM to act as an "Architect" to decide the next best action.
    /// This is a new, separate LLM call.
    async fn get_architect_decision(
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

    pub async fn handle_agentic_message(
        project: &Project,
        app_state: &web::Data<AppState>,
        query_id: &str,
        user_message_content: &str,
        enable_grounding: bool,
        include_file_descriptions: bool,
        unescaped_history: &Vec<ChatMessage>,
        commit_hash_for_user_message: Option<String>,
        hidden_context: Vec<String>,
    ) -> Result<ChatMessage, String> {
        let query_manager = QueryManager::new();
        let yaml_service = YamlService::new();
        let llm_service = LlmService::new();
        let file_service = FileService {};
        let project_dir = Path::new(&app_state.output_dir).join(&project.name);
        
        let mut thoughts: Vec<String> = Vec::new();
        let mut context_files: Vec<String> = Vec::new(); // Files for which full content is sent
        let mut file_contents_map: HashMap<String, String> = HashMap::new(); // Content for context_files
        let mut proactive_file_descriptions: HashMap<String, String> = HashMap::new(); // YAML summaries

        thoughts.push("Agentic mode is enabled. Starting agent decision process.".to_string());

        let initial_query = query_manager
            .get_query_data_field(&project_dir, &query_id, "query")
            .unwrap_or_else(|| "No previous query found".to_string());

        // --- PHASE 1: Initial YAML Search & Pre-fetch ---
        thoughts.push("Performing initial BM25F search over YAML summaries.".to_string());
        let num_initial_yaml_results = 15;
        let initial_yaml_hits = yaml_service.bm25f_search(
            &project,
            &Self::create_agentic_search_query(&initial_query, unescaped_history, user_message_content), // Use the specific agentic search query
            &project_dir,
            num_initial_yaml_results,
        ).await?;
        thoughts.push(format!("BM25F search returned {} YAML hits.", initial_yaml_hits.len()));

        let mut current_yaml_maps: HashMap<String, String> = HashMap::new(); // Store path -> description
        for (path, _) in &initial_yaml_hits {
            if let Ok(yaml_data) = yaml_service.management.get_parsed_yaml_for_file_sync(&project, path, &project_dir) {
                current_yaml_maps.insert(path.clone(), yaml_data.description);
            }
        }
        thoughts.push(format!("Collected {} YAML summaries.", current_yaml_maps.len()));

        
        // Pre-fetch TOP N source files (e.g., 2) based on initial YAML hits
        let num_prefetch_source_files = 2;
        thoughts.push(format!("Pre-fetching top {} source files from initial YAML search.", num_prefetch_source_files));
        let mut pre_fetched_paths: HashSet<String> = HashSet::new();

        for (file_path, _) in initial_yaml_hits.iter().take(num_prefetch_source_files) {
            if !pre_fetched_paths.contains(file_path) {
                if let Some(content) = file_service.read_specific_file(project, &file_path) {
                    file_contents_map.insert(file_path.clone(), content);
                    context_files.push(file_path.clone());
                    pre_fetched_paths.insert(file_path.clone());
                    thoughts.push(format!("Pre-fetched full source for: {}", file_path));
                }
            }
        }

        // Add remaining initial YAML hits (not pre-fetched) to proactive descriptions
        for (file_path, _) in &initial_yaml_hits {
            if !pre_fetched_paths.contains(file_path) {
                if let Ok(yaml_data) = yaml_service.management.get_parsed_yaml_for_file_sync(&project, &file_path, &project_dir) {
                    proactive_file_descriptions.insert(file_path.clone(), yaml_data.description);
                }
            }
        }
        thoughts.push(format!("Added {} YAML summaries to proactive descriptions (excluding pre-fetched).", proactive_file_descriptions.len()));

        // --- PHASE 2: Architect Decision Loop (Simplified to one turn for now) ---
        thoughts.push("Requesting Architect LLM decision on next steps.".to_string());
        let architect_decision = Self::get_architect_decision(
            &llm_service,
            project,
            &initial_query,
            user_message_content,
            &proactive_file_descriptions, // Pass YAML summaries for architect to review
            &file_contents_map, // Pass active code files
        ).await?;
        thoughts.push(format!("Architect decision: {:?}", architect_decision));

        let action = architect_decision["action"]
            .as_str()
            .ok_or_else(|| "Architect decision missing 'action' field or not a string.".to_string())?;
        
        let reason = architect_decision["reason"]
            .as_str()
            .unwrap_or("No reason provided.")
            .to_string();
        thoughts.push(format!("Architect Reason: {}", reason));

        match action {
            "FETCH_SOURCE" => {
                thoughts.push("Architect decided to FETCH_SOURCE.".to_string());
                let paths_to_fetch = architect_decision["paths"]
                    .as_array()
                    .ok_or_else(|| "FETCH_SOURCE action requires 'paths' array.".to_string())?;

                for path_val in paths_to_fetch {
                    let raw_path = path_val.as_str().ok_or_else(|| "Path in 'paths' array is not a string.".to_string())?;
                    if let Some(normalized_path) = PathUtils::normalize_project_path(raw_path, project) {
                        if !file_contents_map.contains_key(&normalized_path) { // Only fetch if not already in active code
                            if let Some(content) = file_service.read_specific_file(project, &normalized_path) {
                                file_contents_map.insert(normalized_path.clone(), content);
                                context_files.push(normalized_path.clone());
                                thoughts.push(format!("Fetched full source for: {}", normalized_path));
                                proactive_file_descriptions.remove(&normalized_path); // Remove from summaries if now full content
                            } else {
                                thoughts.push(format!("Failed to read file: {}", normalized_path));
                            }
                        } else {
                            thoughts.push(format!("File already in active context, skipping fetch: {}", normalized_path));
                        }
                    } else {
                        thoughts.push(format!("Could not normalize path for fetching: {}", raw_path));
                    }
                }
            },
            "SEARCH_MORE" => {
                thoughts.push("Architect decided to SEARCH_MORE.".to_string());
                let new_keywords = architect_decision["keywords"]
                    .as_str()
                    .ok_or_else(|| "SEARCH_MORE action requires 'keywords' string.".to_string())?;
                
                thoughts.push(format!("Performing refined BM25F search with keywords: {}", new_keywords));
                let num_refined_yaml_results = 10;
                let refined_yaml_hits = yaml_service.bm25f_search(
                    &project,
                    new_keywords,
                    &project_dir,
                    num_refined_yaml_results,
                ).await?;
                thoughts.push(format!("Refined BM25F search returned {} YAML hits.", refined_yaml_hits.len()));

                for (path, _) in &refined_yaml_hits {
                    if !file_contents_map.contains_key(path) { // Only add description if not already full content
                        if let Some(normalized_path) = PathUtils::normalize_project_path(path, project) {
                            if let Ok(yaml_data) = yaml_service.management.get_parsed_yaml_for_file_sync(&project, &normalized_path, &project_dir) {
                                proactive_file_descriptions.insert(normalized_path.clone(), yaml_data.description);
                            }
                        }
                    }
                }
                thoughts.push(format!("Added {} new YAML summaries from refined search.", refined_yaml_hits.len()));
            },
            "GENERATE" => {
                thoughts.push("Architect decided to GENERATE directly.".to_string());
                // No additional files to fetch or search, proceed with current context
            },
            _ => {
                thoughts.push(format!("Architect returned an unknown action: {}. Proceeding with current context.", action));
            }
        }

        // --- PHASE 3: Final LLM Generation ---
        thoughts.push("Proceeding to final LLM generation with gathered context.".to_string());
        
        // Populate context_files (for full content)
        context_files = file_contents_map.keys().cloned().collect();

        // Construct file contents string for the main LLM prompt
        let mut file_contents_for_llm = String::new();
        if !context_files.is_empty() {
            thoughts.push("Constructing file content string for the main LLM prompt.".to_string());
            for file_path in &context_files {
                if let Some(content) = file_contents_map.get(file_path) {
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
        let mut combined_project_file_descriptions = project.file_descriptions.clone();
        for (path, desc) in proactive_file_descriptions {
            combined_project_file_descriptions.insert(path, desc);
        }
        thoughts.push(format!("Combined project descriptions with {} proactive descriptions.", combined_project_file_descriptions.len()));


        // Create system prompt
        thoughts.push("Creating system prompt based on initial query and selected context.".to_string());
        let query_text = query_manager
            .get_query_data_field(&project_dir, &query_id, "query")
            .unwrap_or_else(|| "No previous query found".to_string());
        let system_prompt = create_system_prompt(
            &query_text,
            &context_files, // These are the full content files
            &file_contents_for_llm, // Full file contents
            &Project { file_descriptions: combined_project_file_descriptions, ..project.clone() }, // Pass a clone of project with augmented descriptions
            include_file_descriptions,
        );
        thoughts.push("System prompt created.".to_string());

        // Create user message
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
            thoughts: None, // Thoughts are for the model's message
            ..Default::default()
        };

        // Format messages for LLM
        thoughts.push("Formatting messages for the LLM conversation.".to_string());
        let messages = format_messages_for_llm(&system_prompt, &unescaped_history, &user_message_for_llm);
        thoughts.push("Messages formatted.".to_string());

        // Send to LLM
        thoughts.push("Sending final conversation to LLM for response generation.".to_string());
        let mut llm_config = LlmServiceConfig::new();
        if enable_grounding {
            llm_config = llm_config.with_grounding_with_search(true);
            thoughts.push("Grounding with Google Search is enabled for this LLM call.".to_string());
        }
        let llm_response_content = llm_service
            .send_conversation(
                &messages,
                &project.provider.clone(),
                project.specific_model.as_deref(),
                Some(llm_config),
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
            thoughts: Some(thoughts), // ATTACH ALL COLLECTED THOUGHTS HERE
            ..Default::default()
        };

        Ok(model_chat_message)
    }
}