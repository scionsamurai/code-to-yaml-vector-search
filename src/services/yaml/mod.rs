mod management;
mod processing;

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

    // Delegate save_yaml_files to YamlManagement
    pub async fn save_yaml_files(&self, project: &mut Project, output_dir: &str) {
        self.management.save_yaml_files(project, output_dir).await;
    }

    // Delegate check_and_update_yaml_files to YamlManagement
    pub async fn check_and_update_yaml_files(&self, project: &mut Project, output_dir: &str) {
        self.management.check_and_update_yaml_files(project, output_dir).await;
    }

    // Delegate process_yaml_files to YamlProcessing
    pub fn process_yaml_files(&self, output_dir: &Path, project_name: &str, project: &mut Project)
        -> Result<(String, Vec<(String, String)>, bool, Vec<String>), String>
    {
        self.processing.process_yaml_files(output_dir, project_name, project)
    }

    // Delegate clean_up_orphaned_files to YamlManagement
    pub fn clean_up_orphaned_files(&self, project_name: &str, orphaned_files: Vec<String>) {
        self.management.clean_up_orphaned_files(project_name, orphaned_files);
    }
}