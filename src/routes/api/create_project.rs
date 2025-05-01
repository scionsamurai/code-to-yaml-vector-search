// src/routes/create_project.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::{AppState, Project};
use crate::services::yaml::YamlService;
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
    let mut project = Project {
        name: form_data.name.clone(),
        languages: form_data.languages.clone(),
        source_dir: form_data.source_dir.clone(),
        model: form_data.llms.clone(),
        ..Default::default()
    };

    let project_name = project.name.clone();
    let output_dir = Path::new(&app_state.output_dir).join(&project_name);
    std::fs::create_dir_all(&output_dir).unwrap();

    let project_settings_path = output_dir.join("project_settings.json");
    let project_settings_json = serde_json::to_string_pretty(&project).unwrap();
    write(project_settings_path, project_settings_json).unwrap();

    // Use the YamlService instead of direct utils call
    let yaml_service = YamlService::new();
    yaml_service.save_yaml_files(&mut project, &app_state.output_dir).await;
    
    HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish()
}
