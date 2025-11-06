// src/routes/git/checkout_branch.rs

use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::services::project_service::ProjectService;
use crate::services::git_service::GitService;
use crate::models::AppState;
use std::path::Path;

#[derive(Deserialize)]
pub struct CheckoutBranchRequest {
    project_name: String,
    branch_name: String,
}

#[derive(Serialize)]
pub struct CheckoutBranchResponse {
    success: bool,
    message: String,
}

#[post("/checkout-git-branch")]
pub async fn checkout_branch(
    data: web::Json<CheckoutBranchRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let project_service = ProjectService::new();
    let project_name = &data.project_name;
    let new_branch_name = &data.branch_name;

    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(project_name);

    let mut project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError().json(CheckoutBranchResponse {
                success: false,
                message: format!("Failed to load project: {}", e),
            });
        }
    };

    if !project.git_integration_enabled {
        return HttpResponse::BadRequest().json(CheckoutBranchResponse {
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
            return HttpResponse::InternalServerError().json(CheckoutBranchResponse {
                success: false,
                message: format!("Failed to open Git repository: {}", e),
            });
        }
    };

    match GitService::checkout_branch(&repo, new_branch_name) {
        Ok(_) => {
            // Update project's git_branch_name
            project.git_branch_name = Some(new_branch_name.clone());

            // Save the updated project
            if let Err(e) = project_service.save_project(&project, &project_dir) {
                return HttpResponse::InternalServerError().json(CheckoutBranchResponse {
                    success: false,
                    message: format!("Failed to save project: {}", e),
                });
            }

            HttpResponse::Ok().json(CheckoutBranchResponse {
                success: true,
                message: format!("Checked out to branch '{}' successfully.", new_branch_name),
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(CheckoutBranchResponse {
                success: false,
                message: format!("Failed to checkout branch '{}': {}", new_branch_name, e),
            })
        }
    }
}