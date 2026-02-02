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
use crate::services::path_utils::PathUtils;
use std::collections::{HashMap, HashSet};
use serde_json::Value; // To parse Architect LLM's JSON response
use crate::services::utils::html_utils::unescape_html;

pub struct AgentService;

impl AgentService {
    // Helper function to create a concise search query for the embedding model
    /// Generates a comprehensive query string for semantic (vector) search,
    /// combining the initial query, relevant chat history, and the latest user message.
    fn generate_contextual_vector_query(
        initial_query: &str,
        unescaped_history: &Vec<ChatMessage>,
        user_message_content: &str,
    ) -> String {
        let mut query = String::new();
        query.push_str("Consider the following initial task:\n");
        query.push_str(initial_query);

        // Take the last few (e.g., 6) messages from history for conciseness and context
        // Ensure system/model intro messages are not included, only user/model turns
        let mut relevant_history_for_search: Vec<&ChatMessage> = unescaped_history.iter()
            .rev()
            .filter(|msg| msg.role == "user" || msg.role == "model")
            .take(6) // Adjust number of messages as needed
            .collect();
        relevant_history_for_search.reverse();

        if !relevant_history_for_search.is_empty() {
            query.push_str("\n\nReview the recent conversation history to understand the current context and goal:\n");
            for msg in relevant_history_for_search {
                query.push_str(&format!("{}: {}\n", msg.role, msg.content));
            }
        }
        
        query.push_str("\n\nBased on this, and the user's latest message:\n");
        query.push_str(user_message_content);
        query.push_str("\n\nWhat are the most relevant details for addressing the current request and conversation state? Provide a comprehensive and focused query for a semantic search. Your output will fill in the details missing from the latest message (since what you generate here will be prepended to the latest message before sending to the vector search).");
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
        let search_service = SearchService::new();
        let project_dir = Path::new(&app_state.output_dir).join(&project.name);
        
        let mut thoughts: Vec<String> = Vec::new();

        // These two maps are the core state of what the agent "knows" about files
        let mut file_contents_map: HashMap<String, String> = HashMap::new(); // path -> full content
        let mut initial_proactive_yaml_summaries: HashMap<String, String> = HashMap::new(); // path -> YAML description (before filtering/selection)

        thoughts.push("Agentic mode is enabled. Starting agent decision process.".to_string());

        let initial_query = query_manager
            .get_query_data_field(&project_dir, &query_id, "query")
            .unwrap_or_else(|| "No previous query found".to_string());
        
        let mut llm_config = LlmServiceConfig::new();
        if enable_grounding {
            llm_config = llm_config.with_grounding_with_search(true);
            thoughts.push("Grounding with Google Search is enabled for this LLM call.".to_string());
        }

        // --- PHASE 1: Hybrid Search for initial context (Vector + BM25F) ---

        // 1. Generate Contextual Query for Vector Search
        thoughts.push("Generating contextual query for hybrid search.".to_string());
        let contextual_vector_query_llm_input = Self::generate_contextual_vector_query(&initial_query, unescaped_history, user_message_content);
        
        // Use LLM to generate a more detailed query for vector search, potentially based on history and current message
        let detailed_vector_query = llm_service.get_analysis(&contextual_vector_query_llm_input, &project.provider.clone(), project.specific_model.as_deref(), Some(llm_config.clone())).await;
        thoughts.push(format!("Detailed Vector Query from LLM: '{}'", detailed_vector_query));

        // 2. Primary Vector Search
        thoughts.push("Performing primary vector search over project embeddings.".to_string());
        let num_vector_results = 5; // Get top 5 vector results
        let (vector_search_results, llm_analysis_raw) = search_service.search_project(
            &mut project.clone(), // Pass a clone to avoid mutable borrow issues and keep project state clean
            &detailed_vector_query, // Use the LLM-generated detailed query
            None, // No need to save new query data for agent's internal search
            num_vector_results,
            true, // Enable LLM analysis for this internal call (suggested files & BM25 keywords)
        ).await?;
        thoughts.push(format!("Vector search returned {} results.", vector_search_results.len()));

        // Parse LLM analysis for suggested files and BM25 keywords
        let llm_analysis_unescaped = unescape_html(llm_analysis_raw);
        let llm_analysis_json_str = llm_analysis_unescaped
            .split("```json")
            .nth(1)
            .and_then(|s| s.split("```").next())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "".to_string());
        let llm_analysis_json: Value = serde_json::from_str(&llm_analysis_json_str)
            .map_err(|e| format!("Failed to parse LLM analysis JSON: {}", e))?;
        
