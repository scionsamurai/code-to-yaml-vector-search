// src/routes/regenerate_yaml.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::{AppState, Project, ProjectFile};
use crate::services::llm_service::LlmService;
use std::fs::{read_to_string, write};
use std::path::{Path, PathBuf};

#[post("/regenerate")]
pub async fn regenerate_yaml(
    app_state: web::Data<AppState>,
    query: web::Query<RegenParams>,
) -> impl Responder {
    let project_name = &query.project;
    let yaml_path = &query.yamlpath;

    let output_dir = Path::new(&app_state.output_dir).join(project_name);
    let project_settings_path = output_dir.join("project_settings.json");

    if let Ok(project_settings_json) = read_to_string(project_settings_path) {
        if let Ok(project) = serde_json::from_str::<Project>(&project_settings_json) {
            let source_file_path = construct_source_path(&project.source_dir, &yaml_path);
            let file_content = read_to_string(&source_file_path).unwrap();
            let project_file = ProjectFile {
                path: source_file_path.to_string_lossy().to_string(),
                content: file_content,
                last_modified: 0, // this shouldn't be zero but a specific time in ?millis?
            };
            
            // Use the LlmService instead of direct utils call
            let llm_service = LlmService{};
            let yaml_content = llm_service.convert_to_yaml(&project_file, &project.model).await;
            
            write(&yaml_path, yaml_content.as_bytes()).unwrap();
            return HttpResponse::Ok().body(yaml_content);
        }
    }

    HttpResponse::InternalServerError().body("Failed to regenerate YAML")
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
    project: String,
    yamlpath: String
}