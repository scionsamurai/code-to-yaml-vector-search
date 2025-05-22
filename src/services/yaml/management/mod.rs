// src/services/yaml/management/mod.rs
use crate::services::file::FileService;
use crate::services::llm_service::LlmService;
use crate::models::{
    Project,
    ProjectFile,
};
use std::path::Path;
pub mod generation;
pub mod embedding;
pub mod cleanup;
use crate::routes::llm::chat_analysis::utils::unescape_html;

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

    pub async fn create_yaml_with_imports(
        &self, // Added self
        project_file: &ProjectFile,
        model: &str,
    ) -> String {
        let yaml_content = self.llm_service.convert_to_yaml(&project_file, model).await;

        let language = Path::new(&project_file.path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let (imports, _) = self.file_service.extract_imports(&project_file.content, language);

        let yaml_content = unescape_html(yaml_content);

        if !imports.is_empty() {
            let imports_string = imports.join("\n\t- ");
            format!("{}\n\nimports:\n\t- {}", yaml_content, imports_string)
        } else {
            yaml_content
        }
    }
    
    // Move these functions from the standalone to be methods
    pub async fn generate_yaml_files(&self, project: &mut Project, output_dir: &str, force: bool) {
        generation::generate_yaml_files(self, project, output_dir, force).await;
    }

    pub async fn check_and_update_yaml_embeddings(&self, project: &mut Project, output_dir: &str) {
        embedding::check_and_update_yaml_embeddings(project, output_dir).await;
    }

    pub fn clean_up_orphaned_files(&self, project_name: &str, orphaned_files: Vec<String>) {
        cleanup::clean_up_orphaned_files(project_name, orphaned_files);
    }

}