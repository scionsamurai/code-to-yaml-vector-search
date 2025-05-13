// src/models.rs
use crate::services::project_service::ProjectService;
use actix_web::web;
use chrono;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Project {
    pub name: String,
    pub languages: String,
    pub source_dir: String,
    pub model: String,
    #[serde(default)]
    pub embeddings: HashMap<String, EmbeddingMetadata>,
    #[serde(default)]
    pub file_descriptions: HashMap<String, String>,
}

impl Project {
    pub fn load_most_recent_query_data(
        &self,
        app_state: &web::Data<AppState>,
    ) -> Result<Option<QueryData>, String> {
        let project_service = ProjectService::new();
        let project_dir = self.get_project_dir(app_state);

        match project_service.get_most_recent_query_file(&project_dir) {
            Ok(most_recent_file) => {
                match most_recent_file {
                    Some(file_path) => {
                        let file_name =
                            file_path.file_name().unwrap().to_str().unwrap().to_string();
                        match project_service.load_query_data(&project_dir, &file_name) {
                            Ok(query_data) => Ok(Some(query_data)),
                            Err(e) => {
                                eprintln!("Error loading query data: {}", e);
                                Ok(None) // Return None if loading fails, handle appropriately in caller
                            }
                        }
                    }
                    None => Ok(None), // No recent file
                }
            }
            Err(e) => {
                eprintln!("Error getting most recent query file: {}", e);
                Err(e) // Propagate the error
            }
        }
    }

    pub fn get_vector_results(&self, app_state: &web::Data<AppState>) -> Vec<String> {
        match self.load_most_recent_query_data(app_state) {
            Ok(Some(query_data)) => {
                query_data.vector_results
                    .into_iter()
                    .map(|(path, _)| path)
                    .collect()
            }
            Ok(None) => Vec::new(), // No query data found
            Err(_e) => Vec::new(),  // Error occurred, return empty vector
        }
    }

    pub fn get_context_files(&self, app_state: &web::Data<AppState>) -> Vec<String> {
        match self.load_most_recent_query_data(app_state) {
            Ok(Some(query_data)) => query_data.context_files,
            Ok(None) => Vec::new(), // No query data found
            Err(_e) => Vec::new(),  // Error occurred, return empty vector
        }
    }

    pub fn get_analysis_chat_history(&self, app_state: &web::Data<AppState>) -> String {
        match self.load_most_recent_query_data(app_state) {
            Ok(Some(query_data)) => {
                let mut html = String::new();
                for msg in query_data.analysis_chat_history {
                    html.push_str(&format!(
                        r#"<div class="chat-message {}-message">
                            <div class="message-content">{}</div>
                            <div class="message-controls">
                                <button class="edit-message-btn" title="Edit message">Edit</button>
                            </div>
                        </div>"#,
                        msg.role, msg.content
                    ));
                }
                html
            }
            Ok(None) => String::new(), // No query data found
            Err(_e) => String::new(),  // Error occurred, return empty string
        }
    }

    pub fn get_query_text(&self, app_state: &web::Data<AppState>) -> Option<String> {
        match self.load_most_recent_query_data(app_state) {
            Ok(Some(query_data)) => Some(query_data.query),
            Ok(None) => None, // No query data found
            Err(_e) => None,  // Error occurred, return None
        }
    }

    pub fn save_query_data(
        &self,
        app_state: &web::Data<AppState>,
        query_data: QueryData,
    ) -> Result<(), String> {
        let project_service = ProjectService::new();
        let project_dir = self.get_project_dir(app_state);
        let filename = project_service.generate_query_filename();

        project_service.save_query_data(&project_dir, &query_data, &filename)
    }

    // Update an existing query data or create a new one
    pub fn update_query_data(
        &self,
        app_state: &web::Data<AppState>,
        update_fn: impl FnOnce(&mut QueryData),
    ) -> Result<(), String> {
        let project_service = ProjectService::new();
        let project_dir = self.get_project_dir(app_state);

        // Try to load existing query data or create new
        let (mut query_data, filename) = match self.load_most_recent_query_data(app_state) {
            Ok(Some(qd)) => {
                // Get filename of most recent file
                let file_path = project_service
                    .get_most_recent_query_file(&project_dir)
                    .map_err(|e| format!("Failed to get filename: {}", e))?
                    .ok_or_else(|| "No query file found".to_string())?;
                let filename = file_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .ok_or_else(|| "Invalid filename".to_string())?
                    .to_string();
                (qd, filename)
            }
            _ => {
                // Create new query data and filename
                (
                    QueryData::default(),
                    project_service.generate_query_filename(),
                )
            }
        };

        // Apply the update function
        update_fn(&mut query_data);

        // Save the updated query data
        project_service.save_query_data(&project_dir, &query_data, &filename)
    }

    // Add a message to chat history
    pub fn add_chat_message(
        &self,
        app_state: &web::Data<AppState>,
        message: ChatMessage,
    ) -> Result<(), String> {
        self.update_query_data(app_state, |qd| {
            qd.analysis_chat_history.push(message);
        })
    }

    // Update a specific message in the history
    pub fn update_message_in_history(
        &self,
        app_state: &web::Data<AppState>,
        index: usize,
        updated_message: ChatMessage,
    ) -> Result<(), String> {
        self.update_query_data(app_state, |qd| {
            if index < qd.analysis_chat_history.len() {
                qd.analysis_chat_history[index] = updated_message;
            }
        })
    }

    // Reset chat history
    pub fn reset_chat_history(&self, app_state: &web::Data<AppState>) -> Result<(), String> {
        // Create a new query data with empty history but preserve other data
        match self.load_most_recent_query_data(app_state) {
            Ok(Some(mut query_data)) => {
                query_data.analysis_chat_history = Vec::new();
                self.save_query_data(app_state, query_data)
            }
            _ => Ok(()), // Nothing to reset
        }
    }

    // Get project directory
    pub fn get_project_dir(&self, app_state: &web::Data<AppState>) -> PathBuf {
        let output_dir = Path::new(&app_state.output_dir);
        output_dir.join(&self.name)
    }
}

impl std::fmt::Display for EmbeddingMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Define how the struct should be formatted as a string
        write!(
            f,
            "EmbeddingMetadata {{ file_path: {}, last_updated: {}, vector_id: {} }}",
            self.file_path, self.last_updated, self.vector_id
        )
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectFile {
    pub path: String,
    pub content: String,
    pub last_modified: u64,
}

pub struct AppState {
    pub output_dir: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbeddingMetadata {
    pub file_path: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub vector_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct QueryData {
    pub query: String,
    pub vector_results: Vec<(String, f32)>,
    pub context_files: Vec<String>,
    pub analysis_chat_history: Vec<ChatMessage>,
    pub llm_analysis: String,
}
