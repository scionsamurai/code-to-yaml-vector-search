// src/services/embedding_service.rs
use std::error::Error;

pub struct EmbeddingService {}

impl EmbeddingService {
    pub fn new() -> Self {
        EmbeddingService {}
    }
    
    pub async fn generate_embedding(
        &self, 
        content: &str, 
        dimensions: Option<u32>
    ) -> Result<Vec<f32>, Box<dyn Error + Send + Sync>> {
        // Use the correct function from the llm_api_access crate
        llm_api_access::openai::get_embedding(content.to_string(), dimensions).await
    }
}