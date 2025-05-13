// src/routes/llm/chat_analysis/utils.rs
use crate::models::{Project, ChatMessage, AppState};
use crate::services::file_service::FileService;
use actix_web::web;

pub fn get_context_and_contents(project: &Project, app_state: &web::Data<AppState>) -> (Vec<String>, String) {
    // Get selected context files from project 
    let context_files = project.get_context_files(app_state);
    
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

pub fn create_system_prompt(query: &str, context_files: &Vec<String>, file_contents: &str) -> String {
    let mut prompt = format!("You are an AI assistant helping with code analysis for a project. \
        The user's original query was: \"{}\"", query);
    
    if !context_files.is_empty() {
        prompt.push_str(&format!("\n\nYou have access to the following files:\n{}", context_files.join("\n")));
    }
    
    if !file_contents.is_empty() {
        prompt.push_str(&format!("\n\nHere are the contents of these files:\n\n{}", file_contents));
    }
    
    prompt
 }

 pub fn get_full_history(project: &Project, app_state: &web::Data<AppState>) -> Vec<ChatMessage> {
    match project.load_most_recent_query_data(app_state) {
        Ok(Some(query_data)) => query_data.analysis_chat_history,
        _ => Vec::new()
    }
}

pub fn format_messages(system_prompt: &str, full_history: &Vec<ChatMessage>, user_message: &ChatMessage) -> Vec<ChatMessage> {
    let mut messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: system_prompt.to_string(),
        },
        ChatMessage {
            role: "model".to_string(),
            content: "I understand.".to_string(),
        }
    ];

    messages.extend(full_history.clone());

    messages.push(user_message.clone());

    messages
}

pub async fn escape_html(text: String) -> String {
    // Process text line by line to handle code block markers vs inline triple backticks
    let processed_text = text.lines()
        .map(|line| {
            let trimmed = line.trim();
            // If the line starts with triple backticks after trimming, leave it as is
            if trimmed.starts_with("```") {
                line.to_string()
            } else {
                // Replace any triple backticks in the middle of the line
                line.replace("```", "&grave;&grave;&grave;")
            }
        })
        .collect::<Vec<String>>()
        .join("\n");
    
    // Perform normal HTML escaping on the processed text
    html_escape::encode_text(&processed_text)
        .to_string()
        .replace("\"", "&#34;")
}
