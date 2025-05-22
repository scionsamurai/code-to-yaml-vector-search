// src/services/file/mod.rs
pub mod validation;
pub mod reading;
pub mod update_checker;
pub mod extract_imports;

use crate::models::{Project, ProjectFile};

pub struct FileService;

impl FileService {
    pub fn extract_imports(&self, file_content: &str, language: &str) -> (Vec<String>, String) {
        extract_imports::extract_imports(file_content, language)
    }

    pub fn validate_file_paths(&self, project: &Project) -> Vec<(String, bool)> {
        validation::validate_file_paths(project)
    }

    pub fn read_project_files(&self, project: &Project) -> Vec<ProjectFile> {
        reading::read_project_files(project)
    }

    pub fn project_needs_update(&self, project: &Project, output_dir: &str) -> bool {
        update_checker::project_needs_update(project, output_dir)
    }

    pub fn read_specific_file(&self, project: &Project, file_path: &str) -> Option<String> {
        reading::read_specific_file(project, file_path)
    }

    pub fn needs_yaml_update(&self, source_path: &str, yaml_path: &str) -> bool {
        update_checker::needs_yaml_update(source_path, yaml_path)
    }
    
}