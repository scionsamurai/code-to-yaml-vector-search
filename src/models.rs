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