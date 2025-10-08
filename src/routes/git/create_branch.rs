// src/routes/git/create_branch.rs

use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::services::project_service::ProjectService;
use crate::services::git_service::{GitService, GitError};
use crate::models::AppState;
use std::path::Path;

#[derive(Deserialize)]
pub struct CreateBranchRequest {
    project_name: String,
    branch_name: String,
}

#[derive(Serialize)]
pub struct CreateBranchResponse {
    success: bool,
    message: String,
}

#[post("/create-git-branch")]
pub async fn create_branch(
    data: web::Json<CreateBranchRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let project_service = ProjectService::new();

    let project_name = &data.project_name;
    let branch_name = &data.branch_name;

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(project_name);

    let mut project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(CreateBranchResponse {
                    success: false,
                    message: format!("Failed to load project: {}", e),
                });
        }
    };

    if !project.git_integration_enabled {
        return HttpResponse::BadRequest().json(CreateBranchResponse {
            success: false,
            message: "Git integration is not enabled for this project.".to_string(),
        });
    }

    if let Err(e) = project_service.load_project_env(&project_dir) {
        eprintln!("Warning: Failed to load project .env: {}", e);
    }

    let repo = match GitService::open_repository(Path::new(&project.source_dir)) {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::InternalServerError().json(CreateBranchResponse {
                success: false,
                message: format!("Failed to open Git repository: {}", e),
            });
        }
    };

   let latest_commit = match GitService::get_latest_commit(&repo) {
        Ok(commit) => commit,
        Err(e) => {
            return HttpResponse::InternalServerError().json(CreateBranchResponse {
                success: false,
                message: format!("Failed to get latest commit: {}", e),
            });
        }
    };

    match GitService::create_branch(&repo, branch_name, &latest_commit) {
        Ok(_) => {},
        Err(e) => {
            return HttpResponse::InternalServerError().json(CreateBranchResponse {
                success: false,
                message: format!("Failed to create branch: {}", e),
            });
        }
    };

    match GitService::checkout_branch(&repo, branch_name) {
        Ok(_) => {},
        Err(e) => {
            return HttpResponse::InternalServerError().json(CreateBranchResponse {
                success: false,
                message: format!("Failed to checkout branch: {}", e),
            });
        }
    };

    // Update project's git_branch_name
    project.git_branch_name = Some(branch_name.clone());

    // Save the updated project
    if let Err(e) = project_service.save_project(&project, &project_dir) {
        return HttpResponse::InternalServerError().json(CreateBranchResponse {
            success: false,
            message: format!("Failed to save project: {}", e),
        });
    }

    HttpResponse::Ok().json(CreateBranchResponse {
        success: true,
        message: format!("Branch '{}' created and checked out successfully.", branch_name),
    })
}