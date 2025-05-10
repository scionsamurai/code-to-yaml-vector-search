// src/services/yaml/management/cleanup.rs
use crate::services::qdrant_service::QdrantService;
use std::env;

pub fn clean_up_orphaned_files(project_name: &str, orphaned_files: Vec<String>) {
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