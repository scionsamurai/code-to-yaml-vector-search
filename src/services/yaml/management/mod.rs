// src/services/yaml/management/mod.rs
use crate::services::file::FileService;
use crate::services::llm_service::{LlmService, LlmServiceConfig}; // Import LlmServiceConfig
use crate::models::{
    Project,
    ProjectFile,
};
use std::path::Path;
pub mod generation;
pub mod embedding;
pub mod cleanup;
// REMOVED: use crate::services::utils::html_utils::unescape_html; // Not needed if llm_service returns raw YAML
use crate::services::embedding_service::EmbeddingService;
use crate::services::qdrant_service::QdrantService;
use crate::services::yaml::FileYamlData;
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
        project_file: &ProjectFile,
        provider: &str,
        chat_model: Option<&str>, // Existing specific_model, now conceptually chat_model
        yaml_model: Option<&str>, // New parameter for YAML model
        llm_config: Option<LlmServiceConfig>, // Config now comes in directly
    ) -> Option<String> {

        // Pass both chat_model (specific_model) and yaml_model and llm_config to llm_service.convert_to_yaml
        let yaml_content_result = self.llm_service.convert_to_yaml(&project_file, provider, chat_model, yaml_model, llm_config).await;

        let language = Path::new(&project_file.path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match yaml_content_result {
            Ok(yaml_content) => {
                let (imports, _) = self.file_service.extract_imports(&project_file.content, language);

                let mut combined_content = yaml_content; // This is the raw, valid YAML from LLM

                if !imports.is_empty() {
                    let imports_string = imports.join("\n  - ");
                    // Combine raw YAML with imports
                    combined_content = format!("{}\n\nimports:\n  - {}", combined_content, imports_string);
                }
                // Return the final raw YAML string. Escaping for HTML should be done by consumers if needed.
                Some(combined_content)
            },
            Err(e) => {
                eprintln!("Failed to generate or validate YAML for file {}: {}", project_file.path, e);
                None // Return None if all attempts fail or initial prompt setup fails
            }
        }
    }
    
    pub fn get_parsed_yaml_for_file_sync(
        &self,
        project: &Project,
        source_file_path: &str, // Original source file path (e.g., "src/main.rs")
        output_dir: &Path, // The base output directory (e.g., "target")
    ) -> Result<FileYamlData, String> {
        let yaml_file_name = source_file_path.replace("/", "*");
        let yaml_file_path = output_dir.join(&project.name).join(yaml_file_name.clone());

        let file_content = self.file_service.read_specific_file(&project, &yaml_file_path.to_string_lossy()); // READ THE YAML FILE, not the source file
        
        let file_content = file_content.ok_or_else(|| {
            format!(
                "Failed to read YAML file {} (source {}): {}",
                yaml_file_path.display(),
                source_file_path,
                "file not found"
            )
        })?;
        let yaml_data: FileYamlData = serde_yaml::from_str(&file_content)
            .map_err(|e| format!("Failed to parse YAML file {}: {}", yaml_file_path.display(), e))?;
        // if parsing fails, print the content for debugging
        if yaml_data.description.is_empty() && yaml_data.description.is_empty() { // This condition looks incorrect. Likely meant to check other fields
            eprintln!("Debug: YAML content of file {} is empty or missing expected fields:\n{}", yaml_file_path.display(), file_content);
        }

        Ok(yaml_data)
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

        let mut current_blob_hash: Option<String> = None;
        let source_file_abs_path = Path::new(file_path);

        let repo_result = if project.git_integration_enabled {
            crate::services::git_service::GitService::open_repository(Path::new(&project.source_dir))
        } else {
            Err(crate::services::git_service::GitError::Other("Git integration not enabled".to_string()))
        };

        if project.git_integration_enabled && repo_result.is_ok() {
            if let Ok(repo) = repo_result {
                match crate::services::git_service::GitService::get_blob_hash(&repo, &source_file_abs_path) {
                    Ok(hash) => {
                        current_blob_hash = Some(hash);
                    },
                    Err(e) => {
                        eprintln!("Failed to get Git blob hash for {:?}: {}. Proceeding without it.", source_file_abs_path, e);
                    }
                }
            }
        }

        embedding::process_embedding(&embedding_service, &qdrant_service, project, file_path, &content_to_embed, current_blob_hash).await;
    }

}