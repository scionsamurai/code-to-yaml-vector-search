// src/services/yaml/mod.rs
pub mod management;
pub mod processing;

pub use management::YamlManagement;
pub use processing::YamlProcessing;

use crate::models::Project;
use std::path::Path;
use serde::{Deserialize, Serialize}; // For FileYamlData
use std::collections::{ HashMap, HashSet }; // For FileYamlData (if needed, currently not)
use crate::services::file::FileService; // To read YAML files


use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize, Clone, Default)] 
pub struct FileYamlData {
    pub description: String,
    #[serde(default)]
    pub functions: Vec<Function>,
    #[serde(default)]
    pub classes: Vec<Class>,
    #[serde(default, rename = "data_structures")]
    pub data_structures: Vec<DataStructure>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Function {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    #[serde(default)]
    pub return_type: Option<String>,
    #[serde(default)]
    pub calls: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Class {
    pub name: String,
    pub inherits: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub methods: Vec<Function>,
    #[serde(default)]
    pub properties: Vec<Parameter>, // Reusing Parameter struct for properties
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataStructure {
    pub name: String,
    #[serde(rename = "type")]
    pub ds_type: String,
    pub description: Option<String>,
    #[serde(default)]
    pub structure: BTreeMap<String, serde_yaml::Value>,
}

pub struct YamlService {
    pub management: YamlManagement,
    processing: YamlProcessing,
}

impl YamlService {
    pub fn new() -> Self {
        Self {
            management: YamlManagement::new(),
            processing: YamlProcessing::new(),
        }
    }

    // Methods that delegate to appropriate modules
    pub async fn save_yaml_files(&self, project: &mut Project, output_dir: &str, force: bool) {
        self.management.generate_yaml_files(project, output_dir, force).await;
    }

    pub async fn check_and_update_yaml_files(&self, project: &mut Project, output_dir: &str) {
        self.management.check_and_update_yaml_embeddings(project, output_dir).await;
    }

    pub fn clean_up_orphaned_files(&self, project_name: &str, orphaned_files: Vec<String>) {
        self.management.clean_up_orphaned_files(project_name, orphaned_files);
    }

    pub fn process_yaml_files(&self, output_dir: &Path, project_name: &str, project: &mut Project)
        -> Result<(String, Vec<(String, String)>, bool, Vec<String>), String>
    {
        self.processing.process_yaml_files(output_dir, project_name, project)
    }

    /// BM25F Placeholder: Performs a keyword search over YAML data.
    /// In a real implementation, this would be a proper BM25F ranking.
    /// For this example, it's a simple contains-based keyword match with basic scoring.
    pub async fn bm25f_search(
        &self,
        project: &Project,
        query_text: &str,
        output_dir: &Path,
        num_results: usize,
    ) -> Result<Vec<(String, f32)>, String> {
        let file_service = FileService {};
        let yaml_files_dir = output_dir;

        let query_keywords: HashSet<String> = query_text
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if query_keywords.is_empty() {
            return Ok(Vec::new());
        }

        let mut scores: HashMap<String, f32> = HashMap::new();

        // Iterate through all YAML files generated for the project
        if let Ok(entries) = std::fs::read_dir(&yaml_files_dir) {
            for entry in entries {
                let entry = entry.map_err(|e| format!("Error reading directory entry: {}", e))?;
                let path = entry.path();

                if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                    let file_content = file_service.read_specific_file(project, &path.to_string_lossy());
                    let yaml_data: FileYamlData = serde_yaml::from_str(&file_content.unwrap())
                        .map_err(|e| format!("Failed to parse YAML file {}: {}", path.display(), e))?;

                    let mut current_score = 0.0;

                    // Score based on keyword presence in different fields
                    for keyword in &query_keywords {
                        // Summary (High weight)
                        if yaml_data.description.to_lowercase().contains(keyword) {
                            current_score += 3.0;
                        }
                        // Calls (Medium weight) from functions and class methods
                        let has_call = yaml_data.functions.iter()
                            .flat_map(|f| f.calls.iter())
                            .any(|c| c.to_lowercase().contains(keyword))
                            || yaml_data.classes.iter()
                                .flat_map(|c| c.methods.iter())
                                .flat_map(|m| m.calls.iter())
                                .any(|c| c.to_lowercase().contains(keyword));
                        if has_call {
                            current_score += 2.0;
                        }
                        // File path (Low weight) - use YAML file path on disk
                        if path.to_string_lossy().to_lowercase().contains(keyword) {
                            current_score += 0.5;
                        }
                    }

                    if current_score > 0.0 {
                        scores.insert(path.to_string_lossy().into_owned(), current_score);
                    }
                }
            }
        } else {
            // Directory might not exist yet, which is fine if no YAMLs have been generated
            println!("YAML files directory not found: {}. No YAML-based search will be performed.", yaml_files_dir.display());
            return Ok(Vec::new());
        }

        // Sort by score and take top results
        let mut sorted_scores: Vec<(String, f32)> = scores.into_iter().collect();
        sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(sorted_scores.into_iter().take(num_results).collect())
    }
}