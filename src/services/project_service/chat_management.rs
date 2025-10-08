// src/services/project_service/chat_management.rs

use crate::models::ChatMessage;
use std::path::Path;
use crate::services::project_service::query_management::QueryManager;

pub struct ChatManager;

impl ChatManager {
    pub fn new() -> Self {
        Self {}
    }

    // Will handle logic from Project::get_analysis_chat_history
    pub fn get_analysis_chat_history(
        &self,
        query_manager: &QueryManager, // Dependency injection
        project_dir: &Path,
        query_filename: &str,
    ) -> Vec<ChatMessage> {
        match query_manager.load_query_data_by_filename(project_dir, query_filename) {
            Ok(Some(query_data)) => query_data.analysis_chat_history,
            _ => Vec::new(),
        }
    }

    // Will handle logic from Project::add_chat_message
    pub fn add_chat_message(
        &self,
        query_manager: &QueryManager, // Dependency injection
        project_dir: &Path,
        message: ChatMessage,
        query_filename: &str,
    ) -> Result<(), String> {
        query_manager.update_query_data_in_project(project_dir, query_filename, |qd| {
            qd.analysis_chat_history.push(message);
        })
    }

    // Will handle logic from Project::update_message_in_history
    pub fn update_message_in_history(
        &self,
        query_manager: &QueryManager, // Dependency injection
        project_dir: &Path,
        index: usize,
        updated_message: ChatMessage,
        query_filename: &str,
    ) -> Result<(), String> {
        query_manager.update_query_data_in_project(project_dir, query_filename, |qd| {
            if index < qd.analysis_chat_history.len() {
                if updated_message.commit_hash == Some("retain".to_string()) {
                    // Retain the existing commit_hash if "retain" is specified
                    let existing_commit_hash = qd.analysis_chat_history[index].commit_hash.clone();
                    qd.analysis_chat_history[index] = ChatMessage {
                        role: updated_message.role,
                        content: updated_message.content,
                        hidden: updated_message.hidden,
                        commit_hash: existing_commit_hash,
                    };
                    return;
                }
                qd.analysis_chat_history[index] = ChatMessage {
                    role: updated_message.role,
                    content: updated_message.content,
                    hidden: updated_message.hidden,
                    commit_hash: updated_message.commit_hash,
                };
            } else {
                eprintln!("Attempted to update message at index {} but history length is {}", index, qd.analysis_chat_history.len());
            }
        })
    }

    // Will handle logic from Project::update_message_visibility
    pub fn update_message_visibility(
        &self,
        query_manager: &QueryManager, // Dependency injection
        project_dir: &Path,
        index: usize,
        hidden: bool,
        query_filename: &str,
    ) -> Result<(), String> {
        query_manager.update_query_data_in_project(project_dir, query_filename, |qd| {
            if index < qd.analysis_chat_history.len() {
                qd.analysis_chat_history[index].hidden = hidden;
            } else {
                eprintln!("Attempted to update message visibility at index {} but history length is {}", index, qd.analysis_chat_history.len());
            }
        })
    }

    // Will handle logic from Project::reset_chat_history
    pub fn reset_chat_history(
        &self,
        query_manager: &QueryManager, // Dependency injection
        project_dir: &Path,
        query_filename: &str,
    ) -> Result<(), String> {
        match query_manager.load_query_data_by_filename(project_dir, query_filename) {
            Ok(Some(mut query_data)) => {
                query_data.analysis_chat_history = Vec::new();
                query_manager.save_query_data(project_dir, &query_data, query_filename)
            }
            Ok(None) => Ok(()), // Nothing to reset if no query data found
            Err(e) => Err(format!("Failed to load query data for reset: {}", e)),
        }
    }
}