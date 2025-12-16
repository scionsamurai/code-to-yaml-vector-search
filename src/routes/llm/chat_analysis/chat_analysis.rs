// src/routes/llm/chat_analysis/chat_analysis.rs
use super::models::*;
use super::utils::*;
use crate::models::{AppState, ChatMessage};
use crate::services::llm_service::LlmService;
use crate::services::project_service::ProjectService;
use crate::services::git_service::{GitService, GitError};
use actix_web::{post, web, HttpResponse};
use std::path::Path;
use crate::services::utils::html_utils::{escape_html, unescape_html};
use std::env;
use serde_json::json; // <--- ADD THIS LINE to allow creating JSON response

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

    let include_file_descriptions = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "include_file_descriptions").unwrap_or_else(|| "false".to_string()) == "true";

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
            // id and parent_id are part of the stored message, but we're re-creating here.
            // When passing to LLM, these are not directly used, but the overall structure matters.
            // For the LLM context, it's just content and role.
            role: message.role.clone(),
            content: unescaped_content,
            hidden: message.hidden,
            commit_hash: message.commit_hash.clone(), // Ensure commit_hash is carried over
            timestamp: message.timestamp,
            context_files: message.context_files.clone(),
            provider: message.provider.clone(),
            model: message.model.clone(),
            hidden_context: message.hidden_context.clone(),
            ..Default::default() // Fill id and parent_id from default (though they'll be ignored for LLM prompt)
        });
    }

    let query_text = project_service.query_manager.get_query_data_field(&project_dir, &query_id, "query").unwrap_or_else(|| "No previous query found".to_string());

    if git_integration_enabled {
        // Load project-specific .env for Git author/email
        if let Err(e) = project_service.load_project_env(&project_dir) {
            eprintln!("Warning: Failed to load project .env for Git author/email for project '{}': {}", data.project, e);
            // This is a warning. Git operations will proceed with defaults or fail if env vars are required.
        }

        let git_author_name = env::var("GIT_AUTHOR_NAME").unwrap_or_else(|_| "LLM Assistant".to_string());
        let git_author_email = env::var("GIT_AUTHOR_EMAIL").unwrap_or_else(|_| "llm@example.com".to_string());

        let repo = match GitService::open_repository(&Path::new(&project.source_dir)) {
            Ok(r) => r,
            Err(GitError::Git2(e)) if e.code() == git2::ErrorCode::NotFound => {
                eprintln!("Git integration enabled for project '{}', but no Git repository found at {:?}. Skipping Git operations for this chat.", data.project, project_dir);
                // We could try to init here, but per your initial plan, we expect repo to exist.
                // If the project level setting is enabled, but no repo, this is an error state.
                return HttpResponse::InternalServerError().body(format!("Git integration enabled, but repository not found for project '{}'.", data.project));
            },
            Err(e) => {
                eprintln!("Failed to open Git repository for project '{}': {:?}", data.project, e);
                return HttpResponse::InternalServerError().body(format!("Failed to open Git repository"));
            }
        };

        // If a branch is active for this query, ensure we're on it.
        // If no specific branch for this chat, ensure we are on the default branch (e.g., main)
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
                    // Ensure it starts with "Auto:" as requested by the user
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
                    // Still assign the latest commit hash if there was a previous commit,
                    // so the message is linked to a valid state.
                    if let Ok(latest_commit) = GitService::get_latest_commit(&repo) {
                        commit_hash_for_user_message = Some(latest_commit.id().to_string());
                    }
                },
                Err(e) => eprintln!("Failed to check for uncommitted changes in chat '{}': {:?}", query_id, e),
            }
        } else {
            // If auto-commit is off, but git is enabled, still try to associate with latest commit
            if let Ok(latest_commit) = GitService::get_latest_commit(&repo) {
                commit_hash_for_user_message = Some(latest_commit.id().to_string());
            }
        }
    }


    // Get selected context files and file contents
    let (context_files, file_contents) = get_context_and_contents(&project, &app_state, &query_id);

    // Create context prompt with the loaded file contents, project, and description flag
    let system_prompt = create_system_prompt(&query_text, &context_files, &file_contents, &project, include_file_descriptions);

    // Escape the user's message
    let escaped_message = escape_html(data.message.clone()).await;

    // Create user message for LLM (unescaped)
    let user_message_for_llm = ChatMessage {
        role: "user".to_string(),
        content: data.message.clone(),
        hidden: false,
        commit_hash: commit_hash_for_user_message.clone(),
        timestamp: Some(chrono::Utc::now()), // Set timestamp here
        context_files: Some(context_files.clone()),
        provider: Some(project.provider.clone()),
        model: project.specific_model.clone(),
        hidden_context: Some(hidden_context.clone()),
        ..Default::default() // Use default for id and parent_id. These will be overwritten by add_chat_message.
    };

    // Create user message to save (escaped), with the determined commit_hash
    let user_message_to_save = ChatMessage {
        role: "user".to_string(),
        content: escaped_message.to_string(),
        hidden: false,
        commit_hash: commit_hash_for_user_message.clone(),
        timestamp: user_message_for_llm.timestamp, // Use the same timestamp
        context_files: user_message_for_llm.context_files.clone(),
        provider: user_message_for_llm.provider.clone(),
        model: user_message_for_llm.model.clone(),
        hidden_context: user_message_for_llm.hidden_context.clone(),
        ..Default::default() // Use default for id and parent_id. These will be overwritten by add_chat_message.
    };

    // Format messages for LLM with system prompt and existing history + new user message
    let messages = format_messages_for_llm(&system_prompt, &unescaped_history, &user_message_for_llm);

    // Send to LLM
    let llm_response = llm_service
        .send_conversation(&messages, &project.provider.clone(), project.specific_model.as_deref())
        .await;

    // Create assistant message
    let assistant_message_to_save = ChatMessage { // Renamed variable to avoid conflict
        role: "model".to_string(),
        content: llm_response.clone(),
        hidden: false,
        commit_hash: commit_hash_for_user_message.clone(), // Associate LLM response with same commit
        timestamp: Some(chrono::Utc::now()), // Set timestamp here
        context_files: Some(context_files.clone()),
        provider: Some(project.provider.clone()),
        model: project.specific_model.clone(),
        hidden_context: Some(hidden_context), 
        ..Default::default() // Use default for id and parent_id. These will be overwritten by add_chat_message.
    };

    // Add user message to chat history
    // For regular conversation flow, parent_id_override is None, so it appends to current_node_id
    project_service.chat_manager
        .add_chat_message(&project_service.query_manager,&project_dir, user_message_to_save, query_id, None) // <--- UPDATED CALL
        .unwrap();
    
    // Add assistant message to chat history
    // Its parent will be the newly added user message. We need the ID of the user message to be explicit,
    // but since add_chat_message already sets `current_node_id` to the user message, this next call
    // will implicitly link to the user message unless we want to override.
    // However, the `add_chat_message` function returns the ID of the message it just added,
    // which is what we need for the frontend response.
    let model_message_id_for_response = project_service.chat_manager
        .add_chat_message(&project_service.query_manager,&project_dir, assistant_message_to_save, query_id, None) // <--- UPDATED CALL
        .unwrap();

    // Return the content AND the ID of the model message
    HttpResponse::Ok().json(json!({
        "message_id": model_message_id_for_response,
        "content": llm_response
    }))
}