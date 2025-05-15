// src/models/query_management.rs
use crate::models::{AppState, Project};
use actix_web::web;

impl Project {
    pub fn update_query_title(
        &self,
        app_state: &web::Data<AppState>,
        query_filename: &str,
        new_title: &str,
    ) -> Result<(), String> {
        self.update_query_data(app_state, query_filename, |query_data| {
            query_data.title = Some(new_title.to_string());
        })
    }
}