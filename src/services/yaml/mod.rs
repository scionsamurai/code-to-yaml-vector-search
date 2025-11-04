// src/services/yaml/mod.rs
pub mod management;
pub mod processing;

pub use management::YamlManagement;
pub use processing::YamlProcessing;

use crate::models::Project;
use std::path::Path;

pub struct YamlService {
    management: YamlManagement,
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
    
}