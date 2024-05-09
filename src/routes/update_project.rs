// src/routes/update_project.rs
use actix_web::{get, web, HttpResponse, Responder};
use crate::models::{ AppState, Project };
use crate::utils::{convert_to_yaml, read_files};
use std::fs::{metadata, read_to_string, write};
use std::path::Path;

#[get("/update/{name}")]
pub async fn update_project(
    app_state: web::Data<AppState>,
    name: web::Path<String>,
) -> impl Responder {
    let name = name.into_inner();
    let output_dir = Path::new(&app_state.output_dir).join(&name);
    let project_settings_path = output_dir.join("project_settings.json");

    if let Ok(project_settings_json) = read_to_string(project_settings_path) {
        if let Ok(project) = serde_json::from_str::<Project>(&project_settings_json) {
            let mut gitignore_paths = vec![];
            let files = read_files(&project, &mut gitignore_paths);

            for file in &files {
                let source_path = Path::new(&file.path);
                let yaml_path = output_dir.join(format!("{}.yml", file.path.replace("/", "*")));

                let should_update = match metadata(&yaml_path) {
                    Ok(yaml_metadata) => {
                        let source_metadata = metadata(&source_path).unwrap();
                        source_metadata.modified().unwrap() > yaml_metadata.modified().unwrap()
                    }
                    Err(_) => true, // YAML file doesn't exist
                };

                if should_update {
                    let yaml_content = convert_to_yaml(file, &project.model).await;
                    write(yaml_path, yaml_content).unwrap();
                }
            }

            HttpResponse::SeeOther()
            .append_header(("Location", "/"))
            .finish()
        } else {
            HttpResponse::InternalServerError().body("Failed to deserialize project settings")
        }
    } else {
        HttpResponse::NotFound().body("Project not found")
    }
}