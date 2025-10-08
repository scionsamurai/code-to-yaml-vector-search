// src/routes/llm/analyze_query.rs
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use crate::services::template::TemplateService;
use actix_web::{post, web, HttpResponse, Responder};
use std::path::{Path, PathBuf};
use serde_json::Value;
use crate::services::utils::html_utils::unescape_html;
use crate::services::git_service::GitService;

#[derive(serde::Deserialize, Debug)]
pub struct AnalyzeQueryForm {
    project: String,
    query_id: String,
}

// New helper function to encapsulate the LLM analysis parsing logic
fn process_llm_analysis_suggestions(
    project_service: &ProjectService,
    project_dir: &Path,
    query_id: &str,
    project_source_dir: &Path, // NEW: Added project_source_dir
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


#[post("/analyze-query")]
pub async fn analyze_query(
    app_state: web::Data<AppState>,
    form: web::Form<AnalyzeQueryForm>,
) -> impl Responder {
    let project_service = ProjectService::new();
    let template_service = TemplateService::new();

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&form.project);

    let mut project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to load project: {}", e))
        }
    };


    let query_id = form.query_id.clone();
    // Fetch the original vector_results from the query data
    let original_relevant_files = project_service.query_manager.get_query_vec_field(&project_dir, &query_id, "vector_results").unwrap_or_default();
    let saved_context_files = project_service.query_manager.get_query_vec_field(&project_dir, &query_id, "context_files").unwrap_or_default();
    let existing_chat_history = project_service.chat_manager.get_analysis_chat_history(&project_service.query_manager, &project_dir, &query_id);
    let last_query_text = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "query").unwrap_or_else(|| "No previous query found".to_string());
    let include_file_descriptions = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "include_file_descriptions").unwrap_or_else(|| "false".to_string()) == "true";
    let auto_commit = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "auto_commit").unwrap_or_else(|| "false".to_string()) == "true";


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
                        // If project's git_branch_name is not set or differs from actual repo branch, update it
                        if project.git_branch_name.is_none() || project.git_branch_name.as_ref() != Some(&branch) {
                            project.git_branch_name = Some(branch);
                            // Save the updated project to persist this change
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

    // Call the new helper function
    let (llm_suggested_files, actual_relevant_files) = process_llm_analysis_suggestions(
        &project_service,
        &project_dir,
        &query_id,
        Path::new(&project.source_dir),
        original_relevant_files,
    );

    // Get the list of available queries
    let available_queries = match project_service.query_manager.get_query_filenames(&project_dir) {
        Ok(queries) => queries,
        Err(e) => {
            eprintln!("Failed to get query filenames: {}", e);
            Vec::new()
        }
    };

    // Use the template service to render the HTML
    let html = template_service.render_analyze_query_page(
        &form.project,
        &last_query_text,
        &actual_relevant_files, // Pass the filtered relevant files
        &saved_context_files,
        &project,
        &existing_chat_history, // Pass the Vec<ChatMessage>
        &available_queries,
        &query_id,
        include_file_descriptions,
        &llm_suggested_files, // PASS THE NEW LLM SUGGESTED FILES
        auto_commit,
        current_repo_branch_name.unwrap_or_default(),
        all_branches,
    );

    HttpResponse::Ok().body(html)
}

pub fn _format_message(content: &str) -> String {
    // Create a regex to match triple backtick code blocks
    let re = regex::Regex::new(r"```([a-zA-Z]*)([\s\S]*?)```").unwrap();

    // Replace triple backtick code blocks with formatted HTML
    let formatted_content = re.replace_all(&content, |caps: &regex::Captures| {
        let language = &caps[1];
        let code = caps[2].trim();
        format!("<pre class=\"shiki-block\" data-language=\"{}\" data-original-code=\"{}\"><code class=\"language-{}\">{}</code></pre>", language, code, language, code)
    });

    // Replace newlines with <br> tags for normal text (outside of code blocks)
    // let formatted_content = formatted_content.replace("\n", "<br>");

    formatted_content.to_string()
}