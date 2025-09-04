// src/routes/llm/chat_analysis/utils.rs
use crate::models::{Project, ChatMessage, AppState};
use crate::services::file::FileService;
use actix_web::web;

pub fn get_context_and_contents(project: &Project, app_state: &web::Data<AppState>, query_id: &str) -> (Vec<String>, String) {
    // Get selected context files from project 
    let context_files = project.get_query_vec_field(app_state, query_id, "context_files").unwrap();
    
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
    let mut prompt = format!("You are an AI assistant helping with code analysis for a project. In this chat the user controls which files you see and which messages you see with every prompt. \
        The user's original query was: \"{}\"", query);
    
    if !context_files.is_empty() {
        prompt.push_str("\n\nPlease note: The files provided within this message context are live and updated with every message. They represent the user's current code state, which often incorporates their attempts to implement previous suggestions or fix bugs. Always refer to these files for the latest version for all requests. The user may also change which files are included.");
        prompt.push_str(&format!("\n\nYou have access to the following files:\n{}", context_files.join("\n")));
    }
    
    if !file_contents.is_empty() {
        prompt.push_str(&format!("\n\nHere are the contents of these files:\n\n{}", file_contents));
    }
    
    prompt
 }

 pub fn get_full_history(project: &Project, app_state: &web::Data<AppState>, query_id: &str) -> Vec<ChatMessage> {
    match project.load_query_data_by_filename(app_state, query_id) {
        Ok(Some(query_data)) => query_data.analysis_chat_history,
        _ => Vec::new()
    }
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

pub fn unescape_html(text: String) -> String {
    let mut unescaped_text = text.replace("&#96;&#96;&#96;", "```");

    unescaped_text = unescaped_text.replace("&lt;", "<");
    unescaped_text = unescaped_text.replace("&gt;", ">");
    unescaped_text = unescaped_text.replace("&quot;", "\"");
    unescaped_text = unescaped_text.replace("&#34;", "\""); // For &#34; (double quote)
    unescaped_text = unescaped_text.replace("&#39;", "'");  // For &#39; (single quote/apostrophe)
    unescaped_text = unescaped_text.replace("&apos;", "'"); // For &apos; (named entity for apostrophe, though less common)
    unescaped_text = unescaped_text.replace("&amp;", "&"); // This MUST be last

    unescaped_text
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