// src/routes/project/mod.rs
pub mod create;
pub mod update_yaml;
pub mod delete;
pub mod get_project;
pub mod update_settings;
pub mod path_comment;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create::create)
        .service(get_project::get_project)
        .service(update_yaml::update)
        .service(delete::delete)
        .service(path_comment::validate_paths)
        .service(update_settings::update_settings);
}