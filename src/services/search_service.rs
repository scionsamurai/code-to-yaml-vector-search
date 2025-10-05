// src/services/search_service.rs
use crate::models::Project;
use crate::services::embedding_service::EmbeddingService;
use crate::services::qdrant_service::QdrantService;
use crate::services::llm_service::LlmService;
use crate::services::file::FileService;
use crate::services::project_service::ProjectService;
use crate::models::QueryData;
use std::env;

pub struct SearchService;

impl SearchService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn search_project(&self, project: &mut Project, query_text: &str, output_dir: Option<&std::path::PathBuf>) -> Result<(Vec<(String, String, f32, std::option::Option<Vec<f32>>)>, String), String> {

        // Generate embedding for query
        let embedding_service = EmbeddingService::new();
        let query_embedding = match embedding_service.generate_embedding(query_text, Some(1536)).await {
            Ok(embedding) => embedding,
            Err(e) => return Err(e.to_string()),
        };
        
        // Search for similar files using vector search
        let qdrant_server_url = env::var("QDRANT_SERVER_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
        let qdrant_service = match QdrantService::new(&qdrant_server_url, 1536).await {
            Ok(service) => service,
            Err(e) => return Err(format!("Failed to connect to Qdrant: {}", e)),
        };
        
        let similar_files = match qdrant_service.search_similar_files(&project.name, query_embedding, 5, false).await {
            Ok(files) => files,
            Err(e) => return Err(e.to_string()),
        };
        
        // Get LLM recommendations based on search results
        let llm_analysis = self.get_llm_analysis(query_text, &similar_files, project).await?;
        
        if output_dir.is_some() {
            let project_service = ProjectService::new();

            // Always create a new file for each query
            let filename = project_service.query_manager.generate_query_filename(); // MODIFIED LINE
            let mut query_data = QueryData::default();

            // Update the QueryData with search results and LLM analysis
            query_data.query = query_text.to_string();
            query_data.vector_results = similar_files.iter().map(|(path, _, score, _)| (path.clone(), *score)).collect();
            query_data.llm_analysis = llm_analysis.clone();

            // Save the updated QueryData
            match project_service.query_manager.save_query_data(&output_dir.unwrap(), &query_data, &filename) { // MODIFIED LINE
                Ok(_) => {
                    println!("Query data saved successfully.");
                }
                Err(e) => {
                    eprintln!("Failed to save query data: {}", e);
                    return Err(format!("Failed to save query data: {}", e));
                }
            }

        }

        Ok((similar_files, llm_analysis))
    }

    async fn get_llm_analysis(&self, query_text: &str, similar_files: &[(String, String, f32, std::option::Option<Vec<f32>>)], project: &Project) -> Result<String, String> {
        // Initialize LLM service and file service
        let llm_service = LlmService::new();
        let file_service = FileService {};
        let llm_provider = project.provider.clone();
        
        // Extract code from similar files
        let mut file_code = String::new();
        for (file_path, _, _, _) in similar_files {
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
            Based on the user query and the provided code, please provide a JSON code block with the following structure:\n\
            ```json\n{{\n  \"accurate_vector_results\": \"explanation\",\n  \"suggested_files\": [\"file1.rs\", \"file2.rs\", ...]\n}}\n```\n\
            - `accurate_vector_results`: A string explaining whether the vector results seem relevant to the query and why.\n\
            - `suggested_files`: A list of file paths (relative to the project root) that would be needed to fully answer this query. These files should be chosen from the File Descriptions section."
            ,
            query_text,
            file_descriptions,
            file_code
        );
        
        // Get LLM analysis
        let llm_response = llm_service.get_analysis(&prompt, &llm_provider, project.specific_model.as_deref()).await;

        Ok(llm_response)
    }

}