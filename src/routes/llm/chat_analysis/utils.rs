// src/routes/llm/chat_analysis/utils.rs
use crate::models::{Project, ChatMessage, AppState};
use crate::services::file::FileService;
use crate::services::project_service::ProjectService;
use crate::services::llm_service::LlmService;
use crate::services::git_service::{GitService, GitError};
use git2::Repository;
use actix_web::web;
use std::path::Path;

pub fn get_context_and_contents(project: &Project, app_state: &web::Data<AppState>, query_id: &str) -> (Vec<String>, String) {
    // Get selected context files from project
    let project_dir = Path::new(&app_state.output_dir).join(&project.name);
    let project_service = ProjectService::new(); // Create an instance of ProjectService

    let context_files = project_service.query_manager.get_query_vec_field(&project_dir, query_id, "context_files").unwrap_or_default();

    let file_service = FileService {};

    // Load file contents for the selected files
    let file_contents = context_files.iter()
        .filter_map(|file_path| {
            if let Some(content) = file_service.read_specific_file(project, file_path) {
                Some(format!("--- FILE: {} ---\n{}\n\n", file_path, content))
            } else {
                None
            }
        })
        .collect::<String>();

    (context_files, file_contents)
}

pub fn create_system_prompt(
    query: &str,
    context_files: &Vec<String>,
    file_contents: &str,
    project: &Project, // Add project here
    include_file_descriptions: bool, // Add this flag
) -> String {
    let mut prompt = format!("You are an AI assistant helping with code analysis for a project. In this chat the user controls which files you see and which messages you see with every prompt. \
        The user's original query was: \"{}\"", query);

    if include_file_descriptions && !project.file_descriptions.is_empty() {
        prompt.push_str("\n\nHere are descriptions for some of the project files:");
        for (path, description) in project.file_descriptions.iter() {
            prompt.push_str(&format!("\n- Path: {}\n  Description: {}", path, description));
        }
        prompt.push_str("\n");
    }

    if !context_files.is_empty() {
        prompt.push_str("\n\nPlease note: The files provided within this message context are live and updated with every message. They represent the user's current code state, which often incorporates their attempts to implement previous suggestions or fix bugs. Always refer to these files for the latest version for all requests. The user may also change which files are included.");
        // prompt.push_str(&format!("\n\nYou have access to the following files:\n{}", context_files.join("\n")));
    }

    if !file_contents.is_empty() {
        prompt.push_str(&format!("\n\nHere are the files and their contents:\n\n{}", file_contents));
    }

    prompt
 }

 pub fn get_full_history(project: &Project, app_state: &web::Data<AppState>, query_id: &str) -> Vec<ChatMessage> {
    let project_dir = Path::new(&app_state.output_dir).join(&project.name);
    let project_service = ProjectService::new(); // Create an instance of ProjectService
    project_service.chat_manager.get_analysis_chat_history(&project_service.query_manager, &project_dir, query_id)
}

fn replace_hidden_messages(messages: &mut Vec<ChatMessage>) {
    for message in messages.iter_mut() {
        if message.hidden {
            message.content = "User hid this message due to it no longer being contextually necessary and/or it was redundant info.".to_string();
        }
    }
}

pub fn format_messages_for_llm(system_prompt: &str, full_history: &Vec<ChatMessage>, user_message: &ChatMessage) -> Vec<ChatMessage> {
    let mut messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: system_prompt.to_string(),
            hidden: false,
            commit_hash: None,
        },
        ChatMessage {
            role: "model".to_string(),
            content: "I confirm that I'll follow your instructions carefully throughout our conversation. I'm here to assist you according to your specific requirements and will respond to your future requests for code analysis appropriately when needed.\n\nPlease feel free to share your next request when you're ready, and I'll provide the analysis or other assistance you're looking for.".to_string(),
            hidden: false,
            commit_hash: None,
        }
    ];

    messages.extend(full_history.clone());

    messages.push(user_message.clone());

     // Create a mutable copy of the messages
    let mut llm_messages = messages.clone();
    // Modify the mutable copy to replace hidden messages
    replace_hidden_messages(&mut llm_messages);

    llm_messages
}


