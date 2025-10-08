// src/routes/git/commit_changes.rs

use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::services::project_service::ProjectService;
use crate::services::git_service::{GitService, GitError};
use crate::models::AppState;
use std::path::Path;
use std::env;

#[derive(Deserialize)]
pub struct CommitChangesRequest {
    project_name: String,
    message: String,
}

#[derive(Serialize)]
pub struct CommitChangesResponse {
    success: bool,
    message: String,
    commit_hash: Option<String>,
}

#[post("/commit-changes")]
pub async fn commit_changes(
    data: web::Json<CommitChangesRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let project_service = ProjectService::new();
    let project_name = &data.project_name;
    let commit_message = &data.message;

    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(project_name);

    let project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError().json(CommitChangesResponse {
                success: false,
                message: format!("Failed to load project: {}", e),
                commit_hash: None,
            });
        }
    };

    if !project.git_integration_enabled {
        return HttpResponse::BadRequest().json(CommitChangesResponse {
            success: false,
            message: "Git integration is not enabled for this project.".to_string(),
            commit_hash: None,
        });
    }

    if let Err(e) = project_service.load_project_env(&project_dir) {
        eprintln!("Warning: Failed to load project .env for Git author/email for project '{}': {}", project_name, e);
    }

    let git_author_name = env::var("GIT_AUTHOR_NAME").unwrap_or_else(|_| "".to_string());
    let git_author_email = env::var("GIT_AUTHOR_EMAIL").unwrap_or_else(|_| "".to_string());

    let repo = match GitService::open_repository(Path::new(&project.source_dir)) {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::InternalServerError().json(CommitChangesResponse {
                success: false,
                message: format!("Failed to open Git repository: {}", e),
                commit_hash: None,
            });
        }
    };

    match GitService::has_uncommitted_changes(&repo) {
        Ok(true) => {
            match GitService::commit_changes(&repo, &git_author_name, &git_author_email, commit_message) {
                Ok(oid) => {
                    HttpResponse::Ok().json(CommitChangesResponse {
                        success: true,
                        message: format!("Changes committed successfully: {}", oid),
                        commit_hash: Some(oid.to_string()),
                    })
                },
                Err(e) => {
                    HttpResponse::InternalServerError().json(CommitChangesResponse {
                        success: false,
                        message: format!("Failed to commit changes: {}", e),
                        commit_hash: None,
                    })
                }
            }
        },
        Ok(false) => {
            HttpResponse::Ok().json(CommitChangesResponse {
                success: true,
                message: "No uncommitted changes to commit.".to_string(),
                commit_hash: None,
            })
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(CommitChangesResponse {
                success: false,
                message: format!("Failed to check for uncommitted changes: {}", e),
                commit_hash: None,
            })
        }
    }
}