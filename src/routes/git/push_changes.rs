// src/routes/git/push_changes.rs

use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::services::project_service::ProjectService;
use crate::services::git_service::GitService;
use crate::models::AppState;
use std::path::Path;

#[derive(Deserialize)]
pub struct PushChangesRequest {
    project_name: String,
}

#[derive(Serialize)]
pub struct PushChangesResponse {
    success: bool,
    message: String,
}

#[post("/push-git-changes")]
pub async fn push_changes(
    data: web::Json<PushChangesRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let project_service = ProjectService::new();
    let project_name = &data.project_name;

    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(project_name);

    let project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError().json(PushChangesResponse {
                success: false,
                message: format!("Failed to load project: {}", e),
            });
        }
    };

    if !project.git_integration_enabled {
        return HttpResponse::BadRequest().json(PushChangesResponse {
            success: false,
            message: "Git integration is not enabled for this project.".to_string(),
        });
    }

    if let Err(e) = project_service.load_project_env(&project_dir) {
        eprintln!("Warning: Failed to load project .env for Git author/email for project '{}': {}", project_name, e);
    }

    let repo = match GitService::open_repository(Path::new(&project.source_dir)) {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::InternalServerError().json(PushChangesResponse {
                success: false,
                message: format!("Failed to open Git repository: {}", e),
            });
        }
    };

    let current_branch_name = match GitService::get_current_branch_name(&repo) {
        Ok(name) => name,
        Err(e) => {
            return HttpResponse::InternalServerError().json(PushChangesResponse {
                success: false,
                message: format!("Failed to get current branch name: {}", e),
            });
        }
    };

    // Assume "origin" as the default remote name
    // In a more complex app, this might be configurable or detected.
    let remote_name = "origin";

    match GitService::push_to_remote(&repo, remote_name, &current_branch_name) {
        Ok(_) => {
            HttpResponse::Ok().json(PushChangesResponse {
                success: true,
                message: format!("Successfully pushed branch '{}' to remote '{}'.", current_branch_name, remote_name),
            })
        }
        Err(e) => {
            // Provide more specific error messages for common push failures if possible
            let error_message = format!("Failed to push changes to remote '{}': {}. You may need to configure Git credentials (e.g., SSH key or Personal Access Token) for this project.", remote_name, e);
            HttpResponse::InternalServerError().json(PushChangesResponse {
                success: false,
                message: error_message,
            })
        }
    }
}