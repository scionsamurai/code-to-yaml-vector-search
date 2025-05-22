// src/routes/project/create.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::{AppState, Project};
use crate::services::yaml::YamlService;
use crate::services::project_service::ProjectService;
use std::path::Path;

#[derive(serde::Deserialize)]
struct CreateProjectForm {
    name: String,
    languages: String,
    source_dir: String,
    llms: String,
}

#[post("/projects")]
pub async fn create(
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

    // Use ProjectService instead of direct manipulation
    let project_service = ProjectService::new();
    
    // Save the project settings using the service
    project_service.save_project(&project, &output_dir)
        .unwrap_or_else(|e| eprintln!("Failed to save project: {}", e));
    
    let yaml_service = YamlService::new();
    yaml_service.save_yaml_files(&mut project, &app_state.output_dir, false).await;
    
    HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish()
}