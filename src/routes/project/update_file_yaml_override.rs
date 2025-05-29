// src/routes/project/update_file_yaml_override.rs
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::fs::{read_to_string, write};
use std::path::Path;
use crate::models::{AppState, Project};
use crate::services::yaml::management::YamlManagement; 

#[derive(Deserialize, Debug)]
pub struct FileYamlOverride {
    project: String,
    file_path: String,
    use_yaml: bool,
}

#[post("/update-file-yaml-override")]
pub async fn update_file_yaml_override(
    app_state: web::Data<AppState>,
    form: web::Json<FileYamlOverride>,
) -> impl Responder {
    let project_name = &form.project;
    let file_path = &form.file_path;
    let use_yaml = form.use_yaml;

    let output_dir = Path::new(&app_state.output_dir).join(project_name);
    let project_settings_path = output_dir.join("project_settings.json");

    match read_to_string(&project_settings_path) {
        Ok(project_settings_json) => {
            match serde_json::from_str::<Project>(&project_settings_json) {
                Ok(mut project) => {
                    project.file_yaml_override.insert(file_path.clone(), use_yaml);

                    match serde_json::to_string_pretty(&project) {
                        Ok(updated_project_settings_json) => {
                            if write(&project_settings_path, updated_project_settings_json).is_ok() {
                                // Trigger embedding regeneration
                                let yaml_management = &YamlManagement::new(); //Access YamlManagement from app state, or create it here

                                yaml_management.regenerate_embedding(&mut project, file_path, &app_state.output_dir).await;
                                HttpResponse::Ok().json(format!("YAML override updated for {}", file_path))
                            } else {
                                HttpResponse::InternalServerError().body("Failed to write updated project settings")
                            }
                        }
                        Err(_) => HttpResponse::InternalServerError().body("Failed to serialize project settings")
                    }
                }
                Err(_) => HttpResponse::InternalServerError().body("Failed to deserialize project settings")
            }
        }
        Err(_) => HttpResponse::NotFound().body("Project not found")
    }
}