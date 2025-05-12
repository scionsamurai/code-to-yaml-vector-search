// src/models.rs
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use chrono;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Project {
    pub name: String,
    pub languages: String,
    pub source_dir: String,
    pub model: String,
    #[serde(default)]
    pub embeddings: HashMap<String, EmbeddingMetadata>,
    pub saved_queries: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub file_descriptions: HashMap<String, String>,
}

impl Project {
    pub fn get_last_query(&self) -> Option<&serde_json::Value> {
        self.saved_queries.as_ref().and_then(|queries| queries.last())
    }

    pub fn get_vector_results(&self) -> Vec<String> {
        self.get_last_query()
            .and_then(|query| query.get("vector_results"))
            .and_then(Value::as_array)
            .map(|results_array| {
                results_array
                    .iter()
                    .filter_map(|result| {
                        result.get(0)?.as_str().map(String::from)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_context_files(&self) -> Vec<String> {
        self.get_last_query()
            .and_then(|query| query.get("context_files"))
            .and_then(Value::as_array)
            .map(|files_array| {
                files_array.iter().filter_map(|f| f.as_str().map(String::from)).collect()
            })
            .unwrap_or_default()
    }

    pub fn get_analysis_chat_history(&self) -> String {
        self.get_last_query()
            .and_then(|query| query.get("analysis_chat_history"))
            .and_then(Value::as_array)
            .map(|history_array| {
                let mut html = String::new();
                for msg in history_array {
                    if let (Some(role), Some(content)) =
                        (msg.get("role").and_then(Value::as_str),
                         msg.get("content").and_then(Value::as_str))
                    {
                        html.push_str(&format!(
                            r#"<div class="chat-message {}-message">
                                <div class="message-content">{}</div>
                                <div class="message-controls">
                                    <button class="edit-message-btn" title="Edit message">Edit</button>
                                </div>
                            </div>"#,
                            role,
                            content
                        ));
                    }
                }
                html
            })
            .unwrap_or_default()
    }

    pub fn get_query_text(&self) -> Option<String> {
        self.get_last_query()
            .and_then(|query| query.get("query"))
            .and_then(Value::as_str)
            .map(String::from)
    }

}

impl std::fmt::Display for EmbeddingMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Define how the struct should be formatted as a string
        write!(f, "EmbeddingMetadata {{ file_path: {}, last_updated: {}, vector_id: {} }}", 
            self.file_path, 
            self.last_updated,
            self.vector_id)
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

