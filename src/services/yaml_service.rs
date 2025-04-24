use crate::models::{ProjectFile, Project, EmbeddingMetadata};
use crate::services::file_service::FileService;
use crate::services::llm_service::LlmService;
use crate::services::embedding_service::EmbeddingService;
use crate::services::qdrant_service::QdrantService;
use std::fs::write;
use std::path::Path;
use std::env;

pub struct YamlService {
    file_service: FileService,
    llm_service: LlmService,
}

impl YamlService {
    pub fn new() -> Self {
        Self {
            file_service: FileService {},
            llm_service: LlmService {},
        }
    }

    pub async fn save_yaml_files(&self, project: &mut Project, output_dir: &str) {

        let output_path = Path::new(output_dir).join(&project.name);
        std::fs::create_dir_all(&output_path).unwrap();
    
        let embedding_service = EmbeddingService::new();
        let qdrant_server_url = env::var("QDRANT_SERVER_URL").unwrap();
        let qdrant_service = QdrantService::new(&qdrant_server_url, 1536).await.unwrap();
        
        // Create collection for this project
        qdrant_service.create_project_collection(&project.name).await.unwrap();
    
        let files = self.file_service.read_project_files(&project);
    
        for file in files {
            println!("Checking if yaml update needed for {}", &file.path);
            let source_path = &file.path;
            let yaml_path = output_path.join(format!("{}.yml", file.path.replace("/", "*")));
            if self.file_service.needs_yaml_update(&source_path, &yaml_path.display().to_string()) {
                // Convert to YAML
                let yaml_content = self.llm_service.convert_to_yaml(&file, &project.model).await;
                
                // Write YAML to file
                write(&yaml_path, &yaml_content).unwrap();
                
                // Generate embedding
                match embedding_service.generate_embedding(&yaml_content, Some(1536)).await {
                    Ok(embedding) => {
                        // Store embedding
                        let vector_id = qdrant_service.store_file_embedding(
                            &project.name,
                            &source_path,
                            &yaml_content,
                            embedding
                        ).await.unwrap();
                        
                        // Update project embeddings metadata
                        let metadata = EmbeddingMetadata {
                            file_path: source_path.clone(),
                            last_updated: chrono::Utc::now(),
                            vector_id,
                        };
                        project.embeddings.insert(source_path.clone(), metadata);
                    },
                    Err(e) => eprintln!("Failed to generate embedding: {}", e),
                }
            }
        }
    
        // Save updated project metadata
        let project_settings_path = Path::new(output_dir).join(&project.name).join("project_settings.json");
        let project_settings_json = serde_json::to_string_pretty(&project).unwrap();
        write(project_settings_path, project_settings_json).unwrap();
    }
    

    pub async fn check_and_update_yaml_files(&self, project: &mut Project, output_dir: &str) {
        let output_path = Path::new(output_dir).join(&project.name);
        
        // Check if the output directory exists
        if !output_path.exists() {
            println!("Output directory does not exist: {:?}", output_path);
            return;
        }
        
        let embedding_service = EmbeddingService::new();
        let qdrant_server_url = env::var("QDRANT_SERVER_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
        
        let qdrant_service = match QdrantService::new(&qdrant_server_url, 1536).await {
            Ok(service) => service,
            Err(e) => {
                eprintln!("Failed to connect to Qdrant: {}", e);
                return;
            }
        };
        
        // Create collection for this project if it doesn't exist
        if let Err(e) = qdrant_service.create_project_collection(&project.name).await {
            eprintln!("Failed to create collection: {}", e);
            return;
        }
        
        // Check all YAML files in the project directory
        let mut any_updates = false;
        
        if let Ok(entries) = std::fs::read_dir(&output_path) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                
                // Only process YAML files
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("yml") {
                    let file_path = path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.replace("*", "/").replace(".yml", ""))
                        .unwrap_or_default();
                    
                    // Skip if this is not a code file
                    if file_path.is_empty() || file_path == "project_settings" {
                        continue;
                    }
                    
                    // Check if this YAML needs to be re-embedded
                    let needs_update = match project.embeddings.get(&file_path) {
                        Some(metadata) => {
                            // Check if YAML file is newer than our last recorded update
                            if let Ok(yaml_metadata) = std::fs::metadata(&path) {
                                if let Ok(modified) = yaml_metadata.modified() {
                                    let modified_datetime: chrono::DateTime<chrono::Utc> = modified.into();
                                    modified_datetime > metadata.last_updated
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        },
                        None => true, // No embedding record exists
                    };
                    
                    if needs_update {
                        println!("Detected manually updated YAML: {}", file_path);
                        
                        // Read the YAML content
                        if let Ok(yaml_content) = std::fs::read_to_string(&path) {
                            // Generate new embedding
                            match embedding_service.generate_embedding(&yaml_content, Some(1536)).await {
                                Ok(embedding) => {
                                    // Update in Qdrant
                                    match qdrant_service.store_file_embedding(
                                        &project.name,
                                        &file_path,
                                        &yaml_content,
                                        embedding
                                    ).await {
                                        Ok(vector_id) => {
                                            // Update project metadata
                                            let metadata = crate::models::EmbeddingMetadata {
                                                file_path: file_path.clone(),
                                                last_updated: chrono::Utc::now(),
                                                vector_id,
                                            };
                                            project.embeddings.insert(file_path, metadata);
                                            any_updates = true;
                                        },
                                        Err(e) => eprintln!("Failed to store embedding: {}", e),
                                    }
                                },
                                Err(e) => eprintln!("Failed to generate embedding: {}", e),
                            }
                        }
                    }
                }
            }
        }
        
        // Save updated project metadata if any changes were made
        if any_updates {
            let project_settings_path = output_path.join("project_settings.json");
            let project_settings_json = serde_json::to_string_pretty(&project).unwrap();
            if let Err(e) = write(&project_settings_path, project_settings_json) {
                eprintln!("Failed to update project settings: {}", e);
            }
        }
    }
        
}