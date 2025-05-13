// src/models/query_management.rs
use crate::models::{AppState, Project, QueryData};
use actix_web::web;
use crate::services::project_service::ProjectService;

impl Project {
    pub fn load_most_recent_query_data(
        &self,
        app_state: &web::Data<AppState>,
    ) -> Result<Option<QueryData>, String> {
        let project_service = ProjectService::new();
        let project_dir = self.get_project_dir(app_state);

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

    pub fn get_vector_results(&self, app_state: &web::Data<AppState>) -> Vec<String> {
        match self.load_most_recent_query_data(app_state) {
            Ok(Some(query_data)) => {
                query_data.vector_results
                    .into_iter()
                    .map(|(path, _)| path)
                    .collect()
            }
            Ok(None) => Vec::new(), // No query data found
            Err(_e) => Vec::new(),  // Error occurred, return empty vector
        }
    }

    pub fn get_context_files(&self, app_state: &web::Data<AppState>) -> Vec<String> {
        match self.load_most_recent_query_data(app_state) {
            Ok(Some(query_data)) => query_data.context_files,
            Ok(None) => Vec::new(), // No query data found
            Err(_e) => Vec::new(),  // Error occurred, return empty vector
        }
    }

    pub fn get_query_text(&self, app_state: &web::Data<AppState>) -> Option<String> {
        match self.load_most_recent_query_data(app_state) {
            Ok(Some(query_data)) => Some(query_data.query),
            Ok(None) => None, // No query data found
            Err(_e) => None,  // Error occurred, return None
        }
    }

    pub fn save_query_data(
        &self,
        app_state: &web::Data<AppState>,
        query_data: QueryData,
    ) -> Result<(), String> {
        let project_service = ProjectService::new();
        let project_dir = self.get_project_dir(app_state);
        let filename = project_service.generate_query_filename();

        project_service.save_query_data(&project_dir, &query_data, &filename)
    }

    // Update an existing query data or create a new one
    pub fn update_query_data(
        &self,
        app_state: &web::Data<AppState>,
        update_fn: impl FnOnce(&mut QueryData),
    ) -> Result<(), String> {
        let project_service = ProjectService::new();
        let project_dir = self.get_project_dir(app_state);

        // Try to load existing query data or create new
        let (mut query_data, filename) = match self.load_most_recent_query_data(app_state) {
            Ok(Some(qd)) => {
                // Get filename of most recent file
                let file_path = project_service
                    .get_most_recent_query_file(&project_dir)
                    .map_err(|e| format!("Failed to get filename: {}", e))?
                    .ok_or_else(|| "No query file found".to_string())?;
                let filename = file_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .ok_or_else(|| "Invalid filename".to_string())?
                    .to_string();
                (qd, filename)
            }
            _ => {
                // Create new query data and filename
                (
                    QueryData::default(),
                    project_service.generate_query_filename(),
                )
            }
        };

        // Apply the update function
        update_fn(&mut query_data);

        // Save the updated query data
        project_service.save_query_data(&project_dir, &query_data, &filename)
    }
}