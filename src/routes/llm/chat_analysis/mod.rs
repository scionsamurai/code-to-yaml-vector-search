// src/routes/llm/chat_analysis/mod.rs
pub mod chat_analysis;
pub mod regenerate_chat_message;
pub mod reset_analysis_chat;
pub mod update_chat_message;
pub mod update_message_visibility;
pub mod apply_code_to_file; 
pub mod utils;
pub mod models;
pub mod suggest_branch_name;

use actix_web::web;

#[derive(Clone, Debug, Default)]
struct ChatMessageMetadata {
    timestamp: Option<chrono::DateTime<chrono::Utc>>,
    context_files: Option<Vec<String>>,
    provider: Option<String>,
    model: Option<String>,
    hidden_context: Option<Vec<String>>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(chat_analysis::chat_analysis)
        .service(reset_analysis_chat::reset_analysis_chat)
        .service(update_message_visibility::update_message_visibility)
        .service(update_chat_message::update_chat_message)
        .service(apply_code_to_file::apply_code_to_file) // New service for applying code to a file
        .service(regenerate_chat_message::regenerate_chat_message)
        .service(suggest_branch_name::suggest_branch_name);

}