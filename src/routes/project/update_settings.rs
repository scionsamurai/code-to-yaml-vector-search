// src/routes/project/update_settings.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use std::path::Path;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ProjectSettings {
    pub languages: String,
    pub model: String,
}

#[post("/update/{name}/settings")]
pub async fn update_settings(
    app_state: web::Data<AppState>,
    name: web::Path<String>,
    form: web::Form<ProjectSettings>,
) -> impl Responder {
    let name = name.into_inner();
    let output_dir = Path::new(&app_state.output_dir).join(&name);
    
    let project_service = ProjectService::new();
    
    // Load existing project
    match project_service.load_project(&output_dir) {
        Ok(mut project) => {
            // Update project settings
            project.languages = form.languages.clone();
            project.model = form.model.clone();
            
            // Save updated project
            if let Err(e) = project_service.save_project(&project, &output_dir) {
                return HttpResponse::InternalServerError()
                    .body(format!("Failed to update project settings: {}", e));
            }
            
            // Redirect back to the project page
            HttpResponse::SeeOther()
                .append_header(("Location", format!("/projects/{}", name)))
                .finish()
        },
        Err(e) => HttpResponse::NotFound().body(format!("Project not found: {}", e)),
    }
}