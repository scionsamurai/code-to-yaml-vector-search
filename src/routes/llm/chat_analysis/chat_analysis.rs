// src/routes/llm/chat_analysis/chat_analysis.rs

use super::models::*;
use super::utils::*;
use crate::models::{AppState, ChatMessage};
use crate::services::llm_service::{LlmService, LlmServiceConfig};
use crate::services::project_service::ProjectService;
use crate::services::git_service::{GitService, GitError};
use actix_web::{post, web, HttpResponse};
use std::path::Path;
use crate::services::utils::html_utils::{escape_html, unescape_html};
use std::env;

#[post("/chat-analysis")]
pub async fn chat_analysis(
    app_state: web::Data<AppState>,
    data: web::Json<ChatAnalysisRequest>,
) -> HttpResponse {
    let llm_service = LlmService::new();
    let project_service = ProjectService::new();

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&data.project);

    let project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to load project: {}", e))
        }
    };

    let query_id = data.query_id.as_deref().unwrap();

    // Load query data to get grounding_with_search and agentic_mode setting
    let query_data = match project_service.query_manager.load_query_data(&project_dir, query_id) {
        Ok(qd) => qd,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load query data: {}", e)),
    };
    let include_file_descriptions = query_data.include_file_descriptions;
    let enable_grounding = query_data.grounding_with_search;
    let agentic_mode_enabled = query_data.agentic_mode_enabled; // Get agentic_mode setting

    let git_branch_name = project.git_branch_name.clone().unwrap_or_default();

    let git_integration_enabled = project.git_integration_enabled;
    let auto_commit_for_chat = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "auto_commit").unwrap_or_else(|| "No previous query found".to_string());
    let mut commit_hash_for_user_message: Option<String> = None;

    // Get existing chat history from project structure *before* the current user message
    let full_history = get_full_history(&project, &app_state, &query_id);
    let mut unescaped_history: Vec<ChatMessage> = Vec::new();
    let mut hidden_context: Vec<String> = Vec::new();
    for message in full_history.iter() {
        if message.role == "git-flag" {
            continue;
        }
        let code = match (message.role.as_str(), message.hidden) {
            ("user", false) => "P",
            ("user", true) => "p",
            ("model", false) => "R",
            ("model", true) => "r",
            _ => "", // Handle unexpected roles
        };
        if !code.is_empty() {
            hidden_context.push(code.to_string());
        }
        let unescaped_content = unescape_html(message.content.clone());

        unescaped_history.push(ChatMessage {
            role: message.role.clone(),
            content: unescaped_content, // Content is now assumed to be raw markdown
            hidden: message.hidden,
            commit_hash: message.commit_hash.clone(), // Ensure commit_hash is carried over
            timestamp: message.timestamp,
            context_files: message.context_files.clone(),
            provider: message.provider.clone(),
            model: message.model.clone(),
            hidden_context: message.hidden_context.clone(),
            ..Default::default()
        });
    }

    let query_text = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "query").unwrap_or_else(|| "No previous query found".to_string());

    if git_integration_enabled {
        // Load project-specific .env for Git author/email
        if let Err(e) = project_service.load_project_env(&project_dir) {
            eprintln!("Warning: Failed to load project .env for Git author/email for project '{}': {}", data.project, e);
        }

        let git_author_name = env::var("GIT_AUTHOR_NAME").unwrap_or_else(|_| "LLM Assistant".to_string());
        let git_author_email = env::var("GIT_AUTHOR_EMAIL").unwrap_or_else(|_| "llm@example.com".to_string());

        let repo = match GitService::open_repository(&Path::new(&project.source_dir)) {
            Ok(r) => r,
            Err(GitError::Git2(e)) if e.code() == git2::ErrorCode::NotFound => {
                eprintln!("Git integration enabled for project '{}', but no Git repository found at {:?}. Skipping Git operations for this chat.", data.project, project_dir);
                return HttpResponse::InternalServerError().body(format!("Git integration enabled, but repository not found for project '{}'.", data.project));
            },
            Err(e) => {
                eprintln!("Failed to open Git repository for project '{}': {:?}", data.project, e);
                return HttpResponse::InternalServerError().body(format!("Failed to open Git repository"));
            }
        };

        let target_branch_name = if git_branch_name.is_empty() {
            GitService::get_default_branch_name(&repo).unwrap_or_else(|_| "main".to_string())
        } else {
            git_branch_name
        };

        if GitService::get_current_branch_name(&repo).unwrap_or_default() != target_branch_name {
            if let Err(e) = GitService::checkout_branch(&repo, &target_branch_name) {
                eprintln!("Failed to checkout branch '{}' for chat {}: {:?}", target_branch_name, query_id, e);
                return HttpResponse::InternalServerError().body(format!("Failed to checkout Git branch '{}'", target_branch_name));
            }
            println!("Checked out branch: {}", target_branch_name);
        } else {
            println!("Already on branch: {}", target_branch_name);
        }

        if auto_commit_for_chat == "true" {
            match GitService::has_uncommitted_changes(&repo) {
                Ok(true) => {
                    let commit_message = generate_commit_message(
                        &llm_service,
                        &repo,
                        &project,
                        &query_text,
                        &unescaped_history,
                    ).await;

                    match GitService::commit_changes(&repo, &git_author_name, &git_author_email, &commit_message) {
                        Ok(oid) => {
                            commit_hash_for_user_message = Some(oid.to_string());
                            println!("Auto-committed changes before LLM prompt: {}", oid);
                        },
                        Err(e) => eprintln!("Failed to auto-commit changes for chat '{}': {:?}", query_id, e),
                    }
                },
                Ok(false) => {
                    println!("No uncommitted changes for auto-commit in chat '{}'.", query_id);
                    if let Ok(latest_commit) = GitService::get_latest_commit(&repo) {
                        commit_hash_for_user_message = Some(latest_commit.id().to_string());
                    }
                },
                Err(e) => eprintln!("Failed to check for uncommitted changes in chat '{}': {:?}", query_id, e),
            }
        } else {
            if let Ok(latest_commit) = GitService::get_latest_commit(&repo) {
                commit_hash_for_user_message = Some(latest_commit.id().to_string());
            }
        }
    }

    // No HTML escaping needed for storing user message, store as raw markdown
    let user_message_content_raw = data.message.clone();
    let escaped_message = escape_html(data.message.clone()).await;

    // Choose logic based on agentic_mode_enabled
    let model_message_to_save = match handle_chat_message(
        &project,
        &app_state,
        query_id,
        &user_message_content_raw,
        enable_grounding,
        include_file_descriptions,
        &unescaped_history,
        commit_hash_for_user_message.clone(),
        hidden_context.clone(),
        agentic_mode_enabled,
    ).await {
        Ok(model_message) => model_message,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Chat message handling failed: {}", e));
        }
    };

    // Create user message to save (raw markdown), with the determined commit_hash
    let user_message_to_save = ChatMessage {
        role: "user".to_string(),
        content: escaped_message.to_string(), // Raw markdown
        hidden: false,
        commit_hash: commit_hash_for_user_message.clone(),
        timestamp: Some(chrono::Utc::now()),
        context_files: Some(vec!["file1.txt".to_string(), "file2.txt".to_string()]), // ADDED: Add two files to the context
        provider: Some(project.provider.clone()),
        model: project.specific_model.clone(),
        hidden_context: Some(hidden_context.clone()),
        ..Default::default()
    };


    // Add user message to chat history
    let user_message_new_id = project_service.chat_manager
        .add_chat_message(&project_service.query_manager,&project_dir, user_message_to_save, query_id, None) // parent_id will be set to current_node_id if not None
        .unwrap();

    // Add assistant message to chat history
    let assistant_message_to_save = model_message_to_save.clone();
    let assistant_message_new_id = project_service.chat_manager
        .add_chat_message(&project_service.query_manager,&project_dir, assistant_message_to_save, query_id, Some(user_message_new_id))
        .unwrap();

    // --- NEW: Update the QueryData's current_node_id to point to the new assistant message ---
    // The query_data was already loaded at the beginning, but we need to load it again
    // to ensure we have the very latest version before updating current_node_id and saving.
    let mut query_data_for_node_update = match project_service.query_manager.load_query_data(&project_dir, query_id) {
        Ok(qd) => qd,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to load query data for current_node_id update: {}", e)),
    };
    query_data_for_node_update.current_node_id = Some(assistant_message_new_id);
    if let Err(e) = project_service.query_manager.save_query_data(&project_dir, &query_data_for_node_update, query_id) {
        eprintln!("Failed to save query data after sending chat message: {}", e);
        return HttpResponse::InternalServerError().body(format!("Failed to update query's current node: {}", e));
    }
    // --- END NEW ---

    // Load the actual saved messages to return to the frontend, including their new UUIDs.
    let final_user_message = project_service.query_manager.get_chat_node(&project_dir, query_id, &user_message_new_id)
        .ok_or_else(|| HttpResponse::InternalServerError().body("Failed to retrieve saved user message.")).unwrap();
    let final_model_message = project_service.query_manager.get_chat_node(&project_dir, query_id, &assistant_message_new_id)
        .ok_or_else(|| HttpResponse::InternalServerError().body("Failed to retrieve saved model message.")).unwrap();


    // Return JSON response instead of redirect
    HttpResponse::Ok().json(ChatAnalysisResponse {
        success: true,
        user_message: final_user_message,
        model_message: final_model_message,
        new_current_node_id: assistant_message_new_id,
    })
}