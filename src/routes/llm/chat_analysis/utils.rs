// src/routes/llm/chat_analysis/utils.rs
use crate::models::{Project, ChatMessage};
use crate::services::project_service::ProjectService;
use crate::services::file_service::FileService;
use serde_json::Value;
use actix_web::Result;
use super::models::*;

pub fn get_context_and_contents(project: &Project, file_service: &FileService) -> (Vec<String>, String) {
    // Get selected context files from project settings
    let context_files = if let Some(saved_queries) = &project.saved_queries {
        if let Some(last_query) = saved_queries.last() {
            if let Some(files) = last_query.get("context_files") {
                if let Some(files_array) = files.as_array() {
                    files_array.iter()
                        .filter_map(|f| f.as_str().map(String::from))
                        .collect::<Vec<String>>()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Load file contents for the selected files
    let file_contents = context_files.iter()
        .filter_map(|file_path| {
            if let Some(content) = file_service.read_specific_file(&project, file_path) {
                Some(format!("--- FILE: {} ---\n{}\n\n", file_path, content))
            } else {
                None
            }
        })
        .collect::<String>();

    (context_files, file_contents)
}

pub fn create_system_prompt(query: &str, context_files: &Vec<String>, file_contents: &str) -> String {
    format!(
        "You are an AI assistant helping with code analysis for a project. \
        The user's original query was: \"{}\"\n\n\
        You have access to the following files:\n{}\n\n\
        Here are the contents of these files:\n\n{}",
        query,
        context_files.join("\n"),
        file_contents
    )
}

pub fn get_full_history(project: &Project) -> Vec<ChatMessage> {
    if let Some(saved_queries) = &project.saved_queries {
        if let Some(last_query) = saved_queries.last() {
            if let Some(history) = last_query.get("analysis_chat_history") {
                if let Ok(messages) = serde_json::from_value::<Vec<ChatMessage>>(history.clone()) {
                    messages
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    }
}

pub fn format_messages(system_prompt: &str, full_history: &Vec<ChatMessage>, user_message: &str) -> Vec<ChatMessage> {
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

    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_message.to_string(),
    });

    messages
}

pub fn update_and_save_history(project: &mut Project, project_dir: &std::path::PathBuf, full_history: Vec<ChatMessage>, project_service: ProjectService) {
    if project.saved_queries.is_none() {
        project.saved_queries = Some(Vec::new());
    }

    if let Some(saved_queries) = &mut project.saved_queries {
        if let Some(last_query) = saved_queries.last_mut() {
            // Update the last query with the analysis chat history
            last_query["analysis_chat_history"] = serde_json::to_value(&full_history).unwrap_or_default();

            // Save the updated project settings
            if let Err(e) = project_service.save_project(&project, &project_dir) {
                eprintln!("Failed to save project: {}", e);
            }
        }
    }
}

pub fn reset_chat_history(project: &mut Project) {
    if let Some(saved_queries) = &mut project.saved_queries {
        if let Some(last_query) = saved_queries.last_mut() {
            // Remove the analysis chat history
            last_query.as_object_mut().unwrap().remove("analysis_chat_history");
        }
    }
}

pub fn save_chat_history(project: &mut Project, history: &Vec<ChatMessage>) {
    if project.saved_queries.is_none() {
        project.saved_queries = Some(Vec::new());
    }

    if let Some(saved_queries) = &mut project.saved_queries {
        if let Some(last_query) = saved_queries.last_mut() {
            // Update the last query with the analysis chat history
            last_query["analysis_chat_history"] = serde_json::to_value(&history).unwrap_or_default();
        }
    }
}

pub fn update_message_in_history(project: &mut Project, data: &UpdateChatMessageRequest) -> Result<String, String> {
    if let Some(saved_queries) = &mut project.saved_queries {
        if let Some(last_query) = saved_queries.last_mut() {
            if let Some(chat_history) = last_query.get_mut("analysis_chat_history") {
                if let Some(history_array) = chat_history.as_array_mut() {
                    // Make sure the index is valid
                    if data.index < history_array.len() {
                        // Create the updated message
                        let updated_message = serde_json::json!({
                            "role": data.role.clone(),
                            "content": data.content.clone()
                        });

                        // Update the message at the specified index
                        history_array[data.index] = updated_message;

                        return Ok("Message updated successfully".to_string());
                    } else {
                        return Err("Invalid message index".to_string());
                    }
                }
            }
        }
    }

    Err("Failed to update message".to_string())
}