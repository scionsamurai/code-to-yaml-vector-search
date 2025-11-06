// src/routes/git/commit_changes.rs

use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::services::project_service::ProjectService;
use crate::services::git_service::GitService;
use crate::models::{AppState, ChatMessage}; // Import ChatMessage
use crate::services::project_service::chat_management::ChatManager; // Import ChatManager
use crate::services::project_service::query_management::QueryManager; // Import QueryManager
use std::path::Path;
use std::env;
use chrono::Utc; // Import Utc for timestamp

#[derive(Deserialize)]
pub struct CommitChangesRequest {
    project_name: String,
    message: String,
    query_id: String, // Add query_id here
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
    let chat_manager = ChatManager::new(); // Initialize ChatManager
    let query_manager = QueryManager::new(); // Initialize QueryManager

    let project_name = &data.project_name;
    let commit_message = &data.message;
    let query_id = &data.query_id; // Get query_id from the request

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
                    let commit_hash_str = oid.to_string();

                    // Create the new git-flag message
                    let git_flag_message = ChatMessage {
                        role: "git-flag".to_string(),
                        content: "".to_string(), // Empty as requested
                        hidden: true,           // Hidden as requested
                        commit_hash: Some(commit_hash_str.clone()),
                        timestamp: Some(Utc::now()),
                        context_files: None,    // Empty/default as requested
                        provider: None,         // Empty/default as requested
                        model: None,            // Empty/default as requested
                        hidden_context: None,   // Empty/default as requested
                    };

                    // Add the git-flag message to the chat history
                    if let Err(e) = chat_manager.add_chat_message(
                        &query_manager,
                        &project_dir,
                        git_flag_message,
                        &query_id,
                    ) {
                        eprintln!("Failed to add git-flag chat message for project '{}', query '{}': {}", project_name, query_id, e);
                        // Log the error but proceed as the Git commit was successful.
                        // Depending on requirements, this could be a fatal error, but for
                        // chat history persistence, it's often handled as a soft failure.
                    }

                    HttpResponse::Ok().json(CommitChangesResponse {
                        success: true,
                        message: format!("Changes committed successfully: {}", commit_hash_str),
                        commit_hash: Some(commit_hash_str),
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
            // No uncommitted changes, so no git-flag message is added.
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