// src/routes/llm/chat_analysis/utils.rs
use crate::models::{Project, ChatMessage, AppState};
use crate::services::file::FileService;
use crate::services::project_service::ProjectService; // ADDED
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
        },
        ChatMessage {
            role: "model".to_string(),
            content: "I confirm that I'll follow your instructions carefully throughout our conversation. I'm here to assist you according to your specific requirements and will respond to your future requests for code analysis appropriately when needed.\n\nPlease feel free to share your next request when you're ready, and I'll provide the analysis or other assistance you're looking for.".to_string(),
            hidden: false,
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