// src/services/search_service.rs
use crate::models::Project;
use crate::services::embedding_service::EmbeddingService;
use crate::services::qdrant_service::QdrantService;
use crate::services::llm_service::LlmService;
use crate::services::file_service::FileService;
use std::env;

pub struct SearchService;

impl SearchService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn search_project(&self, project: &mut Project, query_text: &str) -> Result<(Vec<(String, String, f32)>, String), String> {
        // Generate embedding for query
        let embedding_service = EmbeddingService::new();
        let query_embedding = match embedding_service.generate_embedding(query_text, None).await {
            Ok(embedding) => embedding,
            Err(e) => return Err(e.to_string()),
        };
        
        // Search for similar files using vector search
        let qdrant_server_url = env::var("QDRANT_SERVER_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
        let qdrant_service = match QdrantService::new(&qdrant_server_url, 1536).await {
            Ok(service) => service,
            Err(e) => return Err(format!("Failed to connect to Qdrant: {}", e)),
        };
        
        let similar_files = match qdrant_service.search_similar_files(&project.name, query_embedding, 5).await {
            Ok(files) => files,
            Err(e) => return Err(e.to_string()),
        };
        
        // Get LLM recommendations based on search results
        let llm_analysis = self.get_llm_analysis(query_text, &similar_files, project).await?;
        
        // Update project with query, vector results, and LLM analysis
        if project.saved_queries.is_none() {
            project.saved_queries = Some(Vec::new());
        }
        
        let query_result = serde_json::json!({
            "query": query_text,
            "vector_results": similar_files,
            "llm_analysis": llm_analysis
        });
        
        if let Some(saved_queries) = &mut project.saved_queries {
            saved_queries.push(query_result);
        }
        
        Ok((similar_files, llm_analysis))
    }

    async fn get_llm_analysis(&self, query_text: &str, similar_files: &[(String, String, f32)], project: &Project) -> Result<String, String> {
        // Initialize LLM service and file service
        let llm_service = LlmService::new();
        let file_service = FileService {};
        let llm_model = project.model.clone();
        
        // Extract code from similar files
        let mut file_code = String::new();
        for (file_path, _, _) in similar_files {
            // Use the targeted file reading method
            match file_service.read_specific_file(project, file_path) {
                Some(content) => {
                    file_code.push_str(&format!("// File: {}\n{}\n\n", file_path, content));
                },
                None => {
                    // If content can't be found, note that in the analysis prompt
                    file_code.push_str(&format!("// File: {} (content not available)\n\n", file_path));
                }
            }
        }
        let file_descriptions = project.file_descriptions.iter()
        .map(|(path, desc)| format!("{}: {}", path, desc))
        .collect::<Vec<String>>()
        .join("\n");
        // Create the prompt for the LLM
        let prompt = format!(
            "User Query: \"{}\"\n\n\
            File Descriptions:\n\
            ```\n{}\n```\n\n\
            Related code from vector search:\n\
            ```\n{}\n```\n\n\
            Based on the user query and the provided code: Were the vector results accurate? What other files or components would be needed to fully answer this query? DO NOT RESPOND WITH ANY CODE SNIPPETS. The purpose of this step is to identify files important to the update and suggest a few query alternatives. On that note, please suggest a few alternative queries that would be useful to explore the codebase more precisely for the desired outcome. Remember that i am saying query, but it is more of a user request about code and doesn't always have the form of a question. And remember the query should be the same but more verbose and potentially more grammatically correct.",
            query_text,
            file_descriptions,
            file_code
        );
        
        // Get LLM analysis
        let llm_response = llm_service.get_analysis(&prompt, &llm_model).await;
        
        Ok(llm_response)
    }

}
