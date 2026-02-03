use std::path::Path;
use actix_web::web;

use crate::models::{ChatMessage, Project, AppState};
use crate::services::llm_service::{LlmService, LlmServiceConfig};
use crate::services::project_service::query_management::QueryManager;
use crate::services::search_service::SearchService;
use crate::services::yaml::YamlService;

pub mod architect;
pub mod architect_handler;
pub mod search_handler;
pub mod file_context;
pub mod search_results_handler;
pub mod response_generation;
pub mod state; // NEW

use state::{AgentState, AgentContext, 
    handle_initial_search, 
    handle_architect_turn, 
    handle_fetch_source, 
    handle_search_more,
};

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

    // Initialize agent context
    let max_architect_turns = 3; // Configurable
    let mut context = AgentContext::new(max_architect_turns);
    
    context.add_thought("Agentic mode is enabled. Starting agent decision process.".to_string());

    let initial_query = query_manager
        .get_query_data_field(&project_dir, query_id, "query")
        .unwrap_or_else(|| "No previous query found".to_string());

    let mut llm_config = LlmServiceConfig::new();
    if enable_grounding {
        llm_config = llm_config.with_grounding_with_search(true);
        context.add_thought("Grounding with Google Search is enabled for this LLM call.".to_string());
    }

    // State machine loop
    let mut current_state = AgentState::InitialSearch;

    loop {
        context.add_thought(format!("Current state: {:?}", current_state));

        current_state = match current_state {
            AgentState::InitialSearch => {
                handle_initial_search(
                    &llm_service,
                    &search_service,
                    &yaml_service,
                    project,
                    &initial_query,
                    user_message_content_raw,
                    previous_history_for_llm,
                    &llm_config,
                    &project_dir,
                    &mut context,
                ).await?
            },

            AgentState::ArchitectDecision => {
                handle_architect_turn(
                    &llm_service,
                    project,
                    &initial_query,
                    user_message_content_raw,
                    &mut context,
                ).await?
            },

            AgentState::FetchingSource(paths) => {
                handle_fetch_source(
                    project,
                    paths,
                    &mut context,
                ).await?
            },

            AgentState::SearchingMore(keywords) => {
                handle_search_more(
                    &yaml_service,
                    project,
                    &project_dir,
                    keywords,
                    &mut context,
                ).await?
            },

            AgentState::ReadyToGenerate => {
                // Exit the loop and proceed to generation
                break;
            },

            AgentState::Error(error_msg) => {
                return Err(error_msg);
            },
        };
    }

    // --- PHASE 3: Final LLM Generation ---
    context.add_thought("Initiating final LLM response generation.".to_string());
    let model_chat_message = response_generation::generate_llm_response(
        &llm_service,
        project,
        query_id,
        user_message_content_raw,
        llm_config,
        include_file_descriptions,
        previous_history_for_llm,
        commit_hash_for_user_message,
        hidden_context,
        &context.file_contents_map,
        &context.yaml_summaries,
        &project_dir,
        &mut context.thoughts,
    ).await?;

    Ok(model_chat_message)
}
