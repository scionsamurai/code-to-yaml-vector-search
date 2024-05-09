// src/routes/create_project.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::{AppState, Project};
use crate::utils::save_yaml_files;
use std::fs::write;
use std::path::Path;

#[derive(serde::Deserialize)]
struct CreateProjectForm {
    name: String,
    languages: String,
    source_dir: String,
    llms: String,
}

#[post("/projects")]
pub async fn create_project(
    app_state: web::Data<AppState>,
    form_data: web::Form<CreateProjectForm>,
) -> impl Responder {
    let form_data = form_data.into_inner();
    let project = Project {
        name: form_data.name.clone(),
        languages: form_data.languages.clone(),
        source_dir: form_data.source_dir.clone(),
        model: form_data.llms.clone()
    };

    let project_name = project.name.clone();
    let output_dir = Path::new(&app_state.output_dir).join(&project_name);
    std::fs::create_dir_all(&output_dir).unwrap();

    let project_settings_path = output_dir.join("project_settings.json");
    let project_settings_json = serde_json::to_string_pretty(&project).unwrap();
    write(project_settings_path, project_settings_json).unwrap();

    save_yaml_files(&project, &app_state).await;
    HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish()
}