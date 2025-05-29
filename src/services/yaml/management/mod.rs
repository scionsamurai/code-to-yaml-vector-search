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
use crate::services::embedding_service::EmbeddingService;
use crate::services::qdrant_service::QdrantService;
use std::env;

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
        project: &Project,
        project_file: &ProjectFile,
        model: &str,
    ) -> Option<String> {

        let use_yaml = project.file_yaml_override.get(&project_file.path).map(|&b| b).unwrap_or(project.default_use_yaml);

        if !use_yaml {
            return Some(project_file.content.clone())
        }

        let yaml_content = self.llm_service.convert_to_yaml(&project_file, model).await;

        let language = Path::new(&project_file.path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let (imports, _) = self.file_service.extract_imports(&project_file.content, language);

        let yaml_content = unescape_html(yaml_content);

        if !imports.is_empty() {
            let imports_string = imports.join("\n\t- ");
            Some(format!("{}\n\nimports:\n\t- {}", yaml_content, imports_string))
        } else {
            Some(yaml_content)
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

    pub async fn regenerate_embedding(&self, project: &mut Project, file_path: &str, output_dir: &str) {
        let embedding_service = EmbeddingService::new();
        let qdrant_server_url = env::var("QDRANT_SERVER_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());

        let qdrant_service = match QdrantService::new(&qdrant_server_url, 1536).await {
            Ok(service) => service,
            Err(e) => {
                eprintln!("Failed to connect to Qdrant: {}", e);
                return;
            }
        };

        // **Delete existing embedding**
        if let Err(e) = qdrant_service.delete_file_vectors(&project.name, file_path).await {
            eprintln!("Failed to delete existing vectors: {}", e);
            // Consider whether to return early here, depending on your error handling policy
            // If deleting the old embedding fails, it might be best to avoid creating a new one
            return;
        }

        let output_path = Path::new(output_dir).join(&project.name);
        let yaml_path = output_path.join(format!("{}.yml", file_path.replace("/", "*")));
        let use_yaml = project.file_yaml_override.get(file_path).map(|&b| b).unwrap_or(project.default_use_yaml);

        let content_to_embed: String;
        if use_yaml {
            // Read the YAML content
            content_to_embed = match std::fs::read_to_string(&yaml_path) {
                Ok(yaml_content) => yaml_content,
                Err(e) => {
                    eprintln!("Error reading YAML file: {}", e);
                    return;
                }
            };
        } else {
            // Read the original source file content
            content_to_embed = match std::fs::read_to_string(file_path) {
                Ok(source_content) => source_content,
                Err(e) => {
                    eprintln!("Error reading original source file: {}", e);
                    return;
                }
            };
        }

        // **Generate and store the new embedding**
        embedding::process_embedding(&embedding_service, &qdrant_service, project, file_path, &content_to_embed).await;
    }

}