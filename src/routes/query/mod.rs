pub mod update_auto_commit;


use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(update_auto_commit::update_auto_commit);
}