pub mod architect;
pub mod search_handler;
pub mod file_context;
pub mod search_results_handler;

use std::collections::{HashMap, HashSet};
use std::path::Path;
use actix_web::web;
use serde_json::Value;

use crate::models::{ChatMessage, Project, AppState, SearchResult}; // Import SearchResult
use crate::services::llm_service::{LlmService, LlmServiceConfig};
use crate::services::project_service::query_management::QueryManager;
use crate::services::search_service::SearchService;
use crate::services::yaml::YamlService;
use crate::services::utils::html_utils::unescape_html;
use crate::services::agent::architect; // Import the architect module
use crate::services::agent::search_handler; // Import the search_handler module
use crate::services::agent::file_context; // Import the file_context module
use crate::services::agent::search_results_handler; // Import the search_results_handler module
use crate::services::path_utils::PathUtils;


pub async fn handle_agentic_message(
    project: &Project,
    app_state: &web::Data<AppState>,
    query_id: &str,
    user_message_content_raw: &str,
    enable_grounding: bool,
    include_file_descriptions: bool,
    previous_history_for_llm: &Vec<ChatMessage>,
    commit_hash_for_user_message: Option<String>,
    hidden_context: Vec<String>,
) -> Result<ChatMessage, String> {
    let query_manager = QueryManager::new();
    let yaml_service = YamlService::new();
    let llm_service = LlmService::new();
    let search_service = SearchService::new();
    let project_dir = Path::new(&app_state.output_dir).join(&project.name);

    let mut thoughts: Vec<String> = Vec::new();

    // These two maps are the core state of what the agent "knows" about files
    let mut file_contents_map: HashMap<String, String> = HashMap::new(); // path -> full content
    let mut initial_proactive_yaml_summaries: HashMap<String, String> = HashMap::new(); // path -> YAML description (before filtering/selection)

    thoughts.push("Agentic mode is enabled. Starting agent decision process.".to_string());

    let initial_query = query_manager
        .get_query_data_field(&project_dir, query_id, "query")
        .unwrap_or_else(|| "No previous query found".to_string());

    let mut llm_config = LlmServiceConfig::new();
    if enable_grounding {
        llm_config = llm_config.with_grounding_with_search(true);
        thoughts.push("Grounding with Google Search is enabled for this LLM call.".to_string());
    }

    // --- PHASE 1: Hybrid Search for initial context (Vector + BM25F) ---

    // 1. Generate Contextual Query for Vector Search
    thoughts.push("Generating contextual query for hybrid search.".to_string());
    let contextual_vector_query_llm_input = search_handler::generate_contextual_vector_query(
        &initial_query,
        previous_history_for_llm,
        user_message_content_raw,
    );

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
        .map_err(|e| format!("Failed to parse LLM analysis JSON2: {}\nFailed data: {}\nSource: {}", e, llm_analysis_json_str, llm_analysis_unescaped))?;

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
    // --- and Populate file_contents_map & initial_proactive_yaml_summaries ---
    thoughts.push("Consolidating search results and populating file context.".to_string());
    let all_relevant_file_paths = search_results_handler::consolidate_search_results(
        project,
        &vector_search_results,
        &bm25f_results,
        &suggested_vector_files,
        &mut thoughts,
    );
    thoughts.push(format!("Total unique relevant files identified across all searches: {}.", all_relevant_file_paths.len()));

    let (file_contents_map, initial_proactive_yaml_summaries) = search_results_handler::populate_file_context(
        project,
        &all_relevant_file_paths,
        &suggested_vector_files,
        &project_dir,
        &mut thoughts,
    ).await;
    thoughts.push(format!("Currently have {} files with full source and {} YAML summaries.", file_contents_map.len(), initial_proactive_yaml_summaries.len()));

    // --- PHASE 2: Architect Decision Loop (Simplified to one turn for now, to be extended) ---
    // ... (Implementation will be moved here) ...

    // --- PHASE 3: Final LLM Generation ---
    // ... (Implementation will be moved here) ...

    // Create the model's ChatMessage including thoughts
    let model_chat_message = ChatMessage {
        role: "model".to_string(),
        content: "LLM Response".to_string(), // Placeholder
        hidden: false,
        commit_hash: commit_hash_for_user_message.clone(),
        timestamp: Some(chrono::Utc::now()),
        context_files: Some(vec![]), // Placeholder
        provider: Some(project.provider.clone()),
        model: project.specific_model.clone(),
        hidden_context: Some(hidden_context.clone()),
        thoughts: Some(thoughts),
        ..Default::default()
    };

    Ok(model_chat_message)
}