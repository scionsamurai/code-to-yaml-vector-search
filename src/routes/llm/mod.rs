// src/routes/llm/mod.rs
mod chat_split;
mod suggest_split;
mod execute_query;
mod regenerate_yaml;
mod analyze_query;
pub mod chat_analysis;
mod update_analysis_context;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(chat_split::chat_split)
        .service(suggest_split::suggest_split)
        .service(regenerate_yaml::regenerate_yaml)
        .service(analyze_query::analyze_query)
        .service(update_analysis_context::update_analysis_context);
}