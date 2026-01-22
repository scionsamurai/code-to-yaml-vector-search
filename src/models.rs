// src/models.rs
use chrono;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid; // <--- ADD THIS LINE

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
    #[serde(default)] // Default to None if not present (for backward compatibility)
    pub yaml_model: Option<String>, // New field for YAML conversion model
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
    // This field is kept for backward compatibility.
    // #[serde(default)] ensures it's an empty Vec if not present in old files.
    // #[serde(skip_serializing_if = "Vec::is_empty")] prevents it from being serialized
    // if empty, which effectively cleans up old files after they've been migrated
    // to the new `chat_nodes` structure.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub analysis_chat_history: Vec<ChatMessage>, // Kept for backward compatibility

    // New fields for the linked list structure
    #[serde(default)]
    pub chat_nodes: HashMap<Uuid, ChatMessage>, // Stores all chat messages by ID
    #[serde(default)]
    pub current_node_id: Option<Uuid>, // Points to the ID of the current head of the conversation

    pub llm_analysis: String,
    pub title: Option<String>,
    #[serde(default = "default_false")]
    pub include_file_descriptions: bool,
    #[serde(default = "default_false")]
    pub auto_commit: bool,
    #[serde(default = "default_false")] // ADDED FOR GROUNDING WITH SEARCH
    pub grounding_with_search: bool, // ADDED FOR GROUNDING WITH SEARCH
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

fn default_uuid_v4() -> Uuid {
    Uuid::new_v4()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    #[serde(default = "default_uuid_v4")]
    pub id: Uuid, // Every message gets a unique ID
    #[serde(default)] // Default to None if not present (for backward compatibility)
    pub parent_id: Option<Uuid>, // Points to the message it's a direct reply to

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

// Implement Default for ChatMessage to ensure new messages get an ID and other defaults
impl Default for ChatMessage {
    fn default() -> Self {
        ChatMessage {
            id: Uuid::new_v4(), // Generate a unique ID for new messages
            parent_id: None,
            role: String::default(),
            content: String::default(),
            hidden: false,
            commit_hash: None,
            timestamp: None,
            context_files: None,
            provider: None,
            model: None,
            hidden_context: None,
        }
    }
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

// New struct to pass branching information to the template
#[derive(Debug, Serialize, Deserialize)] // Add Debug trait for easier debugging
pub struct BranchDisplayData {
    pub current_index: usize, // 0-indexed position of this message among its siblings
    pub total_siblings: usize, // Total number of siblings (including itself)
    pub sibling_ids: Vec<Uuid>, // IDs of all siblings (for navigation)
}