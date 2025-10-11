// src/services/yaml/management/generation.rs
use crate::services::yaml::management::{YamlManagement, embedding};
use crate::models::Project;
use crate::services::embedding_service::EmbeddingService;
use crate::services::qdrant_service::QdrantService;
use std::path::Path;
use std::fs::write;
use std::env;
use crate::services::git_service::GitService; // Import GitService
use git2::Repository; // Import Repository

pub async fn generate_yaml_files(yaml_management: &YamlManagement, project: &mut Project, output_dir: &str, force: bool) {
    let output_path = Path::new(output_dir).join(&project.name);
    std::fs::create_dir_all(&output_path).unwrap();

    let embedding_service = EmbeddingService::new();
    let qdrant_server_url = env::var("QDRANT_SERVER_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let qdrant_service = QdrantService::new(&qdrant_server_url, 1536).await.unwrap();

    // Create collection for this project
    qdrant_service.create_project_collection(&project.name).await.unwrap();

    let files = yaml_management.file_service.read_project_files(&project);

    // Open the repo once if git integration is enabled
    let repo_result = if project.git_integration_enabled {
        GitService::open_repository(Path::new(&project.source_dir))
    } else {
        Err(crate::services::git_service::GitError::Other("Git integration not enabled".to_string()))
    };

    for file in files {
        let source_path_buf = Path::new(&file.path);
        let yaml_path = output_path.join(format!("{}.yml", file.path.replace("/", "*")));
        let use_yaml = project.file_yaml_override.get(&file.path).map(|&b| b).unwrap_or(project.default_use_yaml);

        let file_extension = source_path_buf
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        // Determine if an update is needed using the new `needs_yaml_update` signature
        let needs_update = if force {
            true // Force update overrides all checks
        } else {
            yaml_management.file_service.needs_yaml_update(project, &repo_result, source_path_buf, &yaml_path)
        };
        
        // Get blob hash if git is enabled and repo is open for the current file
        let git_blob_hash_for_file = if repo_result.is_ok() {
            GitService::get_blob_hash(repo_result.as_ref().unwrap(), source_path_buf).ok()
        } else {
            None
        };


        if file_extension == "md" {
            // Handle Markdown files: read content and generate embedding
            println!("Processing Markdown file: {}", &file.path);
            let markdown_content = std::fs::read_to_string(&file.path).unwrap();
            embedding::process_embedding(&embedding_service, &qdrant_service, project, &file.path, &markdown_content, git_blob_hash_for_file.clone()).await;
        } else if use_yaml && needs_update {
            println!("YAML update needed for: {}", &file.path);
            let combined_content = yaml_management.create_yaml_with_imports(&file, &project.provider, project.specific_model.as_deref()).await;

            // Write YAML to file
            write(&yaml_path, combined_content.clone().unwrap()).unwrap();

            // Generate and store embedding, passing the git_blob_hash
            embedding::process_embedding(&embedding_service, &qdrant_service, project, &file.path, &combined_content.unwrap(), git_blob_hash_for_file.clone()).await;
        }
    }

    // Save updated project metadata
    let project_settings_path = Path::new(output_dir).join(&project.name).join("project_settings.json");
    let project_settings_json = serde_json::to_string_pretty(&project).unwrap();
    write(project_settings_path, project_settings_json).unwrap();
}