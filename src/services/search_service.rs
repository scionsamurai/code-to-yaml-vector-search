// src/services/search_service.rs
use crate::models::Project;
use crate::services::embedding_service::EmbeddingService;
use crate::services::qdrant_service::QdrantService;
use crate::services::llm_service::LlmService;
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
        
        // Get LLM-based file recommendations
        let llm_recommendations = self.get_llm_recommendations(query_text, project).await;
        
        // Combine vector-based and LLM-based results
        let mut combined_results = similar_files.clone();
        
        // Add LLM recommended files that aren't already in the vector results
        if let Ok(recommendations) = llm_recommendations {
            for (file_path, yaml_content, score) in recommendations {
                if !combined_results.iter().any(|(path, _, _)| path == &file_path) {
                    combined_results.push((file_path, yaml_content, score));
                }
            }
        }
        
        // Sort by score (descending) for final ranking
        combined_results.sort_by(|(_, _, score1), (_, _, score2)| 
            score2.partial_cmp(score1).unwrap_or(std::cmp::Ordering::Equal)
        );
        
        // Update project with query and results
        if project.saved_queries.is_none() {
            project.saved_queries = Some(Vec::new());
        }
        
        let query_result = serde_json::json!({
            "query": query_text,
            "results": combined_results
        });
        
        if let Some(saved_queries) = &mut project.saved_queries {
            saved_queries.push(query_result);
        }
        
        Ok(combined_results)
    }

    async fn get_llm_recommendations(&self, query_text: &str, project: &Project) -> Result<Vec<(String, String, f32)>, String> {
        // Initialize LLM service
        let llm_service = LlmService::new();
        let llm_model = project.model.clone();
        
        // Extract file information from project
        let mut file_info = String::new();
        
        // Format the file information to include in the promptb
        for (file_path, description) in project.file_descriptions.iter() {
            file_info.push_str(&format!("- {}: {}\n", file_path, description));
        }
        
        // Create the prompt for the LLM
        let prompt = format!(
            "Based on the user query: \"{}\", which of the following files would be most relevant? \
            Rank the top 3 files by relevance and explain why they're relevant to the query. \
            Format your response as a JSON array of objects with the fields 'file_path', 'reason', and 'relevance_score' \
            (a number between 0 and 1). Only include the JSON in your response.\n\n\
            Available files:\n{}", 
            query_text, 
            file_info
        );
        
        // Get LLM analysis
        let llm_response = llm_service.get_analysis(&prompt, &llm_model).await;
        
        // Parse the LLM response to extract file recommendations
        let mut recommendations = Vec::new();
        
        match serde_json::from_str::<Vec<serde_json::Value>>(&llm_response) {
            Ok(json_array) => {
                for item in json_array {
                    if let (Some(file_path), Some(score)) = (
                        item.get("file_path").and_then(|v| v.as_str()),
                        item.get("relevance_score").and_then(|v| v.as_f64())
                    ) {
                        // Look up the YAML content for this file
                        if let Some(yaml_content) = project.embeddings.get(file_path) {
                            recommendations.push((
                                file_path.to_string(),
                                yaml_content.to_string(),
                                score as f32
                            ));
                        }
                    }
                }
            },
            Err(_) => {
                // Fallback if LLM doesn't return valid JSON
                return Err("Failed to parse LLM recommendations".to_string());
            }
        }
        
        Ok(recommendations)
    }
}