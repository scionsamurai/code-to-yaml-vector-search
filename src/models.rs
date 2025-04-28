// src/models.rs
use serde::{Deserialize, Serialize};
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

impl std::fmt::Display for EmbeddingMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Define how the struct should be formatted as a string
        write!(f, "EmbeddingMetadata {{ file_path: {}, last_updated: {}, vector_id: {} }}", 
            self.file_path, 
            self.last_updated, //chrono::DateTime implements Display
            self.vector_id)
    }
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