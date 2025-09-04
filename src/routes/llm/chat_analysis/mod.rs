// src/routes/llm/chat_analysis/mod.rs
pub mod handlers;
pub mod models;
pub mod utils;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::chat_analysis)
        .service(handlers::reset_analysis_chat)
        .service(handlers::update_message_visibility)
        .service(handlers::update_chat_message)
        .service(handlers::regenerate_chat_message);
}