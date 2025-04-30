// src/services/yaml/management/mod.rs
use crate::services::file_service::FileService;
use crate::services::llm_service::LlmService;
use crate::models::Project;

pub mod generation;
pub mod embedding;
pub mod cleanup;

pub struct YamlManagement {
    pub file_service: FileService,
    pub llm_service: LlmService,
}

impl YamlManagement {
    pub fn new() -> Self {
        Self {
            file_service: FileService {},
            llm_service: LlmService {},
        }
    }
    
    // Move these functions from the standalone to be methods
    pub async fn generate_yaml_files(&self, project: &mut Project, output_dir: &str) {
        generation::generate_yaml_files(self, project, output_dir).await;
    }

    pub async fn check_and_update_yaml_embeddings(&self, project: &mut Project, output_dir: &str) {
        embedding::check_and_update_yaml_embeddings(project, output_dir).await;
    }

    pub fn clean_up_orphaned_files(&self, project_name: &str, orphaned_files: Vec<String>) {
        cleanup::clean_up_orphaned_files(project_name, orphaned_files);
    }
}