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
    for message in full_history.iter() {
        let unescaped_content = unescape_html(message.content.clone());
        unescaped_history.push(ChatMessage {
            role: message.role.clone(),
            content: unescaped_content,
            hidden: message.hidden,
            commit_hash: message.commit_hash.clone(), // Ensure commit_hash is carried over
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
                    // --- Dynamic Commit Message Generation ---
                    let mut commit_llm_messages: Vec<ChatMessage> = Vec::new();

                    // Get the git diff for uncommitted changes
                    let git_diff_output = match GitService::get_uncommitted_diff(&repo) {
                        Ok(diff) => diff,
                        Err(e) => {
                            eprintln!("Failed to get uncommitted diff for commit message generation: {:?}", e);
                            // Fallback to a simpler message or continue without diff
                            "Could not retrieve code changes for detailed commit message.".to_string()
                        }
                    };

                    // Determine relevant chat history since the last recorded commit
                    let mut relevant_history_for_commit_msg: Vec<ChatMessage> = Vec::new();
                    let mut last_commit_idx: Option<usize> = None;

                    // Find the index of the last message in history that had a commit_hash
                    for (idx, msg) in unescaped_history.iter().enumerate() {
                        if msg.commit_hash.is_some() {
                            last_commit_idx = Some(idx);
                        }
                    }

                    if let Some(idx) = last_commit_idx {
                        // Collect all messages *after* the one with the last commit hash
                        // This assumes the commit message should reflect the changes
                        // that occurred as a result of the conversation *since* that last commit.
                        if idx + 1 < unescaped_history.len() {
                            relevant_history_for_commit_msg.extend_from_slice(&unescaped_history[idx + 1..]);
                        }
                    } else {
                        // If no commit hash found in history, all history is relevant
                        // (implies this might be the first commit for this query)
                        relevant_history_for_commit_msg.extend_from_slice(&unescaped_history);
                    }

                    let formatted_relevant_history = if !relevant_history_for_commit_msg.is_empty() {
                        relevant_history_for_commit_msg.iter()
                            .map(|msg| format!("{}: {}", msg.role, msg.content))
                            .collect::<Vec<String>>()
                            .join("\n")
                    } else {
                        "No relevant chat history since the last tracked commit.".to_string()
                    };

                    let mut user_messsage_for_commit = String::new();
           
                    user_messsage_for_commit.push_str("You are an AI assistant tasked with generating concise Git commit messages. Your response *must* be only the commit message itself, with no 'Auto:' prefix, conversational text, confirmations of compliance, or extraneous information. The message should be a short, descriptive summary (under 100 characters) of the provided code changes and the conversation that led to them. Focus on the core change or task requested in the previous interaction.");
                    // Add the initial query for overall context
                    user_messsage_for_commit.push_str(&format!("Initial chat query for context: {}\n", query_text));
    
                    // Add the relevant chat history
                    user_messsage_for_commit.push_str(&format!("Relevant chat history leading to these changes:\n{}\n", formatted_relevant_history));

                    // Add the code changes (git diff)
                    user_messsage_for_commit.push_str(&format!("Here are the uncommitted code changes:\n```diff\n{}\n```", git_diff_output));

                    // Final instruction to generate the message
                    user_messsage_for_commit.push_str("\nBased on the above, provide the summarized Git commit message.");
                    
                    commit_llm_messages.push(ChatMessage {
                        role: "user".to_string(),
                        content: user_messsage_for_commit,
                        hidden: false,
                        commit_hash: None,
                    });

                    let generated_message_llm_response = llm_service
                        .send_conversation(
                            &commit_llm_messages,
                            &project.provider.clone(),
                            project.specific_model.as_deref(),
                        )
                        .await;

                    // Clean up the LLM response (e.g., remove quotes or unwanted formatting)
                    let cleaned_commit_message = generated_message_llm_response
                        .trim_matches(|c| c == '"' || c == '\'') // Remove surrounding quotes
                        .lines()
                        .next() // Take only the first line for conciseness
                        .unwrap_or("Generated commit message failed.")
                        .trim() // Trim any remaining whitespace
                        .to_string();

                    // Ensure it starts with "Auto:" as requested by the user
                    let commit_message = format!("Auto: {}", cleaned_commit_message);
                    // --- End Dynamic Commit Message Generation ---

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

    // Create user message to save, with the determined commit_hash
    let user_message_to_save = ChatMessage {
        role: "user".to_string(),
        content: escaped_message.to_string(),
        hidden: false,
        commit_hash: commit_hash_for_user_message.clone(), // Assign the commit hash here
    };

    // Format messages for LLM with system prompt and existing history + new user message
    let messages = format_messages_for_llm(&system_prompt, &unescaped_history, &user_message_to_save);

    // Send to LLM
    let llm_response = llm_service
        .send_conversation(&messages, &project.provider.clone(), project.specific_model.as_deref())
        .await;

    // Create response message (LLM response doesn't directly cause a commit here)
    let assistant_message = ChatMessage {
        role: "model".to_string(),
        content: llm_response.clone(),
        hidden: false,
        commit_hash: commit_hash_for_user_message.clone(), // Associate LLM response with same commit
    };

    // Add messages to chat
    project_service.chat_manager
        .add_chat_message(&project_service.query_manager,&project_dir, user_message_to_save, query_id)
        .unwrap();
    project_service.chat_manager
        .add_chat_message(&project_service.query_manager,&project_dir, assistant_message, query_id)
        .unwrap();

    HttpResponse::Ok().body(llm_response)
}