// src/routes/git/merge_branch.rs

use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::services::project_service::ProjectService;
use crate::services::git_service::{GitService, GitError};
use std::path::Path;
use crate::models::AppState;
use std::env;

#[derive(Deserialize)]
pub struct MergeBranchRequest {
    project_name: String,
}

#[derive(Serialize)]
pub struct MergeBranchResponse {
    success: bool,
    message: String,
}

#[post("/merge-git-branch")]
pub async fn merge_branch(
    data: web::Json<MergeBranchRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let project_service = ProjectService::new();
    let project_name = &data.project_name;

    let git_author_name = env::var("GIT_AUTHOR_NAME").unwrap_or_else(|_| "LLM Assistant".to_string());
    let git_author_email = env::var("GIT_AUTHOR_EMAIL").unwrap_or_else(|_| "llm@example.com".to_string());

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(project_name);

    let mut project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError().json(MergeBranchResponse {
                success: false,
                message: format!("Failed to load project: {}", e),
            });
        }
    };

    if !project.git_integration_enabled {
        return HttpResponse::BadRequest().json(MergeBranchResponse {
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
            return HttpResponse::InternalServerError().json(MergeBranchResponse {
                success: false,
                message: format!("Failed to open Git repository: {}", e),
            });
        }
    };

    let chat_branch_name = match project.git_branch_name.clone() {
        Some(branch_name) => branch_name,
        None => {
            return HttpResponse::BadRequest().json(MergeBranchResponse {
                success: false,
                message: "No chat branch associated with this project.".to_string(),
            });
        }
    };

    // Checkout the default branch (e.g., "main")
    let default_branch_name = match GitService::get_default_branch_name(&repo) {
        Ok(branch_name) => branch_name,
        Err(_) => "main".to_string(), // Fallback to "main" if can't determine default
    };

    if let Err(e) = GitService::checkout_branch(&repo, &default_branch_name) {
        return HttpResponse::InternalServerError().json(MergeBranchResponse {
            success: false,
            message: format!("Failed to checkout default branch ({}): {}", default_branch_name, e),
        });
    }

    // Attempt to merge the chat branch
    match GitService::merge_branch(&repo, &chat_branch_name, &git_author_name, &git_author_email) {
        Ok(_) => {
            // Merge successful, delete the chat branch
            if let Err(e) = GitService::delete_branch(&repo, &chat_branch_name) {
                eprintln!("Warning: Failed to delete branch '{}': {}", chat_branch_name, e);
                // Don't fail the whole operation if delete fails
            }


            let remote_name = "origin"; // Assuming "origin" is the remote name
            if let Err(e) = GitService::delete_remote_branch(&repo, remote_name, &chat_branch_name) {
                eprintln!("Warning: Failed to delete remote branch '{}' on remote '{}': {}", chat_branch_name, remote_name, e);
                // Log but don't fail; local state is updated, and default branch push can still happen.
            }

            // Update project's git_branch_name to None
            project.git_branch_name = None;

            // Save the updated project
            if let Err(e) = project_service.save_project(&project, &project_dir) {
                return HttpResponse::InternalServerError().json(MergeBranchResponse {
                    success: false,
                    message: format!("Failed to save project: {}", e),
                });
            }

            HttpResponse::Ok().json(MergeBranchResponse {
                success: true,
                message: format!("Branch '{}' merged into '{}' and deleted successfully.", chat_branch_name, default_branch_name),
            })
        }
        Err(GitError::Other(msg)) if msg == "Merge conflicts detected" => {
            HttpResponse::Conflict().json(MergeBranchResponse {
                success: false,
                message: "Merge failed due to conflicts. Please resolve conflicts in your project directory manually.".to_string(),
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(MergeBranchResponse {
                success: false,
                message: format!("Failed to merge branch: {}", e),
            })
        }
    }
}
