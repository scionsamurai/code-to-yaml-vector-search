use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::services::path_utils::PathUtils;
use crate::models::{ChatMessage, Project};
use crate::services::llm_service::{LlmService, LlmServiceConfig};
use crate::services::search_service::SearchService;
use crate::services::yaml::YamlService;
use super::{architect_handler, file_context, search_handler, search_results_handler};

#[derive(Debug, Clone)]
pub enum AgentState {
    /// Initial state - need to perform hybrid search
    InitialSearch,
    /// Architect is making a decision
    ArchitectDecision,
    /// Architect decided to fetch specific source files
    FetchingSource(Vec<String>),
    /// Architect decided to search for more YAML summaries
    SearchingMore(String), // Keywords for search
    /// Ready to generate final response
    ReadyToGenerate,
    /// Terminal error state
    Error(String),
}

/// Holds all the mutable context that the agent needs across state transitions
pub struct AgentContext {
    /// Full source code of files
    pub file_contents_map: HashMap<String, String>,
    /// YAML summaries of files (for files not in file_contents_map)
    pub yaml_summaries: HashMap<String, String>,
    /// Accumulated thoughts/logs for debugging
    pub thoughts: Vec<String>,
    /// Number of architect decision turns taken
    pub turn_count: usize,
    /// Maximum allowed turns before forcing generation
    pub max_turns: usize,
}

impl AgentContext {
    pub fn new(max_turns: usize) -> Self {
        Self {
            file_contents_map: HashMap::new(),
            yaml_summaries: HashMap::new(),
            thoughts: Vec::new(),
            turn_count: 0,
            max_turns,
        }
    }

    /// Increments turn count and checks if max turns reached
    pub fn increment_turn(&mut self) -> bool {
        self.turn_count += 1;
        if self.turn_count >= self.max_turns {
            self.thoughts.push(format!(
                "Warning: Reached maximum architect turns ({}). Forcing generation.",
                self.max_turns
            ));
            true // max turns reached
        } else {
            false
        }
    }

    /// Adds a thought/log message
    pub fn add_thought(&mut self, thought: String) {
        self.thoughts.push(thought);
    }

    /// Merges file content into the context
    pub fn add_file_content(&mut self, path: String, content: String) {
        self.file_contents_map.insert(path.clone(), content);
        // Remove from YAML summaries if it was there
        self.yaml_summaries.remove(&path);
    }

    /// Adds a YAML summary (only if we don't already have full source)
    pub fn add_yaml_summary(&mut self, path: String, summary: String) {
        if !self.file_contents_map.contains_key(&path) {
            self.yaml_summaries.insert(path, summary);
        }
    }

    /// Get total number of files in context (full source + YAML)
    pub fn total_files(&self) -> usize {
        self.file_contents_map.len() + self.yaml_summaries.len()
    }
}


/// Handles the initial hybrid search state
pub async fn handle_initial_search(
    llm_service: &LlmService,
    search_service: &SearchService,
    yaml_service: &YamlService,
    project: &Project,
    initial_query: &str,
    user_message_content_raw: &str,
    previous_history_for_llm: &Vec<ChatMessage>,
    llm_config: &LlmServiceConfig,
    project_dir: &Path,
    context: &mut AgentContext,
) -> Result<AgentState, String> {
    context.add_thought("--- PHASE 1: Initial Hybrid Search ---".to_string());

    let (vector_search_results, bm25f_results, suggested_vector_files, _bm25_keywords_str) =
        search_handler::perform_initial_hybrid_search(
            llm_service,
            search_service,
            yaml_service,
            project,
            initial_query,
            user_message_content_raw,
            previous_history_for_llm,
            llm_config,
            project_dir,
            &mut context.thoughts,
        ).await?;

    // ensure paths in suggested_vector_files are not relative with normalize_project_path from path_utils
    let mut normalized_suggested_vector_files: HashSet<String> = HashSet::new();
    for raw_path in suggested_vector_files {
        if let Some(normalized_path) = PathUtils::normalize_project_path(&raw_path, project) {
            normalized_suggested_vector_files.insert(normalized_path);
        } else {
            context.add_thought(format!("Could not normalize suggested vector file path: {}", raw_path));
        }
    }

    context.add_thought("Consolidating search results and populating file context.".to_string());
    let all_relevant_file_paths = search_results_handler::consolidate_search_results(
        project,
        &vector_search_results,
        &bm25f_results,
        &normalized_suggested_vector_files,
        &mut context.thoughts,
    );
    context.add_thought(format!(
        "Total unique relevant files identified across all searches: {}.",
        all_relevant_file_paths.len()
    ));

    let (file_contents_map, yaml_summaries) = search_results_handler::populate_file_context(
        project,
        &all_relevant_file_paths,
        &normalized_suggested_vector_files,
        project_dir,
        &mut context.thoughts,
    ).await;

    // Populate context
    for (path, content) in file_contents_map {
        context.add_file_content(path, content);
    }
    for (path, summary) in yaml_summaries {
        context.add_yaml_summary(path, summary);
    }

    context.add_thought(format!(
        "Currently have {} files with full source and {} YAML summaries.",
        context.file_contents_map.len(),
        context.yaml_summaries.len()
    ));

    // Transition to architect decision
    Ok(AgentState::ArchitectDecision)
}

