// src/routes/project/mod.rs
pub mod create;
pub mod update_yaml;
pub mod delete;
pub mod get_project;
pub mod update_settings;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create::create)
        .service(get_project::get_project)
        .service(update_yaml::update)
        .service(delete::delete)
        .service(update_settings::update_settings);
}