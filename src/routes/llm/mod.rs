// src/routes/llm/mod.rs
mod chat_split;
mod suggest_split;
mod regenerate_yaml;
pub mod chat_analysis;
mod update_analysis_context;
mod update_analysis_query;
mod update_analysis_title;
mod search_files;
mod optimize_prompt;
mod get_branch_data;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(chat_split::chat_split)
        .service(suggest_split::suggest_split)
        .service(regenerate_yaml::regenerate_yaml)
        .service(update_analysis_context::update_analysis_context)
        .service(update_analysis_query::update_analysis_query)
        .service(update_analysis_title::update_analysis_title)
        .service(optimize_prompt::optimize_prompt_route)
        .configure(chat_analysis::configure)
        .service(search_files::search_related_files)
        .service(get_branch_data::get_branching_data);
}