        let suggested_vector_files: HashSet<String> = llm_analysis_json["suggested_files"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        thoughts.push(format!("LLM Analysis suggested {} files for full source from vector search.", suggested_vector_files.len()));

        let bm25_keywords_str = llm_analysis_json["bm25_keywords"]
            .as_str()
            .unwrap_or("");
        thoughts.push(format!("LLM Analysis provided BM25 keywords: '{}'", bm25_keywords_str));

        // 3. Secondary BM25F Search (using keywords from LLM Analysis)
        thoughts.push("Performing secondary BM25F search over YAML summaries using LLM-generated keywords.".to_string());
        let num_bm25_results = 25; // Get top 25 BM25F results
        let bm25f_results = yaml_service.bm25f_search(
            project,
            bm25_keywords_str,
            &project_dir,
            num_bm25_results,
        ).await?;
        thoughts.push(format!("BM25F search returned {} results.", bm25f_results.len()));


        // --- Consolidate all potential relevant file paths into a single set ---
        let mut all_relevant_file_paths: HashSet<String> = HashSet::new();

        // Add paths from suggested_vector_files (most critical)
        for file_path in &suggested_vector_files {
            if let Some(normalized_path) = PathUtils::normalize_project_path(file_path, project) {
                all_relevant_file_paths.insert(normalized_path);
            } else {
                thoughts.push(format!("Could not normalize suggested path from vector search: {}", file_path));
            }
        }

        // Add paths from vector_search_results (the raw list from vector DB)
        for search_result in &vector_search_results {
            if let Some(normalized_path) = PathUtils::normalize_project_path(&search_result.file_path, project) {
                all_relevant_file_paths.insert(normalized_path);
            } else {
                thoughts.push(format!("Could not normalize path from raw vector search results: {}", search_result.file_path));
            }
        }

        // Add paths from bm25f_results
        for (file_path, _) in &bm25f_results {
            // split on project_name/ and get the second item of the split
            let yaml_target_path = file_path.split(&format!("{}/", project.name))
                .nth(1);
            let final_yaml_target_path = match yaml_target_path {
                Some(p) => {
                    p.replace("*", "/").replace(".yml", "") // Reverse the earlier replacement to get original path
                },
                None => {
                    thoughts.push(format!("Could not extract target path from YAML hit: {}", file_path));
                    continue; // Skip this hit
                },
            };
            all_relevant_file_paths.insert(final_yaml_target_path.clone());
        }
        thoughts.push(format!("Total unique relevant files identified across all searches: {}.", all_relevant_file_paths.len()));

        // --- Populate file_contents_map (Full Source) & initial_proactive_yaml_summaries (YAML descriptions) ---

        let convert_path_to_yaml_location = |path: &str| {
            project_dir.join(format!("{}.yml", path.replace("/", "*")))
        };

        println!("All relevant file paths: {:?}", all_relevant_file_paths);
        // Priority 1: Fetch full source for suggested_vector_files
        for file_path in &all_relevant_file_paths {
            if suggested_vector_files.contains(file_path) { // Prioritize files suggested by LLM analysis for full source
                if let Some(content) = file_service.read_specific_file(project, file_path) {
                    file_contents_map.insert(file_path.clone(), content);
                    thoughts.push(format!("Added full source (from LLM vector suggestion): {}", file_path));
                } else {
                    thoughts.push(format!("Failed to read suggested file for full source: {}. Trying YAML description.", file_path));
                    // If source cannot be read, fall back to YAML description
                    let yaml_loc = convert_path_to_yaml_location(file_path);
                    if let Ok(yaml_data) = yaml_service.management.get_parsed_yaml_for_file_sync(project, &yaml_loc.to_string_lossy(), &project_dir) {
                        initial_proactive_yaml_summaries.insert(file_path.clone(), yaml_data.description);
                        thoughts.push(format!("Added YAML description (suggested source failed): {}", file_path));
                    } else {
                        thoughts.push(format!("Failed to get YAML description for {}: {}", file_path, yaml_loc.display()));
                    }
                }
            }
        }

        // Priority 2: Get YAML descriptions for other relevant files (if not already full source)
        for file_path in &all_relevant_file_paths {
            if !file_contents_map.contains_key(file_path) { // Only process if we don't have full source yet
                let yaml_loc = convert_path_to_yaml_location(file_path);
                if let Ok(yaml_data) = yaml_service.management.get_parsed_yaml_for_file_sync(project, &yaml_loc.to_string_lossy(), &project_dir) {
                    initial_proactive_yaml_summaries.insert(file_path.clone(), yaml_data.description);
                } else {
                    thoughts.push(format!("Failed to get YAML description for {}: {}", file_path, yaml_loc.display()));
                }
            }
        }

        thoughts.push(format!("Currently have {} files with full source and {} YAML summaries.", file_contents_map.len(), initial_proactive_yaml_summaries.len()));

        // --- PHASE 2: Architect Decision Loop (Simplified to one turn for now, to be extended) ---

        // Prepare context for the Architect LLM:
        // - active_code: files in file_contents_map
        // - current_yaml_maps: YAML descriptions *only* for files NOT in file_contents_map, and limited in number.
        let mut proactive_file_descriptions_for_architect_prompt: HashMap<String, String> = HashMap::new();
        let max_proactive_descriptions_for_architect = 30; // Tune this number

        let mut sorted_initial_yamls: Vec<(&String, &String)> = initial_proactive_yaml_summaries.iter().collect();
        // Sort for stable selection if total exceeds max
        sorted_initial_yamls.sort_by_key(|(path, _)| path.clone());

        for (path, desc) in sorted_initial_yamls.into_iter() {
            if !file_contents_map.contains_key(path) && proactive_file_descriptions_for_architect_prompt.len() < max_proactive_descriptions_for_architect {
                proactive_file_descriptions_for_architect_prompt.insert(path.clone(), desc.clone());
            }
        }
        thoughts.push(format!("Refined proactive descriptions for Architect to {} files (not already full source).", proactive_file_descriptions_for_architect_prompt.len()));


        thoughts.push("Requesting Architect LLM decision on next steps.".to_string());
        let architect_decision = Self::get_architect_decision(
            &llm_service,
            project,
            &initial_query,
            user_message_content,
            &proactive_file_descriptions_for_architect_prompt, // Pass filtered YAML summaries
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
                        if !file_contents_map.contains_key(&normalized_path) {
                            if let Some(content) = file_service.read_specific_file(project, &normalized_path) {
                                file_contents_map.insert(normalized_path.clone(), content);
                                thoughts.push(format!("Fetched full source for: {}", normalized_path));
                                // Remove from initial_proactive_yaml_summaries if it was there, as we now have full source
                                initial_proactive_yaml_summaries.remove(&normalized_path);
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
                    project,
                    new_keywords,
                    &project_dir,
                    num_refined_yaml_results,
                ).await?;
                thoughts.push(format!("Refined BM25F search returned: {:?}", refined_yaml_hits));

                for (path, _) in &refined_yaml_hits {
                    // split on project_name/ and get the second item of the split
                    let yaml_target_path = path.split(&format!("{}/", project.name))
                        .nth(1);
                    let final_yaml_target_path = match yaml_target_path {
                        Some(p) => {
                            p.replace("*", "/").replace(".yml", "") // Reverse the earlier replacement to get original path
                        },
                        None => {
                            thoughts.push(format!("Could not extract target path from YAML hit: {}", path));
                            continue; // Skip this hit
                        },
                    };
                    // Add new YAML hits to our pool of initial_proactive_yaml_summaries if not already full source
                    if !file_contents_map.contains_key(final_yaml_target_path.as_str()) && !initial_proactive_yaml_summaries.contains_key(&final_yaml_target_path) {
                        match yaml_service.management.get_parsed_yaml_for_file_sync(project, &path, &project_dir) {
                            Ok(yaml_data) => {
                                initial_proactive_yaml_summaries.insert(final_yaml_target_path.clone(), yaml_data.description);
                                thoughts.push(format!("Added new YAML summary from refined search: {}", final_yaml_target_path));
                            }
                            Err(err) => {
                                thoughts.push(format!("Failed to parse YAML for refined BM25F result: {}. Error: {}", final_yaml_target_path, err));
                            }
                        }
                    }
                }
            },
            "GENERATE" => {
                thoughts.push("Architect decided to GENERATE directly.".to_string());
            },
            _ => {
                thoughts.push(format!("Architect returned an unknown action: {}. Proceeding with current context.", action));
            }
        }

        // --- PHASE 3: Final LLM Generation ---
        
        // Populate context_files (for full content) - ensure uniqueness and consistent ordering
        let mut context_files: Vec<String> = file_contents_map.keys().cloned().collect();
        context_files.sort(); // Consistent order
        thoughts.push(format!("Final list of files with full source content: {}.", context_files.len()));


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
        // Only include YAMLs for files that are *not* included as full source.
        let mut combined_project_file_descriptions = project.file_descriptions.clone();
        let max_proactive_descriptions_for_final_llm = 10; // Tune this number

        let mut sorted_final_yamls: Vec<(&String, &String)> = initial_proactive_yaml_summaries.iter().collect();
        sorted_final_yamls.sort_by_key(|(path, _)| path.clone()); // Sort for stable selection

        let mut final_proactive_descriptions_for_llm_map: HashMap<String, String> = HashMap::new();
        for (path, desc) in sorted_final_yamls.into_iter() {
            if !file_contents_map.contains_key(path) && final_proactive_descriptions_for_llm_map.len() < max_proactive_descriptions_for_final_llm {
                final_proactive_descriptions_for_llm_map.insert(path.clone(), desc.clone());
            }
        }

        for (path, desc) in final_proactive_descriptions_for_llm_map {
            combined_project_file_descriptions.insert(path, desc);
        }
        thoughts.push(format!("Combined project descriptions with {} filtered proactive descriptions for final LLM.", combined_project_file_descriptions.len()));


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
        let messages = format_messages_for_llm(&system_prompt, unescaped_history, &user_message_for_llm);
        thoughts.push("Messages formatted.".to_string());

        // Send to LLM
        thoughts.push("Sending final conversation to LLM for response generation.".to_string());

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