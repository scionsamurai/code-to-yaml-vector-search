// src/services/search_service.rs
use crate::models::Project;
use crate::services::embedding_service::EmbeddingService;
use crate::services::qdrant_service::QdrantService;
use crate::services::llm_service::{LlmService, LlmServiceConfig}; // Import LlmServiceConfig
use crate::services::file::FileService;
use crate::services::project_service::ProjectService;
use crate::models::QueryData;
use std::env;

#[derive(Debug, Clone)] // Add Clone and Debug for use in agent service
pub struct SearchResult {
    pub file_path: String,
    pub file_content: String, // Include file content
    pub file_description: Option<String>, // Include file description
    pub score: f32,
    pub embedding: Option<Vec<f32>>,
}

pub struct SearchService;
// src/services/search_service.rs

impl SearchService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn search_project(
        &self,
        project: &mut Project,
        query_text: &str,
        output_dir: Option<&std::path::PathBuf>,
        num_results: u64, // Add num_results parameter
        llm_analysis: bool, // Add llm_analysis parameter
    ) -> Result<(Vec<SearchResult>, String), String> {

        // 2. Generate embedding
        let embedding_service = EmbeddingService::new();
        let query_embedding = match embedding_service
            .generate_embedding(query_text, Some(1536))
            .await
        {
            Ok(embedding) => embedding,
            Err(e) => return Err(e.to_string()),
        };

        // 3. Search for similar files using vector search
        let qdrant_server_url =
            env::var("QDRANT_SERVER_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
        let qdrant_service = match QdrantService::new(&qdrant_server_url, 1536).await {
            Ok(service) => service,
            Err(e) => return Err(format!("Failed to connect to Qdrant: {}", e)),
        };

        let similar_files_with_score = match qdrant_service
            .search_similar_files(&project.name, query_embedding, num_results, false) // Use num_results
            .await
        {
            Ok(files) => files,
            Err(e) => return Err(e.to_string()),
        };

        // Initialize FileService
        let file_service = FileService {};

        // Map the search results with file data and descriptions
        let mut search_results: Vec<SearchResult> = Vec::new();
        for (file_path, _, score, embedding) in similar_files_with_score {
            let file_content = file_service
                .read_specific_file(project, &file_path)
                .unwrap_or_default();

            // Use existing file description if available
            let file_description = project.file_descriptions.get(&file_path).cloned();

            search_results.push(SearchResult {
                file_path,
                file_content,
                file_description,
                score,
                embedding,
            });
        }


        // Get LLM recommendations based on search results
        let llm_analysis = if llm_analysis {
            self
            .get_llm_analysis(query_text, &search_results, project) // Pass the query_text and search_results
            .await?
        } else {
            String::new()
        };

        if output_dir.is_some() {
            let project_service = ProjectService::new();

            // Always create a new file for each query
            let filename = project_service.query_manager.generate_query_filename(); // MODIFIED LINE
            let mut query_data = QueryData::default();

            // Update the QueryData with search results and LLM analysis
            query_data.query = query_text.to_string();
            query_data.vector_results = search_results
                .iter()
                .map(|sr| (sr.file_path.clone(), sr.score))
                .collect(); // MODIFIED LINE
            query_data.llm_analysis = llm_analysis.clone();

            // Save the updated QueryData
            match project_service.query_manager.save_query_data(
                &output_dir.unwrap(),
                &query_data,
                &filename,
            ) {
                // MODIFIED LINE
                Ok(_) => {
                    println!("Query data saved successfully.");
                }
                Err(e) => {
                    eprintln!("Failed to save query data: {}", e);
                    return Err(format!("Failed to save query data: {}", e));
                }
            }
        }

        Ok((search_results, llm_analysis)) // MODIFIED LINE - removed search_results_with_explanations
    }

    async fn get_llm_analysis(&self, query_text: &str, search_results: &Vec<SearchResult>, project: &Project) -> Result<String, String> { //MODIFIED LINE
        // Initialize LLM service
        let llm_service = LlmService::new();
        let llm_provider = project.provider.clone();
        let llm_config = LlmServiceConfig::new(); // Default config

        // Extract code from similar files
        let mut file_code = String::new();
        for search_result in search_results {
            file_code.push_str(&format!("// File: {}\n{}\n{}\n\n", search_result.file_path, search_result.file_description.clone().unwrap_or_default(), search_result.file_content));
        }
        let file_descriptions = project.file_descriptions.iter()
        .map(|(path, desc)| format!("{}: {}", path, desc))
        .collect::<Vec<String>>()
        .join("\n");

        // Create the prompt for the LLM
        let prompt: String = format!(
            "User Query: \"{}\"\n\n\
            File Descriptions:\n\
            ```\n{}\n```\n\n\
            Related code from vector search:\n\
            ```\n{}\n```\n\n\
            Based on the user query and the provided code, please provide a JSON code block with the following structure:\n\
            ```json\n{{\n  \"accurate_vector_results\": \"explanation\",\n  \"suggested_files\": [\"file1.rs\", \"file2.rs\", ...],\n  \"bm25_keywords\": \"space separated keywords for broader search\"\n}}\n```\n\
            - `accurate_vector_results`: A string explaining whether the vector results seem relevant to the query and why.\n\
            - `suggested_files`: A list of file paths (relative to the project root) that would be needed to fully answer this query. These files should be chosen from the File Descriptions section.\n\
            - `bm25_keywords`: A space-separated string of keywords that represent a broader set of terms that might be relevant for finding additional files via a keyword-based search (BM25F) over YAML summaries."
            ,
            query_text,
            file_descriptions,
            file_code
        );
        // Get LLM analysis
        let llm_response = llm_service.get_analysis(&prompt, &llm_provider, project.specific_model.as_deref(), Some(llm_config)).await;

        Ok(llm_response)
    }
}