// src/models.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub name: String,
    pub languages: String,
    pub source_dir: String,
    pub model: String
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

