// src/routes/git/get_branches.rs

use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::services::project_service::ProjectService;
use crate::services::git_service::{GitService, GitError};
use crate::models::AppState;
use std::path::Path;

#[derive(Deserialize)]
pub struct GetBranchesRequest {
    project_name: String,
}

#[derive(Serialize)]
pub struct GetBranchesResponse {
    success: bool,
    message: String,
    branches: Option<Vec<String>>,
    current_project_branch: Option<String>,
    current_repo_branch: Option<String>, // The actual branch the repo is currently on
}

#[get("/git-branches")]
pub async fn get_branches(
    info: web::Query<GetBranchesRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let project_service = ProjectService::new();
    let project_name = &info.project_name;

    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(project_name);

    let project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError().json(GetBranchesResponse {
                success: false,
                message: format!("Failed to load project: {}", e),
                branches: None,
                current_project_branch: None,
                current_repo_branch: None,
            });
        }
    };

    if !project.git_integration_enabled {
        return HttpResponse::BadRequest().json(GetBranchesResponse {
            success: false,
            message: "Git integration is not enabled for this project.".to_string(),
            branches: None,
            current_project_branch: None,
            current_repo_branch: None,
        });
    }

    let repo = match GitService::open_repository(Path::new(&project.source_dir)) {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::InternalServerError().json(GetBranchesResponse {
                success: false,
                message: format!("Failed to open Git repository: {}", e),
                branches: None,
                current_project_branch: None,
                current_repo_branch: None,
            });
        }
    };

    let branches = match GitService::get_all_branch_names(&repo) {
        Ok(b) => Some(b),
        Err(e) => {
            return HttpResponse::InternalServerError().json(GetBranchesResponse {
                success: false,
                message: format!("Failed to get branch names: {}", e),
                branches: None,
                current_project_branch: None,
                current_repo_branch: None,
            });
        }
    };

    let current_repo_branch = match GitService::get_current_branch_name(&repo) {
        Ok(b) => Some(b),
        Err(e) => {
            eprintln!("Warning: Failed to get current repository branch name: {}", e);
            None
        }
    };

    HttpResponse::Ok().json(GetBranchesResponse {
        success: true,
        message: "Branches fetched successfully.".to_string(),
        branches,
        current_project_branch: project.git_branch_name,
        current_repo_branch,
    })
}