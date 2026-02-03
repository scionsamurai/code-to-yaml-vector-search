// src/services/agent/search_handler.rs

use crate::models::{ChatMessage, Project};
use crate::services::llm_service::{LlmService, LlmServiceConfig};
use crate::services::search_service::{SearchService, SearchResult};
use crate::services::yaml::YamlService;
use crate::services::utils::html_utils::unescape_html;
use std::collections::HashSet;
use serde_json::Value;
use std::path::Path;

/// Generates a comprehensive query string for semantic (vector) search,
/// combining the initial query, relevant chat history, and the latest user message.
pub fn generate_contextual_vector_query(
    initial_query: &str,
    previous_history_for_llm: &Vec<ChatMessage>, // Use this for history
    user_message_content: &str,
) -> String {
    let mut query = String::new();
    query.push_str("Consider the following initial task:\n");
    query.push_str(initial_query);

    // Take the last few (e.g., 6) messages from history for conciseness and context
    // Ensure system/model intro messages are not included, only user/model turns
    let mut relevant_history_for_search: Vec<&ChatMessage> = previous_history_for_llm.iter()
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

/// Parses the raw LLM analysis response to extract suggested files and BM25 keywords.
fn parse_llm_search_analysis(
    llm_analysis_raw: String,
) -> Result<(HashSet<String>, String), String> {
    let llm_analysis_unescaped = unescape_html(llm_analysis_raw);
    let llm_analysis_json_str = llm_analysis_unescaped
        .split("```json")
        .nth(1)
        .and_then(|s| s.split("```").next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "".to_string());

    let llm_analysis_json: Value = serde_json::from_str(&llm_analysis_json_str)
        .map_err(|e| format!("Failed to parse LLM analysis JSON: {}\nFailed data: {}\nSource: {}", e, llm_analysis_json_str, llm_analysis_unescaped))?;
    
    let suggested_vector_files: HashSet<String> = llm_analysis_json["suggested_files"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    let bm25_keywords_str = llm_analysis_json["bm25_keywords"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok((suggested_vector_files, bm25_keywords_str))
}

/// Performs the initial hybrid search (vector + BM25F) to gather relevant file context.
#[allow(clippy::too_many_arguments)]
pub async fn perform_initial_hybrid_search(
    llm_service: &LlmService,
    search_service: &SearchService,
    yaml_service: &YamlService,
    project: &Project,
    initial_query: &str,
    user_message_content_raw: &str,
    previous_history_for_llm: &Vec<ChatMessage>,
    llm_config: &LlmServiceConfig,
    project_dir: &Path,
    thoughts: &mut Vec<String>,
) -> Result<(Vec<SearchResult>, Vec<(String, f32)>, HashSet<String>, String), String> {
    // 1. Generate Contextual Query for Vector Search
    thoughts.push("Generating contextual query for hybrid search.".to_string());
    let contextual_vector_query_llm_input = generate_contextual_vector_query(
        initial_query,
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
        &mut project.clone(),
        &detailed_vector_query,
        None,
        num_vector_results,
        true, // Enable LLM analysis for this internal call (suggested files & BM25 keywords)
    ).await?;
    thoughts.push(format!("Vector search returned {} results.", vector_search_results.len()));

    // Parse LLM analysis for suggested files and BM25 keywords
    let (suggested_vector_files, bm25_keywords_str) = parse_llm_search_analysis(llm_analysis_raw)?;
    thoughts.push(format!("LLM Analysis suggested {} files for full source from vector search.", suggested_vector_files.len()));
    thoughts.push(format!("LLM Analysis provided BM25 keywords: '{}'", bm25_keywords_str));

    // 3. Secondary BM25F Search (using keywords from LLM Analysis)
    thoughts.push("Performing secondary BM25F search over YAML summaries using LLM-generated keywords.".to_string());
    let num_bm25_results = 25; // Get top 25 BM25F results
    let bm25f_results = yaml_service.bm25f_search(
        project,
        &bm25_keywords_str,
        project_dir,
        num_bm25_results,
    ).await?;
    thoughts.push(format!("BM25F search returned {} results.", bm25f_results.len()));

    Ok((vector_search_results, bm25f_results, suggested_vector_files, bm25_keywords_str))
}