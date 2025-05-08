pub mod handlers;
pub mod models;
pub mod utils;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::chat_analysis)
        .service(handlers::reset_analysis_chat)
        .service(handlers::save_analysis_history)
        .service(handlers::update_chat_message);
}