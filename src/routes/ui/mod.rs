// src/routes/ui/mod.rs
mod home;
mod update_env;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(home::home)
        .service(update_env::update_env);
}