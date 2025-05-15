// src/models/chat_management.rs
use crate::models::{AppState, ChatMessage, Project};
use actix_web::web;

impl Project {
    pub fn get_analysis_chat_history(
        &self,
        app_state: &web::Data<AppState>,
        query_filename: &str,
    ) -> String {
        match self.load_query_data_by_filename(app_state, query_filename) {
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

    // Add a message to chat history
    pub fn add_chat_message(
        &self,
        app_state: &web::Data<AppState>,
        message: ChatMessage,
        query_filename: &str,
    ) -> Result<(), String> {
        self.update_query_data(app_state, query_filename, |qd| {
            qd.analysis_chat_history.push(message);
        })
    }

    // Update a specific message in the history
    pub fn update_message_in_history(
        &self,
        app_state: &web::Data<AppState>,
        index: usize,
        updated_message: ChatMessage,
        query_filename: &str,
    ) -> Result<(), String> {
        self.update_query_data(app_state, query_filename, |qd| {
            if index < qd.analysis_chat_history.len() {
                qd.analysis_chat_history[index] = updated_message;
            }
        })
    }

    // Reset chat history
    pub fn reset_chat_history(
        &self,
        app_state: &web::Data<AppState>,
        query_filename: &str,
    ) -> String {
        // Create a new query data with empty history but preserve other data
        match self.load_query_data_by_filename(app_state, query_filename) {
            Ok(Some(mut query_data)) => {
                query_data.analysis_chat_history = Vec::new();
                self.save_new_query_data(app_state, query_data)
            }
            _ => "Nothing to reset".to_string(),
        }
    }
}
