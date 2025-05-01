// src/routes/update_project.rs
use actix_web::{get, web, HttpResponse, Responder};
use crate::models::{AppState, Project};
use crate::services::yaml::YamlService;
use crate::services::project_service::ProjectService;
use std::fs::read_to_string;
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
        if let Ok(mut project) = serde_json::from_str::<Project>(&project_settings_json) {


            project.saved_queries = Some(Vec::new());

            let project_service = ProjectService::new();
            if let Err(e) = project_service.save_project(&project, &output_dir) {
                eprintln!("Failed to save project settings after clearing queries: {}", e);
                return HttpResponse::InternalServerError().body(format!("Failed to save updated project settings: {}", e));
            }

            let yaml_service = YamlService::new();
            yaml_service.save_yaml_files(&mut project, &app_state.output_dir).await;

            // Redirect back to the project page
            HttpResponse::SeeOther()
                .append_header(("Location", format!("/projects/{}", name)))
                .finish()
        } else {
            HttpResponse::InternalServerError().body("Failed to deserialize project settings")
        }
    } else {
        HttpResponse::NotFound().body("Project not found")
    }
}
