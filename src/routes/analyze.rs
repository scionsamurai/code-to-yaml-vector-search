// src/routes/analyze.rs

use crate::services::git_service::GitService;
use crate::render_svelte;
use serde::Serialize;
use std::path::{Path, PathBuf};
use serde_json::Value;
use crate::services::utils::html_utils::unescape_html;
use std::collections::HashMap;

use actix_web::{web, get, Responder, HttpResponse};
use crate::models::{AppState, ChatMessage};
use crate::services::project_service::ProjectService;
use crate::services::template::TemplateService;

// --- ADD THIS DATA STRUCTURE ---
#[derive(Serialize)]
struct ChatHistoryResponse {
    history: Vec<ChatMessage>,
}

// --- ADD THIS ROUTE ---
#[get("/{project}/{query_id}/chat_history")]
async fn get_chat_history(
    app_state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (project_name, query_id) = path.into_inner();

    let project_service = ProjectService::new();
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&project_name);

    // --- Load Chat History from the project ---
    let chat_history = project_service
        .chat_manager
        .get_analysis_chat_history(&project_service.query_manager, &project_dir, &query_id);

    // --- Create the response structure ---
    let response = ChatHistoryResponse { history: chat_history };

    // --- Serialize to JSON and return ---
    match serde_json::to_string(&response) {
        Ok(json) => HttpResponse::Ok()
            .content_type("application/json")
            .body(json),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to serialize chat history: {}", e)),
    }
}

