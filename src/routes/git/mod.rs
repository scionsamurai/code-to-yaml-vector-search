pub mod create_branch;
pub mod merge_branch;
pub mod get_branches;
pub mod checkout_branch;
pub mod commit_changes;
pub mod push_changes;
pub mod status;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
   cfg.service(create_branch::create_branch);
   cfg.service(merge_branch::merge_branch);
   cfg.service(get_branches::get_branches);
   cfg.service(checkout_branch::checkout_branch);
   cfg.service(commit_changes::commit_changes);
   cfg.service(push_changes::push_changes);
   cfg.service(status::get_git_status);
}