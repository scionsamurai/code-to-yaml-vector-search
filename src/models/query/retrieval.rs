// src/models/query/retrieval.rs
use crate::models::{AppState, Project};
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

        let mut filenames: Vec<(String, String)> = Vec::new();
        for path in files.iter() {
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            let timestamp = filename.trim_end_matches(".json").to_string(); // Remove .json extension
    
            // Load the query data to get the title
            match self.load_query_data_by_filename(app_state, &filename) {
                Ok(Some(query_data)) => {
                    let display_title = query_data.title.unwrap_or(timestamp.clone());
                    filenames.push((filename, display_title));
                }
                _ => {
                    // If loading fails, use timestamp as default
                    filenames.push((filename.clone(), filename));
                }
            }
        }
    
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
                    Some(most_recent.0.clone())
                }
            },
            Err(e) => {
                eprintln!("Failed to get recent query ID: {}", e);
                None
            }
        }
    }

    pub fn get_query_data_field(
        &self,
        app_state: &web::Data<AppState>,
        query_filename: &str,
        field: &str,
    ) -> Option<String> {
        match self.load_query_data_by_filename(app_state, query_filename) {
            Ok(Some(query_data)) => {
                match field {
                    "query" => Some(query_data.query),
                    "title" => query_data.title,
                    _ => None,
                }
            }
            Ok(None) => None,
            Err(_e) => None,
        }
    }

    pub fn get_query_vec_field(
        &self,
        app_state: &web::Data<AppState>,
        query_filename: &str,
        field: &str,
    ) -> Option<Vec<String>> {
        match self.load_query_data_by_filename(app_state, query_filename) {
            Ok(Some(query_data)) => {
                match field {
                    "vector_results" => Some(
                        query_data.vector_results
                            .into_iter()
                            .map(|(path, _)| path)
                            .collect()
                    ),
                    "context_files" => Some(query_data.context_files),
                    _ => Some(Vec::new()),
                }
            }
            Ok(None) => Some(Vec::new()),
            Err(_e) => Some(Vec::new()),
        }
    }
}