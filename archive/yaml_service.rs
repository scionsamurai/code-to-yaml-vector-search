use crate::models::{Project, EmbeddingMetadata};
use crate::services::file_service::FileService;
use crate::services::llm_service::LlmService;
use crate::services::embedding_service::EmbeddingService;
use crate::services::qdrant_service::QdrantService;
use std::path::Path;
use std::fs::{read_dir, read_to_string, write};
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
        let qdrant_server_url = env::var("QDRANT_SERVER_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
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
        
    pub fn process_yaml_files(&self, output_dir: &Path, project_name: &str, project: &mut Project) 
        -> Result<(String, Vec<(String, String)>, bool, Vec<String>), String> {
        let mut file_descriptions: Vec<(String, String)> = Vec::new();
        let mut cleanup_needed = false;
        let mut orphaned_files = Vec::new();
        
        let yaml_html = read_dir(output_dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?
            .filter_map(|entry| {
                self.process_yaml_entry(
                    entry, 
                    project, 
                    &mut file_descriptions, 
                    &mut orphaned_files, 
                    &mut cleanup_needed,
                    project_name
                )
            })
            .collect::<Result<Vec<_>, String>>()?
            .join("");
            
        Ok((yaml_html, file_descriptions, cleanup_needed, orphaned_files))
    }

    fn process_yaml_entry(&self, 
                        entry: Result<std::fs::DirEntry, std::io::Error>, 
                        project: &mut Project,
                        file_descriptions: &mut Vec<(String, String)>,
                        orphaned_files: &mut Vec<String>,
                        cleanup_needed: &mut bool,
                        project_name: &str) -> Option<Result<String, String>> {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e)).ok()?;
        let yaml_path = entry.path();
        
        // Skip project_settings.json
        if yaml_path.file_name().unwrap().to_string_lossy() == "project_settings.json" {
            return Some(Ok(String::new()));
        }
        
        // Check if file is a YAML file
        if yaml_path.extension().and_then(|ext| ext.to_str()) != Some("yml") {
            return Some(Ok(String::new()));
        }
        
        // Extract the original source file path
        let file_name = yaml_path.file_name()?.to_string_lossy();
        let source_path = file_name
            .replace(".yml", "")
            .replace("*", "/");
        
        // Check if source file exists
        let original_source_path = Path::new(&project.source_dir).join(&source_path);
        if !original_source_path.exists() {
            // Source file doesn't exist, mark it for cleanup
            orphaned_files.push(source_path.clone());
            
            // Remove the YAML file
            if let Err(e) = std::fs::remove_file(&yaml_path) {
                eprintln!("Failed to remove orphaned YAML file {}: {}", yaml_path.display(), e);
            }
            
            // Remove from embeddings in project settings
            if project.embeddings.remove(&source_path).is_some() {
                *cleanup_needed = true;
            }
            
            return Some(Ok(String::new()));
        }
        
        // Process existing file
        match self.process_yaml_file(&yaml_path, &source_path, file_descriptions, project_name) {
            Ok(html) => Some(Ok(html)),
            Err(e) => Some(Err(e))
        }
    }

    fn process_yaml_file(&self, 
                        yaml_path: &Path, 
                        source_path: &str,
                        file_descriptions: &mut Vec<(String, String)>,
                        project_name: &str) -> Result<String, String> {
        // Read file content
        let content = read_to_string(yaml_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        // Extract description
        let description = self.parse_description(&content)
            .unwrap_or_else(|| "No description.".to_string())
            .replace(|c: char| c == '\n' || c == '\r', " ")
            .trim()
            .to_string();
        
        // Store file description
        file_descriptions.push((source_path.to_string(), description.clone()));
        
        // Return HTML for this file
        Ok(format!(
            "<div class=\"page\"><p>---</p><h3>path: {}</h3><pre>{}</pre><button onclick=\"regenerate('{}', '{}')\">Regenerate</button></div>",
            source_path,
            content.replace("---\n", "").replace("```", ""),
            project_name,
            yaml_path.display()
        ))
    }

    pub fn clean_up_orphaned_files(&self, project_name: &str, orphaned_files: Vec<String>) {
        let qdrant_server_url = env::var("QDRANT_SERVER_URL")
            .unwrap_or_else(|_| "http://localhost:6334".to_string());
            
        // Clone project_name to own the data for the async task
        let project_name_owned = project_name.to_string();
        
        // Spawn a task to clean up vectors
        tokio::spawn(async move {
            match QdrantService::new(&qdrant_server_url, 1536).await {
                Ok(qdrant_service) => {
                    for file_path in orphaned_files {
                        match qdrant_service.delete_file_vectors(&project_name_owned, &file_path).await {
                            Ok(_) => println!("Removed vector for orphaned file: {}", file_path),
                            Err(e) => eprintln!("Failed to remove vector for {}: {}", file_path, e),
                        }
                    }
                },
                Err(e) => eprintln!("Failed to connect to Qdrant for cleanup: {}", e),
            }
        });
    }

    pub fn parse_description(&self, content: &str) -> Option<String> {
        let mut lines = content.lines();
        // 1) Must start with '---'
        if lines.next()? != "---" { return None; }

        let mut in_block = false;
        let mut desc = String::new();

        for line in lines {
            let trimmed = line.trim_start();

            // 2) if we hit the end of front-matter, stop
            if trimmed == "---" {
                break;
            }

            if !in_block {
                // 3) look for the `description:` key at top-level
                if let Some(rest) = trimmed.strip_prefix("description:") {
                    let rest = rest.trim();
                    match rest.chars().next() {
                        // block scalar start
                        Some('|') | Some('>') => {
                            in_block = true;
                            continue;
                        }
                        // inline scalar on the same line
                        _ if !rest.is_empty() => {
                            // strip optional quotes
                            let s = rest.trim_matches('"').to_string();
                            return Some(s);
                        }
                        // exactly `description:` with no value → treat as block
                        _ => {
                            in_block = true;
                            continue;
                        }
                    }
                }
            } else {
                // 4) we're inside a block — collect indented lines
                // YAML spec: block-scalar content must be indented at least one space
                if line.starts_with(' ') || line.starts_with('\t') {
                    // drop only the leading indent
                    desc.push_str(line.trim_start());
                    desc.push('\n');
                } else {
                    // non-indented → end of block
                    break;
                }
            }
        }

        if desc.is_empty() {
            None
        } else {
            // trim the final newline
            Some(desc.trim_end().to_string())
        }
    }
}