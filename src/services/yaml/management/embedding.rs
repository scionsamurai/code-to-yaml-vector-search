// src/services/yaml/management/embedding.rs
use crate::models::EmbeddingMetadata;
use crate::services::embedding_service::EmbeddingService;
use crate::services::qdrant_service::QdrantService;
use crate::models::Project;
use std::path::Path;
use std::env;
use std::fs::write;

pub async fn process_embedding(embedding_service: &EmbeddingService, qdrant_service: &QdrantService, project: &mut Project, source_path: &str, yaml_content: &String) {
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
                file_path: source_path.to_string(), // Changed to owned string
                last_updated: chrono::Utc::now(),
                vector_id,
            };
            project.embeddings.insert(source_path.to_string(), metadata);
        },
        Err(e) => eprintln!("Failed to generate embedding: {}", e),
    }
}


pub async fn check_and_update_yaml_embeddings(project: &mut Project, output_dir: &str)  {
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

                // **Check use_yaml setting**
                let use_yaml = project.file_yaml_override.get(&file_path).map(|&b| b).unwrap_or(project.default_use_yaml);


                let needs_update = match project.embeddings.get(&file_path) {
                    Some(metadata) => {
                        // Check the appropriate file based on use_yaml

                        let metadata_path = if use_yaml {
                            path.as_path() // YAML path
                        } else {
                            Path::new(&file_path)
                        };
                        
                        if let Ok(file_metadata) = std::fs::metadata(&metadata_path) {
                            if let Ok(modified) = file_metadata.modified() {
                                let modified_datetime: chrono::DateTime<chrono::Utc> = modified.into();
                                modified_datetime > metadata.last_updated
                            } else {
                                false
                            }
                        } else {
                            // If we cannot get the metadata (file might not exist), force update
                            true
                        }
                    },
                    None => true, // No embedding record exists
                };
                
                if needs_update {
                    println!("Detected manually updated YAML: {}", file_path);

                    let content_to_embed: String;

                    if use_yaml {
                        // Read the YAML content
                        content_to_embed = match std::fs::read_to_string(&path) {
                            Ok(yaml_content) => yaml_content,
                            Err(e) => {
                                eprintln!("Error reading YAML file: {}", e);
                                continue; // Skip this file
                            }
                        };
                    } else {
                        content_to_embed = match std::fs::read_to_string(&file_path) {
                            Ok(source_content) => source_content,
                            Err(e) => {
                                eprintln!("Error reading original source file: {}", e);
                                continue; // Skip this file
                            }
                        };
                    }

                    // Generate and Store embedding
                    process_embedding(&embedding_service, &qdrant_service, project, &file_path, &content_to_embed).await;
                    any_updates = true;
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
