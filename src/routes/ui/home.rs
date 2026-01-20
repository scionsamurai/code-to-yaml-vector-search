// src/routes/ui/home.rs
use actix_web::{get, web, Responder};
use crate::models::AppState;
use crate::services::file::FileService;
use crate::services::project_service::ProjectService;
use std::path::Path;
use serde::Serialize;
use crate::render_svelte;

#[derive(Serialize)]
struct HomeProps {
    projects: Vec<ProjectData>,
}

#[derive(Serialize)]
struct ProjectData {
    name: String,
    needs_update: bool,
}

#[get("/")]
pub async fn home(app_state: web::Data<AppState>) -> impl Responder {
    let output_dir = Path::new(&app_state.output_dir);
    let mut projects = Vec::new();
    let file_service = FileService {};
    let project_service = ProjectService::new();

    // Read existing projects from the output directory
    if let Ok(entries) = std::fs::read_dir(output_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();

                if path.is_dir() {
                    if let Ok(project) = project_service.load_project(&path) {
                        let needs_update = file_service.project_needs_update(&project, &app_state.output_dir);
                        projects.push(ProjectData {
                            name: project.name,
                            needs_update,
                        });
                    }
                }
            }
        }
    }

    let props = HomeProps { projects };

    render_svelte("Home", Some("Project Manager"), Some(props))
}