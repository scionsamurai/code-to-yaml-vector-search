// src/routes/project/update_settings.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use crate::services::yaml::management::YamlManagement;
use std::path::Path;
use serde::Deserialize;
use actix_rt::spawn;
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Debug)]
pub struct ProjectSettings {
    pub languages: String,
    pub provider: String,
    pub default_use_yaml: Option<bool>,
    pub specific_model: Option<String>,
    pub yaml_model: Option<String>, // New YAML model field
    pub git_integration_enabled: Option<bool>,
}

#[post("/update/{name}/settings")]
pub async fn update_settings(
    app_state: web::Data<AppState>,
    name: web::Path<String>,
    form: web::Form<ProjectSettings>,
) -> impl Responder {
    let name = name.into_inner();
    let output_dir = Path::new(&app_state.output_dir).join(&name);
    let app_state_arc = Arc::new(app_state.get_ref());

    let project_service = ProjectService::new();
    let yaml_management = YamlManagement::new();

    // Load existing project
    match project_service.load_project(&output_dir) {
        Ok(mut project) => {
            let old_default_use_yaml = project.default_use_yaml;
            // Update project settings
            project.languages = form.languages.clone();
            project.provider = form.provider.clone();
            project.default_use_yaml = form.default_use_yaml.unwrap_or(false);
            project.specific_model = form.specific_model.clone();
            project.yaml_model = form.yaml_model.clone(); // Save the new YAML model
            project.git_integration_enabled = form.git_integration_enabled.unwrap_or(false);
            let new_default_use_yaml = project.default_use_yaml;

            // Save updated project
            if let Err(e) = project_service.save_project(&project, &output_dir) {
                return HttpResponse::InternalServerError()
                    .body(format!("Failed to update project settings: {}", e));
            }

            // Clone necessary data for the background task
           let output_dir_str = app_state_arc.output_dir.clone();
           let arc_yaml_management = Arc::new(yaml_management);

           // Wrap the project in a Mutex *before* putting it in the Arc
           let project_mutex = Arc::new(Mutex::new(project.clone()));

           // Check if default_use_yaml has changed and trigger re-embedding
           if old_default_use_yaml != new_default_use_yaml {
               let project_mutex_clone = Arc::clone(&project_mutex);
                // Spawn a background task using actix_rt::spawn
               spawn(async move {
                   println!("default_use_yaml setting changed, regenerating embeddings in background...");

                   let mut project = project_mutex_clone.lock().unwrap();
                   // Iterate through files and regenerate embeddings
                   for file_path in project.embeddings.keys().cloned().collect::<Vec<String>>() {
                    arc_yaml_management.regenerate_embedding(&mut *project, &file_path, &output_dir_str).await; // Access the project through the MutexGuard
                }
               });
           }

            // Redirect back to the project page
            HttpResponse::SeeOther()
                .append_header(("Location", format!("/projects/{}", name)))
                .finish()
        },
        Err(e) => HttpResponse::NotFound().body(format!("Project not found: {}", e)),
    }
}
