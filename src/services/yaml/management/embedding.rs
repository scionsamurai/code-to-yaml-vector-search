// src/services/yaml/management/embedding.rs
use crate::models::EmbeddingMetadata;
use crate::services::embedding_service::EmbeddingService;
use crate::services::qdrant_service::QdrantService;
use crate::models::Project;
use std::path::Path;
use std::env;
use std::fs::write;
use crate::services::git_service::GitService; // Import GitService
use git2::Repository; // Import Repository

pub async fn process_embedding(
    embedding_service: &EmbeddingService,
    qdrant_service: &QdrantService,
    project: &mut Project,
    source_path: &str,
    content_to_embed: &String, // Renamed from yaml_content for clarity, as it can be source or yaml
    git_blob_hash: Option<String>, // Add this parameter
) {
    match embedding_service.generate_embedding(content_to_embed, Some(1536)).await {
        Ok(embedding) => {
            // Store embedding
            let vector_id = qdrant_service.store_file_embedding(
                &project.name,
                source_path,
                content_to_embed,
                embedding
            ).await.unwrap();

            // Update project embeddings metadata
            let metadata = EmbeddingMetadata {
                file_path: source_path.to_string(),
                last_updated: chrono::Utc::now(),
                vector_id,
                git_blob_hash, // Store the blob hash
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

    // Open the repo once if git integration is enabled
    let repo_result = if project.git_integration_enabled {
        GitService::open_repository(Path::new(&project.source_dir))
    } else {
        Err(crate::services::git_service::GitError::Other("Git integration not enabled".to_string()))
    };
    
    let mut any_updates = false;
    
    if let Ok(entries) = std::fs::read_dir(&output_path) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path(); // Path to the YAML file
            
            // Only process YAML files
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("yml") {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();
                
                // Convert YAML filename back to original source file path format
                let source_file_path_str = file_name.replace("*", "/").replace(".yml", "");
                let source_file_path = Path::new(&source_file_path_str);

                // Skip if this is not a code file (e.g., project_settings.json or a malformed name)
                if source_file_path_str.is_empty() || file_name == "project_settings.json" {
                    continue;
                }

                let use_yaml = project.file_yaml_override.get(&source_file_path_str).map(|&b| b).unwrap_or(project.default_use_yaml);

                let mut needs_update = false;
                let mut current_blob_hash: Option<String> = None;

                // Determine if Git tracking is active and repository is successfully opened
                let use_git_tracking = project.git_integration_enabled && repo_result.is_ok();

                if use_git_tracking {
                    let repo_ref = repo_result.as_ref().unwrap();
                    match GitService::get_blob_hash(repo_ref, source_file_path) {
                        Ok(hash) => {
                            current_blob_hash = Some(hash.clone());
                            if let Some(metadata_entry) = project.embeddings.get(&source_file_path_str) {
                                if let Some(stored_hash) = &metadata_entry.git_blob_hash {
                                    if stored_hash != &hash {
                                        needs_update = true; // Git content changed
                                    }
                                } else {
                                    // Git enabled, but no hash stored - needs update to bootstrap
                                    needs_update = true;
                                }
                            } else {
                                // No metadata entry - needs update
                                needs_update = true;
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to get Git blob hash for {:?}: {}. Falling back to timestamp.", source_file_path, e);
                            // Fallback to timestamp logic
                            needs_update = check_timestamp_update_logic(project, &source_file_path_str, &path, use_yaml);
                        }
                    }
                } else {
                    // Git not enabled or repo not available - use timestamp logic
                    needs_update = check_timestamp_update_logic(project, &source_file_path_str, &path, use_yaml);
                }
                
                if needs_update {
                    println!("Detected update needed for: {}", source_file_path_str);

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
                        // Read the original source file content
                        content_to_embed = match std::fs::read_to_string(source_file_path) {
                            Ok(source_content) => source_content,
                            Err(e) => {
                                eprintln!("Error reading original source file: {}", e);
                                continue; // Skip this file
                            }
                        };
                    }

                    // Generate and Store embedding, passing the current_blob_hash
                    process_embedding(&embedding_service, &qdrant_service, project, &source_file_path_str, &content_to_embed, current_blob_hash).await;
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

// Helper function for timestamp comparison logic (used as fallback)
fn check_timestamp_update_logic(project: &Project, source_file_path_str: &str, yaml_path: &Path, use_yaml: bool) -> bool {
    let source_metadata = match std::fs::metadata(source_file_path_str) {
        Ok(m) => Some(m),
        Err(e) => {
            eprintln!("Fallback: Failed to get metadata for source file {:?}: {}", source_file_path_str, e);
            return true; // Assume update needed if source metadata is unavailable
        }
    };

    let yaml_metadata = match std::fs::metadata(yaml_path) {
        Ok(m) => Some(m),
        Err(e) => {
            eprintln!("Fallback: Failed to get metadata for YAML file {:?}: {}", yaml_path, e);
            return true; // Assume update needed if YAML metadata is unavailable (already checked by yaml_path.exists() but kept for robustness)
        }
    };

    if let (Some(source_meta), Some(yaml_meta)) = (source_metadata, yaml_metadata) {
        // Compare modified times
        let source_modified = source_meta.modified().unwrap();
        let yaml_modified = yaml_meta.modified().unwrap();
        
        let metadata_entry = project.embeddings.get(source_file_path_str);

        match metadata_entry {
            Some(metadata) => {
                let updated_source_modified: chrono::DateTime<chrono::Utc> = source_modified.into();
                let updated_yaml_modified: chrono::DateTime<chrono::Utc> = yaml_modified.into();

                // If using YAML, compare against YAML file's modification time
                // If not using YAML, compare against source file's modification time
                let comparison_time = if use_yaml { updated_yaml_modified } else { updated_source_modified };

                // If the file itself is newer than when we last recorded it, update
                comparison_time > metadata.last_updated
            }
            None => {
                true // No embedding record exists, so update
            }
        }
    } else {
        true // If any metadata is missing, assume update needed
    }
}