// src/models.rs
use chrono;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// pub mod query_management;
// pub mod chat_management;
pub mod utils;
// pub mod query;


fn default_use_yaml_default() -> bool {
    true
}

// Helper function for default_include_file_descriptions
fn default_false() -> bool {
    false
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Project {
    pub name: String,
    pub languages: String,
    pub source_dir: String,
    pub provider: String,
    #[serde(default)]
    pub specific_model: Option<String>,
    #[serde(default)]
    pub embeddings: HashMap<String, EmbeddingMetadata>,
    #[serde(default)]
    pub file_descriptions: HashMap<String, String>,
    #[serde(default = "default_use_yaml_default")]
    pub default_use_yaml: bool,
    #[serde(default)]
    pub file_yaml_override: HashMap<String, bool>,
    #[serde(default = "default_false")]
    pub git_integration_enabled: bool,
    pub git_branch_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct QueryData {
    pub query: String,
    pub vector_results: Vec<(String, f32)>,
    pub context_files: Vec<String>,
    pub analysis_chat_history: Vec<ChatMessage>,
    pub llm_analysis: String,
    pub title: Option<String>,
    #[serde(default = "default_false")]
    pub include_file_descriptions: bool,
    #[serde(default = "default_false")]
    pub auto_commit: bool, 
}

impl std::fmt::Display for EmbeddingMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EmbeddingMetadata {{ file_path: {}, last_updated: {}, vector_id: {}, git_blob_hash: {:?} }}", // Added git_blob_hash
            self.file_path, self.last_updated, self.vector_id, self.git_blob_hash
        )
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub hidden: bool,
    pub commit_hash: Option<String>,  
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>, // Add timestamp
    pub context_files: Option<Vec<String>>,       // Add context files
    pub provider: Option<String>,              // Add provider
    pub model: Option<String>,                 // Add model
    pub hidden_context: Option<Vec<String>>,    // Add hidden context representation
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectFile {
    pub path: String,
    pub content: String,
    pub last_modified: u64,
}
#[derive(Clone, Debug)]
pub struct AppState {
    pub output_dir: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbeddingMetadata {
    pub file_path: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub vector_id: String,
    #[serde(default)] // Make it optional and default to None for backward compatibility
    pub git_blob_hash: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateQuery {
    pub force: Option<bool>,
}