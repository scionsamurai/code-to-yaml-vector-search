// src/routes/analyze_query.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::{AppState, Project};
use crate::services::llm_service::LlmService;
use crate::services::embedding_service::EmbeddingService;
use crate::services::qdrant_service::QdrantService;
use std::fs::read_to_string;
use std::path::Path;
use std::env;

#[derive(serde::Deserialize)]
pub struct AnalyzeQueryForm {
    project: String,
    query: String,
}

#[post("/analyze-query")]
pub async fn analyze_query(
    app_state: web::Data<AppState>,
    form: web::Form<AnalyzeQueryForm>,
) -> impl Responder {
    let output_dir = Path::new(&app_state.output_dir).join(&form.project);
    let project_settings_path = output_dir.join("project_settings.json");

    if let Ok(project_settings_json) = read_to_string(project_settings_path) {
        if let Ok(project) = serde_json::from_str::<Project>(&project_settings_json) {
            // 1. Generate embedding for the query
            let embedding_service = EmbeddingService::new();
            let query_embedding = match embedding_service.generate_embedding(&form.query, None).await {
                Ok(embedding) => embedding,
                Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to generate embedding: {}", e)),
            };

            // 2. Search for similar files
            let qdrant_server_url = env::var("QDRANT_SERVER_URL").unwrap();
            let qdrant_service = match QdrantService::new(&qdrant_server_url, 1536).await {
                Ok(service) => service,
                Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to connect to Qdrant: {}", e)),
            };

            let similar_files = match qdrant_service.search_similar_files(&project.name, query_embedding, 5).await {
                Ok(files) => files,
                Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to search similar files: {}", e)),
            };

            // 3. Collect YAML content and file paths
            let mut yaml_contents = Vec::new();
            let mut file_paths = Vec::new();

            for (file_path, yaml_content, _score) in similar_files {
                yaml_contents.push(yaml_content);
                file_paths.push(file_path);
            }

            // 4. Use LLM to determine which parts of files are needed
            let llm_service = LlmService::new();
            
            // Create prompt for LLM
            let prompt = format!(
                "Based on the user query: '{}' and the following YAML representations of code files, identify which files or specific parts of files are most relevant to answer the query. Return your answer in the format:\n\n```json\n{{\n  \"relevant_files\": [\n    {{\n      \"file_path\": \"path/to/file.rs\",\n      \"include_whole_file\": true/false,\n      \"relevant_parts\": [\"function_name\", \"class_name\", etc.]\n    }}\n  ]\n}}\n```\n\nYAML representations:\n\n{}",
                form.query,
                yaml_contents.join("\n\n")
            );
            
            let analysis = llm_service.get_analysis(&prompt, &project.model).await;
            
            // 5. Extract code from the files based on LLM response
            let mut final_code = String::new();
            let mut final_llm_query = format!("User Query: {}\n\nRelevant code:\n\n", form.query);
            
            // Parse LLM response to get relevant files and parts
            // This is a simplified version - you'll need to parse the JSON response properly
            if let Some(json_start) = analysis.find("```json") {
                if let Some(json_end) = analysis[json_start..].find("```") {
                    let json_str = &analysis[json_start + 7..json_start + json_end].trim();
                    
                    if let Ok(analysis_result) = serde_json::from_str::<serde_json::Value>(json_str) {
                        if let Some(relevant_files) = analysis_result.get("relevant_files").and_then(|v| v.as_array()) {
                            for file in relevant_files {
                                if let Some(file_path) = file.get("file_path").and_then(|v| v.as_str()) {
                                    let source_path = Path::new(&project.source_dir).join(file_path);
                                    
                                    if let Ok(source_content) = read_to_string(&source_path) {
                                        let include_whole_file = file.get("include_whole_file")
                                            .and_then(|v| v.as_bool())
                                            .unwrap_or(false);
                                            
                                        if include_whole_file {
                                            final_code.push_str(&format!("// File: {}\n{}\n\n", file_path, source_content));
                                        } else if let Some(parts) = file.get("relevant_parts").and_then(|v| v.as_array()) {
                                            // In a real implementation, you'd need to parse the file and extract the specific parts
                                            // This is a placeholder
                                            final_code.push_str(&format!("// File: {} (relevant parts)\n", file_path));
                                            
                                            for part in parts {
                                                if let Some(part_name) = part.as_str() {
                                                    final_code.push_str(&format!("// Part: {}\n", part_name));
                                                    // Here you would extract the specific part from the source code
                                                }
                                            }
                                            
                                            final_code.push_str("\n\n");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            final_llm_query.push_str(&final_code);
            
            // 6. Display the final query and code
            let html = format!(
                r#"
                <html>
                    <head>
                        <title>Query Analysis - {}</title>
                        <link rel="stylesheet" href="/static/project.css">
                    </head>
                    <body>
                        <div class="head">
                            <h1>Query Analysis for "{}"</h1>
                            <p>Project: {}</p>
                        </div>
                        
                        <div class="analysis-container">
                            <h2>LLM Analysis</h2>
                            <pre>{}</pre>
                            
                            <h2>Final LLM Query</h2>
                            <pre>{}</pre>
                            
                            <div class="actions">
                                <a href="/projects/{}" class="button">Back to Project</a>
                                <form action="/execute-query" method="post">
                                    <input type="hidden" name="project" value="{}">
                                    <input type="hidden" name="query" value="{}">
                                    <input type="hidden" name="code" value="{}">
                                    <button type="submit">Execute Query</button>
                                </form>
                            </div>
                        </div>
                    </body>
                </html>
                "#,
                project.name,
                form.query,
                project.name,
                analysis,
                final_llm_query,
                project.name,
                project.name,
                form.query,
                final_code
            );
            
            HttpResponse::Ok().body(html)
        } else {
            HttpResponse::InternalServerError().body("Failed to deserialize project settings")
        }
    } else {
        HttpResponse::NotFound().body("Project not found")
    }
}