fn process_llm_analysis_suggestions(
    project_service: &ProjectService,
    project_dir: &Path,
    query_id: &str,
    project_source_dir: &Path,
    original_relevant_files: Vec<String>,
) -> (Vec<String>, Vec<String>) {
    let mut llm_suggested_files: Vec<String> = Vec::new();
    let mut actual_relevant_files: Vec<String> = Vec::new();

    if let Some(llm_analysis_str) = project_service.query_manager.get_query_data_field(project_dir, query_id, "llm_analysis") {
        let llm_analysis_unescaped = unescape_html(llm_analysis_str);
        let llm_analysis_json_str = llm_analysis_unescaped
            .split("```json")
            .nth(1)
            .and_then(|s| s.split("```").next())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "".to_string());

        match serde_json::from_str::<Value>(&llm_analysis_json_str) {
            Ok(json_value) => {
                if let Some(files_array) = json_value["suggested_files"].as_array() {
                    for file_val in files_array {
                        if let Some(file_path_str) = file_val.as_str() {
                            // Resolve the file_path_str to an absolute path
                            let absolute_path: PathBuf = project_source_dir.join(file_path_str);
                            if let Some(abs_path_str) = absolute_path.to_str() {
                                llm_suggested_files.push(abs_path_str.to_string());
                            } else {
                                eprintln!("Warning: Failed to convert absolute path to string for LLM suggested file: {:?}", absolute_path);
                            }
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("Failed to parse LLM analysis JSON for query {}: {}", query_id, e);
                // Continue without LLM suggested files
            }
        }
    }

    // Filter out any llm_suggested_files from the original_relevant_files
    for file in original_relevant_files {
        if !llm_suggested_files.contains(&file) {
            actual_relevant_files.push(file);
        }
    }

    (llm_suggested_files, actual_relevant_files)
}


#[derive(serde::Deserialize, Debug)]
pub struct AnalyzeQueryPath {
    project: String,
    query_id: String,
}

#[derive(Serialize)]
struct NewAnalyzeQueryProps { // Example data structure
    project_name: String,
    query_id: String,
    query_text: String,
    project_source_dir: String,
    project_provider: String,
    relevant_files: Vec<String>,
    saved_context_files: Vec<String>,
    llm_suggested_files: Vec<String>,
    available_queries: Vec<(String, String)>,
    include_file_descriptions: bool,
    auto_commit: bool,
    current_repo_branch_name: String,
    all_branches: Vec<String>,
    git_enabled: bool,
    file_yaml_override: HashMap<String, bool>,
    default_use_yaml: bool,
    agentic_mode_enabled: bool,
}

#[get("/{project}/{query_id}")]
async fn new_analyze_query_route(
    app_state: web::Data<AppState>,
    path: web::Path<AnalyzeQueryPath>,
) -> impl Responder {

    let project_name = path.project.clone();
    let query_id = path.query_id.clone();
    println!("Received request for project: {}, query_id: {}", project_name, query_id);


    let project_service = crate::services::project_service::ProjectService::new();

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    println!("output_dir: {:?}", output_dir);
    let project_dir = output_dir.join(&project_name); 
    println!("Loading project from directory: {:?}", project_dir);


    let mut project = project_service.load_project(&project_dir)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to load project: {}", e))).unwrap();

    let original_relevant_files = project_service.query_manager.get_query_vec_field(&project_dir, &query_id, "vector_results").unwrap_or_default();
    let saved_context_files = project_service.query_manager.get_query_vec_field(&project_dir, &query_id, "context_files").unwrap_or_default();
    let last_query_text = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "query").unwrap_or_else(|| "No previous query found".to_string());
    let include_file_descriptions = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "include_file_descriptions").unwrap_or_else(|| "false".to_string()) == "true";
    let auto_commit = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "auto_commit").unwrap_or_else(|| "false".to_string()) == "true";
    let agentic_mode_enabled = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "agentic_mode_enabled").unwrap_or_else(|| "false".to_string()) == "true";

    let available_queries = match project_service.query_manager.get_query_filenames(&project_dir) {
        Ok(queries) => queries,
        Err(e) => {
            eprintln!("Failed to get query filenames: {}", e);
            Vec::new()
        }
    };
    
    let mut all_branches: Vec<String> = Vec::new();
    let mut current_repo_branch_name: Option<String> = None;

    if project.git_integration_enabled {
        match GitService::open_repository(Path::new(&project.source_dir)) {
            Ok(repo) => {
                match GitService::get_all_branch_names(&repo) {
                    Ok(branches) => all_branches = branches,
                    Err(e) => eprintln!("Failed to get all branch names: {}", e),
                }
                match GitService::get_current_branch_name(&repo) {
                    Ok(branch) => {
                        current_repo_branch_name = Some(branch.clone());
                        if project.git_branch_name.is_none() || project.git_branch_name.as_ref() != Some(&branch) {
                            project.git_branch_name = Some(branch);
                            if let Err(e) = project_service.save_project(&project, &project_dir) {
                                eprintln!("Failed to save project after updating git_branch_name: {}", e);
                            }
                        }
                    },
                    Err(e) => eprintln!("Failed to get current repository branch name: {}", e),
                }
            },
            Err(e) => {
                eprintln!("Failed to open Git repository for branch info: {}", e);
            }
        }
    }

    let (llm_suggested_files, relevant_files) = process_llm_analysis_suggestions(
        &project_service,
        &project_dir,
        &query_id,
        Path::new(&project.source_dir),
        original_relevant_files,
    );

    let git_enabled = project.git_integration_enabled;
    // convert last_query_text to json safe string
    let query_text = last_query_text.replace("\'", "&#39;");
    println!("Rendering AnalyzeQuery with query_text: {}", query_text);

    let props = NewAnalyzeQueryProps {
        project_name: project_name.clone(),
        query_id: query_id.clone(),
        query_text: query_text.replace("&#34;", "\""),
        project_source_dir: project.source_dir.clone(),
        project_provider: project.provider.clone(),
        relevant_files: relevant_files.clone(),
        saved_context_files: saved_context_files.clone(),
        llm_suggested_files: llm_suggested_files.clone(),
        available_queries: available_queries.clone(),
        include_file_descriptions: include_file_descriptions,
        auto_commit: auto_commit,
        current_repo_branch_name: current_repo_branch_name.unwrap_or_default(),
        all_branches: all_branches.clone(),
        git_enabled: git_enabled,
        file_yaml_override: project.file_yaml_override.clone(),
        default_use_yaml: project.default_use_yaml,
        agentic_mode_enabled: agentic_mode_enabled,
    };

    render_svelte("AnalyzeQuery", Some("Analyze Query"), Some(props))
}


// --- NEW: Request and Response models for `get_other_project_files` endpoint ---
#[derive(serde::Deserialize)]
pub struct GetOtherFilesRequest {
    project_name: String,
    excluded_files: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct GetOtherFilesResponse {
    success: bool,
    files: Vec<String>,
    message: Option<String>,
}

#[actix_web::post("/get-other-project-files")]
async fn get_other_project_files(
    app_state: web::Data<AppState>,
    req: web::Json<GetOtherFilesRequest>,
) -> impl Responder {
    let project_service = ProjectService::new();
    let template_service = TemplateService::new();
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&req.project_name);

    let project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError().json(GetOtherFilesResponse {
                success: false,
                files: Vec::new(),
                message: Some(format!("Failed to load project: {}", e)),
            });
        }
    };

    let other_files = template_service.get_other_files_list_raw(
        &project,
        &req.excluded_files,
    );

    HttpResponse::Ok().json(GetOtherFilesResponse {
        success: true,
        files: other_files,
        message: None,
    })
}


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_chat_history);
    cfg.service(new_analyze_query_route);
    cfg.service(get_other_project_files);
}
