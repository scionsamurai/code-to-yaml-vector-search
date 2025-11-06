// src/models/utils.rs
use crate::models::{AppState, Project};
use actix_web::web;
use std::path::{Path, PathBuf};

impl Project {
    // Get project directory
    pub fn _get_project_dir(&self, app_state: &web::Data<AppState>) -> PathBuf {
        let output_dir = Path::new(&app_state.output_dir);
        output_dir.join(&self.name)
    }
}