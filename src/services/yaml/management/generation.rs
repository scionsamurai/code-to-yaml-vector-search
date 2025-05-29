// src/services/yaml/management/generation.rs
use crate::services::yaml::management::{YamlManagement, embedding};
use crate::models::Project;
use crate::services::embedding_service::EmbeddingService;
use crate::services::qdrant_service::QdrantService;
use std::path::Path;
use std::fs::write;
use std::env;

pub async fn generate_yaml_files(yaml_management: &YamlManagement, project: &mut Project, output_dir: &str, force: bool) {
    let output_path = Path::new(output_dir).join(&project.name);
    std::fs::create_dir_all(&output_path).unwrap();

    let embedding_service = EmbeddingService::new();
    let qdrant_server_url = env::var("QDRANT_SERVER_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let qdrant_service = QdrantService::new(&qdrant_server_url, 1536).await.unwrap();

    // Create collection for this project
    qdrant_service.create_project_collection(&project.name).await.unwrap();

    let files = yaml_management.file_service.read_project_files(&project);

    for file in files {
        println!("Checking if yaml update needed for {}", &file.path);
        let source_path = &file.path;
        let yaml_path = output_path.join(format!("{}.yml", file.path.replace("/", "*")));
        let use_yaml = project.file_yaml_override.get(&source_path.clone()).map(|&b| b).unwrap_or(project.default_use_yaml);

        if use_yaml && (force || yaml_management.file_service.needs_yaml_update(&source_path, &yaml_path.display().to_string())) {

            let combined_content = yaml_management.create_yaml_with_imports(project, &file, &project.model).await;

            // Write YAML to file
            write(&yaml_path, combined_content.clone().unwrap()).unwrap();

            // Generate and store embedding
            embedding::process_embedding(&embedding_service, &qdrant_service, project, &source_path, &combined_content.unwrap()).await;
        }
    }

    // Save updated project metadata
    let project_settings_path = Path::new(output_dir).join(&project.name).join("project_settings.json");
    let project_settings_json = serde_json::to_string_pretty(&project).unwrap();
    write(project_settings_path, project_settings_json).unwrap();
}