pub async fn generate_commit_message(
    llm_service: &LlmService,
    repo: &Repository,
    project: &Project,
    query_text: &str,
    unescaped_history: &[ChatMessage],
) -> String {
    // Check if there are uncommitted changes

    let mut commit_llm_messages: Vec<ChatMessage> = Vec::new();

    // Get the git diff for uncommitted changes
    let git_diff_output = match GitService::get_uncommitted_diff(&repo) {
        Ok(diff) => diff,
        Err(e) => {
            eprintln!("Failed to get uncommitted diff for commit message generation: {:?}", e);
            // Fallback to a simpler message or continue without diff
            "Could not retrieve code changes for detailed commit message.".to_string()
        }
    };

    // Determine relevant chat history since the last recorded commit
    let mut relevant_history_for_commit_msg: Vec<ChatMessage> = Vec::new();
    let mut last_commit_idx: Option<usize> = None;

    // Find the index of the last message in history that had a commit_hash
    for (idx, msg) in unescaped_history.iter().enumerate() {
        if msg.commit_hash.is_some() {
            last_commit_idx = Some(idx);
        }
    }

    if let Some(idx) = last_commit_idx {
        // Collect all messages *after* the one with the last commit hash
        // This assumes the commit message should reflect the changes
        // that occurred as a result of the conversation *since* that last commit.
        if idx + 1 < unescaped_history.len() {
            relevant_history_for_commit_msg.extend_from_slice(&unescaped_history[idx + 1..]);
        }
    } else {
        // If no commit hash found in history, all history is relevant
        // (implies this might be the first commit for this query)
        relevant_history_for_commit_msg.extend_from_slice(&unescaped_history);
    }
     // Create a mutable copy of the messages
    let mut visible_history = relevant_history_for_commit_msg.clone();
    // Modify the mutable copy to replace hidden messages
    replace_hidden_messages(&mut visible_history);
    
    let formatted_relevant_history = if !visible_history.is_empty() {
        visible_history
            .iter()
            .map(|msg| format!("{}: {}", msg.role, msg.content))
            .collect::<Vec<String>>()
            .join("\n")
    } else {
        "No relevant chat history since the last tracked commit.".to_string()
    };

    let mut user_messsage_for_commit = String::new();

    user_messsage_for_commit.push_str("You are an AI assistant tasked with generating concise Git commit messages. Your response *must* be only the commit message itself, with no 'Auto:' prefix, conversational text, confirmations of compliance, or extraneous information. The message should be a short, descriptive summary (under 100 characters) of the provided code changes and the conversation that led to them. Focus on the core change or task requested in the previous interaction.");
    // Add the initial query for overall context
    user_messsage_for_commit.push_str(&format!("Initial chat query for context: {}\n", query_text));

    // Add the relevant chat history
    user_messsage_for_commit.push_str(&format!("Relevant chat history leading to these changes:\n{}\n", formatted_relevant_history));

    // Add the code changes (git diff)
    user_messsage_for_commit.push_str(&format!("Here are the uncommitted code changes:\n```diff\n{}\n```", git_diff_output));

    // Final instruction to generate the message
    user_messsage_for_commit.push_str("\nBased on the above, provide the summarized Git commit message.");
    
    commit_llm_messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_messsage_for_commit,
        hidden: false,
        commit_hash: None,
    });

    let generated_message_llm_response = llm_service
        .send_conversation(
            &commit_llm_messages,
            &project.provider.clone(),
            project.specific_model.as_deref(),
        )
        .await;

    // Clean up the LLM response (e.g., remove quotes or unwanted formatting)
    let cleaned_commit_message = generated_message_llm_response
        .trim_matches(|c| c == '"' || c == '\'') // Remove surrounding quotes
        .lines()
        .next() // Take only the first line for conciseness
        .unwrap_or("Generated commit message failed.")
        .trim() // Trim any remaining whitespace
        .to_string();


    // Ensure it starts with "Auto:"
    format!("Auto: {}", cleaned_commit_message)

}