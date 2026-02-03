// src/routes/llm/regenerate_yaml.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::{AppState, Project, ProjectFile};
use crate::services::yaml::management::YamlManagement;
use std::fs::{read_to_string, write};
use std::path::{Path, PathBuf};
use serde::Deserialize;
use crate::services::llm_service::LlmServiceConfig; // Import LlmServiceConfig

#[derive(Deserialize)]
pub struct QueryData {
    pub yaml_path: String,
}

#[post("/regenerate")]
pub async fn regenerate_yaml(
    app_state: web::Data<AppState>,
    query: web::Query<RegenParams>,
    data: web::Json<QueryData>,
) -> impl Responder {
    let project_name = &query.project;
    let yaml_path = &data.yaml_path;

    let output_dir = Path::new(&app_state.output_dir).join(project_name);
    let project_settings_path = output_dir.join("project_settings.json");

    if let Ok(project_settings_json) = read_to_string(project_settings_path) {
        if let Ok(project) = serde_json::from_str::<Project>(&project_settings_json) {
            let source_file_path = construct_source_path(&project.source_dir, &yaml_path);
            
            let file_content = match read_to_string(&source_file_path) {
                Ok(content) => content,
                Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to read source file {}: {}", source_file_path.display(), e)),
            };
            let project_file = ProjectFile {
                path: source_file_path.to_string_lossy().to_string(),
                content: file_content,
                last_modified: 0,
            };
            
            let yaml_management = YamlManagement::new();
            // Create a default LlmServiceConfig for the regeneration process
            let llm_config = LlmServiceConfig::new();
            let combined_content_option = yaml_management.create_yaml_with_imports(
                &project_file, 
                &project.provider, 
                project.specific_model.as_deref(), 
                project.yaml_model.as_deref(), 
                Some(llm_config) // Pass config
            ).await;

            if let Some(combined_content) = combined_content_option {
                match write(&yaml_path, &combined_content) {
                    Ok(_) => HttpResponse::Ok().body(combined_content),
                    Err(e) => HttpResponse::InternalServerError().body(format!("Failed to write regenerated YAML to {}: {}", yaml_path, e)),
                }
            } else {
                HttpResponse::InternalServerError().body("Failed to regenerate YAML: LLM conversion failed")
            }
        } else {
            HttpResponse::InternalServerError().body("Failed to parse project settings")
        }
    } else {
        HttpResponse::InternalServerError().body("Failed to read project settings file")
    }
}


fn construct_source_path(source_dir: &str, yaml_path: &str) -> PathBuf {
    let yaml_file_name = Path::new(yaml_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .replace("*", "/").replace(".yml", "");

    let mut source_path = PathBuf::from(source_dir);
    source_path.push(yaml_file_name);
    source_path
}

#[derive(serde::Deserialize)]
struct RegenParams {
    project: String
}