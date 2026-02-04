// src/routes/project/mod.rs
pub mod create;
pub mod update_yaml;
pub mod delete;
pub mod get_project;
pub mod update_settings;
pub mod path_comment;
pub mod update_file_yaml_override;
pub mod cluster;
pub mod git_env_settings;
pub mod submit_query;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create::create)
        .service(get_project::get_project)
        .service(update_yaml::update)
        .service(delete::delete)
        .service(path_comment::validate_paths)
        .service(update_settings::update_settings)
        .service(update_file_yaml_override::update_file_yaml_override)
        .service(cluster::cluster_project_embeddings)
        .service(git_env_settings::get_git_env_settings)
        .service(git_env_settings::post_git_env_settings)
        .service(submit_query::submit_query);
}