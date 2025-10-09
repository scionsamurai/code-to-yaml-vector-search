// src/routes/git/status.rs

use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::services::project_service::ProjectService;
use crate::services::git_service::GitService;
use crate::models::AppState;
use std::path::Path;

#[derive(Deserialize)]
pub struct GitStatusRequest {
    project_name: String,
}

#[derive(Serialize)]
pub struct GitStatusResponse {
    success: bool,
    message: String,
    has_uncommitted_changes: bool,
    has_unpushed_commits: bool,
}

#[get("/git-status")]
pub async fn get_git_status(
    info: web::Query<GitStatusRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let project_service = ProjectService::new();
    let project_name = &info.project_name;

    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(project_name);

    let project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError().json(GitStatusResponse {
                success: false,
                message: format!("Failed to load project: {}", e),
                has_uncommitted_changes: false,
                has_unpushed_commits: false,
            });
        }
    };

    if !project.git_integration_enabled {
        return HttpResponse::Ok().json(GitStatusResponse {
            success: true,
            message: "Git integration is not enabled for this project.".to_string(),
            has_uncommitted_changes: false, // No changes if git not enabled
            has_unpushed_commits: false,   // No commits to push if git not enabled
        });
    }

    let repo = match GitService::open_repository(Path::new(&project.source_dir)) {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::InternalServerError().json(GitStatusResponse {
                success: false,
                message: format!("Failed to open Git repository: {}", e),
                has_uncommitted_changes: false,
                has_unpushed_commits: false,
            });
        }
    };

    let current_branch_name = match GitService::get_current_branch_name(&repo) {
        Ok(name) => name,
        Err(e) => {
            eprintln!("Failed to get current branch name for status check: {}", e);
            // Continue, but assume no unpushed commits if branch name cannot be determined
            // or return an error depending on desired strictness.
            // For simplicity here, we'll return an error.
             return HttpResponse::InternalServerError().json(GitStatusResponse {
                success: false,
                message: format!("Failed to determine current branch for status check: {}", e),
                has_uncommitted_changes: false,
                has_unpushed_commits: false,
            });
        }
    };

    let uncommitted_changes = match GitService::has_uncommitted_changes(&repo) {
        Ok(status) => status,
        Err(e) => {
            return HttpResponse::InternalServerError().json(GitStatusResponse {
                success: false,
                message: format!("Failed to check for uncommitted changes: {}", e),
                has_uncommitted_changes: false,
                has_unpushed_commits: false,
            });
        }
    };

    let unpushed_commits = match GitService::has_unpushed_commits(&repo, "origin", &current_branch_name) {
        Ok(status) => status,
        Err(e) => {
            // If there's an error checking for unpushed commits (e.g., no remote),
            // we might want to treat it as "unknown" or "potentially unpushed".
            // For now, let's treat it as an error.
            return HttpResponse::InternalServerError().json(GitStatusResponse {
                success: false,
                message: format!("Failed to check for unpushed commits: {}", e),
                has_uncommitted_changes: uncommitted_changes, // Keep uncommitted status
                has_unpushed_commits: false, // Assume false on error to avoid over-alerting
            });
        }
    };

    HttpResponse::Ok().json(GitStatusResponse {
        success: true,
        message: "Git status retrieved successfully.".to_string(),
        has_uncommitted_changes: uncommitted_changes,
        has_unpushed_commits: unpushed_commits,
    })
}