/// Handles an architect decision turn
pub async fn handle_architect_turn(
    llm_service: &LlmService,
    project: &Project,
    initial_query: &str,
    user_message_content_raw: &str,
    context: &mut AgentContext,
) -> Result<AgentState, String> {
    // Check if we've exceeded max turns
    if context.increment_turn() {
        return Ok(AgentState::ReadyToGenerate);
    }

    context.add_thought(format!("--- Architect Turn {} ---", context.turn_count));

    let next_state = architect_handler::handle_architect_decision_stateful(
        llm_service,
        project,
        initial_query,
        user_message_content_raw,
        context,
    ).await?;

    Ok(next_state)
}

/// Handles fetching source files
pub async fn handle_fetch_source(
    project: &Project,
    paths: Vec<String>,
    context: &mut AgentContext,
) -> Result<AgentState, String> {
    context.add_thought(format!("--- FETCH_SOURCE: {} files ---", paths.len()));

    let loaded_contents = file_context::load_file_contents(
        project,
        &paths,
        &mut context.thoughts,
    );

    for (path, content) in loaded_contents {
        context.add_file_content(path, content);
    }

    context.add_thought(format!(
        "After FETCH_SOURCE: {} files with full source, {} YAML summaries.",
        context.file_contents_map.len(),
        context.yaml_summaries.len()
    ));

    // After fetching, go back to architect to decide next step
    Ok(AgentState::ArchitectDecision)
}

/// Handles searching for more YAML summaries
pub async fn handle_search_more(
    yaml_service: &YamlService,
    project: &Project,
    project_dir: &Path,
    keywords: String,
    context: &mut AgentContext,
) -> Result<AgentState, String> {
    context.add_thought(format!("--- SEARCH_MORE: '{}' ---", keywords));

    let num_refined_yaml_results = 10;
    let refined_yaml_hits = yaml_service.bm25f_search(
        project,
        &keywords,
        project_dir,
        num_refined_yaml_results,
    ).await?;

    context.add_thought(format!("Refined BM25F search returned {} results.", refined_yaml_hits.len()));

    for (path, _) in &refined_yaml_hits {
        let yaml_target_path = path.split(&format!("{}/", project.name)).nth(1);
        let final_yaml_target_path = match yaml_target_path {
            Some(p) => p.replace("*", "/").replace(".yml", ""),
            None => {
                context.add_thought(format!("Could not extract target path from YAML hit: {}", path));
                continue;
            },
        };

        // Only add if we don't already have it
        if !context.file_contents_map.contains_key(&final_yaml_target_path)
            && !context.yaml_summaries.contains_key(&final_yaml_target_path)
        {
            match yaml_service.management.get_parsed_yaml_for_file_sync(project, &final_yaml_target_path, project_dir) {
                Ok(yaml_data) => {
                    context.add_yaml_summary(final_yaml_target_path.clone(), yaml_data.description);
                    context.add_thought(format!("Added new YAML summary from refined search: {}", final_yaml_target_path));
                }
                Err(err) => {
                    context.add_thought(format!(
                        "Failed to parse YAML for refined BM25F result: {}. Error: {}",
                        final_yaml_target_path, err
                    ));
                }
            }
        }
    }

    context.add_thought(format!(
        "After SEARCH_MORE: {} files with full source, {} YAML summaries.",
        context.file_contents_map.len(),
        context.yaml_summaries.len()
    ));

    // After searching, go back to architect to decide next step
    Ok(AgentState::ArchitectDecision)
}
