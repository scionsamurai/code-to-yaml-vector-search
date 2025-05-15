// src/models/query_management.rs
use crate::models::{AppState, Project, QueryData};
use actix_web::web;
use crate::services::project_service::ProjectService;
use std::path::PathBuf;
use std::fs::read_dir;

impl Project {
    // function to get a list of query data filenames, sorted by creation date
    pub fn get_query_filenames(
        &self,
        app_state: &web::Data<AppState>,
    ) -> Result<Vec<(String, String)>, String> { // (filename, timestamp)
        let project_service = ProjectService::new();
        let project_dir = self.get_project_dir(app_state);
        let queries_dir = project_service.get_queries_dir(&project_dir); // Get queries directory

        if !queries_dir.exists() {
            return Ok(Vec::new()); // Return empty vector if directory doesn't exist
        }

        let mut files: Vec<PathBuf> = read_dir(queries_dir) // Use queries_dir
            .map_err(|e| format!("Failed to read queries directory: {}", e))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| {
                path.is_file() && path.extension().map_or(false, |ext| ext == "json") // Check for .json extension
            })
            .collect();

        // Sort files by modified time (creation time is not readily available)
        files.sort_by(|a, b| {
            a.metadata()
                .unwrap()
                .modified()
                .unwrap()
                .cmp(&b.metadata().unwrap().modified().unwrap())
        });

        let filenames: Vec<(String, String)> = files.iter().map(|path| {
                let filename = path.file_name().unwrap().to_str().unwrap().to_string();
                // Extract timestamp or identifier from the filename (UUID in this case)
                let timestamp = filename.trim_end_matches(".json").to_string(); // Remove .json extension

                (filename, timestamp)
            }).collect();

        Ok(filenames)
    }


    // Get the ID of the most recent query file
    pub fn get_recent_query_id(
        &self,
        app_state: &web::Data<AppState>,
    ) -> Option<String> {
        match self.get_query_filenames(app_state) {
            Ok(filenames) => {
                if filenames.is_empty() {
                    None
                } else {
                    // The filenames are already sorted by modification time,
                    // so the last one is the most recent
                    let most_recent = filenames.last().unwrap();
                    // Return the timestamp/ID part (second element in the tuple)
                    Some(most_recent.1.clone() + ".json")
                }
            },
            Err(e) => {
                eprintln!("Failed to get recent query ID: {}", e);
                None
            }
        }
    }

    // Modified function to load a specific query data
    pub fn load_query_data_by_filename(
        &self,
        app_state: &web::Data<AppState>,
        filename: &str,
    ) -> Result<Option<QueryData>, String> {
        let project_service = ProjectService::new();
        let project_dir = self.get_project_dir(app_state);

        match project_service.load_query_data(&project_dir, filename) {
            Ok(query_data) => Ok(Some(query_data)),
            Err(e) => {
                eprintln!("Error loading query data: {}", e);
                Ok(None) // Return None if loading fails, handle appropriately in caller
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

    pub fn get_vector_results(&self, app_state: &web::Data<AppState>, query_filename: &str) -> Vec<String> {
        match self.load_query_data_by_filename(app_state, query_filename) {
            Ok(Some(query_data)) => {
                query_data.vector_results
                    .into_iter()
                    .map(|(path, _)| path)
                    .collect()
            }
            Ok(None) => Vec::new(),
            Err(_e) => Vec::new(),
        }
    }

    pub fn get_context_files(&self, app_state: &web::Data<AppState>, query_filename: &str) -> Vec<String> {
        match self.load_query_data_by_filename(app_state, query_filename) {
            Ok(Some(query_data)) => query_data.context_files,
            Ok(None) => Vec::new(),
            Err(_e) => Vec::new(),
        }
    }

    pub fn get_query_text(&self, app_state: &web::Data<AppState>, query_filename: &str) -> Option<String> {
        match self.load_query_data_by_filename(app_state, query_filename) {
            Ok(Some(query_data)) => Some(query_data.query),
            Ok(None) => None,
            Err(_e) => None,
        }
    }

    pub fn save_new_query_data(
        &self,
        app_state: &web::Data<AppState>,
        query_data: QueryData,
    ) -> String {
        let project_service = ProjectService::new();
        let project_dir = self.get_project_dir(app_state);
        let filename = project_service.generate_query_filename();

        let _ = project_service.save_query_data(&project_dir, &query_data, &filename);
        filename
    }

    // Update an existing query data or create a new one
    pub fn update_query_data(
        &self,
        app_state: &web::Data<AppState>,
        query_filename: &str,
        update_fn: impl FnOnce(&mut QueryData)
    ) -> Result<(), String> {
        let project_service = ProjectService::new();
        let project_dir = self.get_project_dir(app_state);

        // Try to load existing query data or create new
        let (mut query_data, filename) = match self.load_query_data_by_filename(app_state, query_filename) {
            Ok(Some(qd)) => {
                (qd, query_filename.to_string())
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