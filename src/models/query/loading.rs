// src/models/query/loading.rs
use crate::models::{AppState, Project, QueryData};
use actix_web::web;
use crate::services::project_service::ProjectService;

impl Project {
    // Modified function to load a specific query data
    pub fn load_query_data_by_filename(
        &self,
        app_state: &web::Data<AppState>,
        filename: &str,
    ) -> Result<Option<QueryData>, String> {
        let project_service = ProjectService::new();
        let project_dir = self.get_project_dir(app_state);

        match project_service.load_query_data(&project_dir, &filename) {
            Ok(query_data) => Ok(Some(query_data)),
            Err(e) => {
                println!("filename: {}", filename);
                eprintln!("Error loading query data: {}", e);
                self.load_most_recent_query_data(app_state) // Fallback to most recent query data
            }
        }

    }

    pub fn load_most_recent_query_data(
        &self,
        app_state: &web::Data<AppState>,
    ) -> Result<Option<QueryData>, String> {
        let project_service = ProjectService::new();
        let project_dir = self.get_project_dir(app_state);

        // Load the most recent query data
        match project_service.get_most_recent_query_file(&project_dir) {
            Ok(most_recent_file) => {
                match most_recent_file {
                    Some(file_path) => {
                        let file_name =
                            file_path.file_name().unwrap().to_str().unwrap().to_string();
                        match project_service.load_query_data(&project_dir, &file_name) {
                            Ok(query_data) => Ok(Some(query_data)),
                            Err(e) => {
                                eprintln!("Error loading query data: {}", e);
                                Ok(None) // Return None if loading fails, handle appropriately in caller
                            }
                        }
                    }
                    None => Ok(None), // No recent file
                }
            }
            Err(e) => {
                eprintln!("Error getting most recent query file: {}", e);
                Err(e) // Propagate the error
            }
        }
    }
}