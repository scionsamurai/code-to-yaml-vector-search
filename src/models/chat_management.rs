// src/models/chat_management.rs
use crate::models::{AppState, ChatMessage, Project};
use actix_web::web;

impl Project {
    pub fn get_analysis_chat_history(
        &self,
        app_state: &web::Data<AppState>,
        query_filename: &str,
    ) -> Vec<ChatMessage> { // Changed return type to Vec<ChatMessage>
        match self.load_query_data_by_filename(app_state, query_filename) {
            Ok(Some(query_data)) => query_data.analysis_chat_history,
            Ok(None) => Vec::new(), // No query data found
            Err(_e) => Vec::new(),  // Error occurred, return empty vector
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
            } else {
                eprintln!("Attempted to update message at index {} but history length is {}", index, qd.analysis_chat_history.len());
            }
        })
    }

    pub fn update_message_visibility(
        &self,
        app_state: &web::Data<AppState>,
        index: usize,
        hidden: bool,
        query_filename: &str,
    ) -> Result<(), String> {
        self.update_query_data(app_state, query_filename, |qd| {
            if index < qd.analysis_chat_history.len() {
                qd.analysis_chat_history[index].hidden = hidden;
            } else {
                eprintln!("Attempted to update message visibility at index {} but history length is {}", index, qd.analysis_chat_history.len());
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