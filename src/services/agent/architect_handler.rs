use std::collections::HashMap;

use crate::models::Project;
use crate::services::llm_service::LlmService;
use crate::services::agent::architect;
use crate::services::agent::state::{AgentState, AgentContext};

/// Handles the architect's decision-making process using the state machine.
/// Returns the next AgentState based on the architect's decision.
pub async fn handle_architect_decision_stateful(
    llm_service: &LlmService,
    project: &Project,
    initial_query: &str,
    user_message_content_raw: &str,
    context: &mut AgentContext,
) -> Result<AgentState, String> {
    // Prepare YAML descriptions for architect (limited number, exclude files with full source)
    let mut proactive_file_descriptions_for_architect_prompt: HashMap<String, String> = HashMap::new();
    let max_proactive_descriptions_for_architect = 30;

    let mut sorted_yamls: Vec<(&String, &String)> = context.yaml_summaries.iter().collect();
    sorted_yamls.sort_by_key(|(path, _)| path.as_str());

    for (path, desc) in sorted_yamls.into_iter() {
        if !context.file_contents_map.contains_key(path)
            && proactive_file_descriptions_for_architect_prompt.len() < max_proactive_descriptions_for_architect
        {
            proactive_file_descriptions_for_architect_prompt.insert(path.clone(), desc.clone());
        }
    }

    context.add_thought(format!(
        "Refined proactive descriptions for Architect to {} files (not already full source).",
        proactive_file_descriptions_for_architect_prompt.len()
    ));

    context.add_thought("Requesting Architect LLM decision on next steps.".to_string());
    let architect_decision = architect::get_architect_decision(
        llm_service,
        project,
        initial_query,
        user_message_content_raw,
        &proactive_file_descriptions_for_architect_prompt,
        &context.file_contents_map,
    ).await?;

    context.add_thought(format!("Architect decision: {:?}", architect_decision));

    let action = architect_decision["action"]
        .as_str()
        .ok_or_else(|| "Architect decision missing 'action' field or not a string.".to_string())?;

    let reason = architect_decision["reason"]
        .as_str()
        .unwrap_or("No reason provided.")
        .to_string();
    context.add_thought(format!("Architect Reason: {}", reason));

    match action {
        "FETCH_SOURCE" => {
            context.add_thought("Architect decided to FETCH_SOURCE.".to_string());
            let paths_to_fetch = architect_decision["paths"]
                .as_array()
                .ok_or_else(|| "FETCH_SOURCE action requires 'paths' array.".to_string())?
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<String>>();

            if paths_to_fetch.is_empty() {
                context.add_thought("Warning: FETCH_SOURCE with empty paths. Proceeding to GENERATE.".to_string());
                return Ok(AgentState::ReadyToGenerate);
            }

            Ok(AgentState::FetchingSource(paths_to_fetch))
        },
        "SEARCH_MORE" => {
            context.add_thought("Architect decided to SEARCH_MORE.".to_string());
            let new_keywords = architect_decision["keywords"]
                .as_str()
                .ok_or_else(|| "SEARCH_MORE action requires 'keywords' string.".to_string())?
                .to_string();

            if new_keywords.trim().is_empty() {
                context.add_thought("Warning: SEARCH_MORE with empty keywords. Proceeding to GENERATE.".to_string());
                return Ok(AgentState::ReadyToGenerate);
            }

            Ok(AgentState::SearchingMore(new_keywords))
        },
        "GENERATE" => {
            context.add_thought("Architect decided to GENERATE directly.".to_string());
            Ok(AgentState::ReadyToGenerate)
        },
        _ => {
            context.add_thought(format!(
                "Architect returned unknown action: '{}'. Defaulting to GENERATE.",
                action
            ));
            Ok(AgentState::ReadyToGenerate)
        }
    }
}
