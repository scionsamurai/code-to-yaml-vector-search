// src/routes/suggest_split.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::{AppState, Project};
use crate::services::llm_service::LlmService;
use std::fs::read_to_string;
use std::path::Path;

#[post("/suggest-split")]
pub async fn suggest_split(
    app_state: web::Data<AppState>,
    query: web::Query<SplitParams>,
) -> impl Responder {
    let project_name = &query.project;
    let file_path = &query.file_path;
    
    // Get project details
    let project_settings_path = Path::new(&app_state.output_dir)
        .join(project_name)
        .join("project_settings.json");
    
    let project = match read_to_string(&project_settings_path) {
        Ok(json) => match serde_json::from_str::<Project>(&json) {
            Ok(project) => project,
            Err(e) => return HttpResponse::BadRequest().body(format!("Invalid project settings: {}", e)),
        },
        Err(e) => return HttpResponse::NotFound().body(format!("Project settings not found: {}", e)),
    };
    
    // Read the file content
    let file_path_full = Path::new(&project.source_dir).join(file_path);
    let file_content = match read_to_string(&file_path_full) {
        Ok(content) => content,
        Err(e) => return HttpResponse::NotFound().body(format!("File not found: {}", e)),
    };
    // Create a prompt for the LLM with file descriptions
    let file_descriptions_text = project.file_descriptions.iter()
        .map(|(path, desc)| format!("{}: {}", path, desc))
        .collect::<Vec<String>>()
        .join("\n");
    // print!("project: {:?}", project);
    // Create a prompt for the LLM
    let prompt = format!(
        "{}\nAbove is the descriptions for the files in this project. I need to split a large file into smaller, more manageable pieces. Please suggest how to split this file into multiple smaller files.\n\n\
        File path: {}\n\n\
        Please provide a detailed plan for splitting this file, including:\n\
        1. The new files that should be created\n\
        2. What content should go in each file\n\
        3. How the files should reference each other\n\
        4. Any refactoring needed to maintain functionality\n\
        Be specific and provide actual code examples when appropriate.\n\n\
        File content:\n```\n{}\n```\n\n",
        file_descriptions_text,
        file_path,
        file_content
    );
    
    // Use the LLM service to get the analysis
    let llm_service = LlmService::new();
    let analysis = llm_service.get_analysis(&prompt, &project.model).await;
    
    HttpResponse::Ok().body(analysis)
}

#[derive(serde::Deserialize)]
struct SplitParams {
    project: String,
    file_path: String,
}