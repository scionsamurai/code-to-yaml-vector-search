// src/services/search_service.rs
use crate::models::Project;
use crate::services::embedding_service::EmbeddingService;
use crate::services::qdrant_service::QdrantService;
use std::env;

pub struct SearchService;

impl SearchService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn search_project(&self, project: &mut Project, query_text: &str) -> Result<Vec<(String, String, f32)>, String> {
        // Generate embedding for query
        let embedding_service = EmbeddingService::new();
        let query_embedding = match embedding_service.generate_embedding(query_text, None).await {
            Ok(embedding) => embedding,
            Err(e) => return Err(e.to_string()),
        };
        
        // Search for similar files
        let qdrant_server_url = env::var("QDRANT_SERVER_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
        let qdrant_service = match QdrantService::new(&qdrant_server_url, 1536).await {
            Ok(service) => service,
            Err(e) => return Err(format!("Failed to connect to Qdrant: {}", e)),
        };
        
        let similar_files = match qdrant_service.search_similar_files(&project.name, query_embedding, 5).await {
            Ok(files) => files,
            Err(e) => return Err(e.to_string()),
        };
        
        // Update project with query and results
        if project.saved_queries.is_none() {
            project.saved_queries = Some(Vec::new());
        }
        
        let query_result = serde_json::json!({
            "query": query_text,
            "results": similar_files
        });
        
        if let Some(saved_queries) = &mut project.saved_queries {
            saved_queries.push(query_result);
        }
        
        Ok(similar_files)
    }
}