// src/services/project_service/query_management.rs

use crate::models::QueryData;
use std::fs::{create_dir_all, read_to_string, write, read_dir, DirEntry};
use std::path::{Path, PathBuf};
use uuid::Uuid;
use std::time::SystemTime;
// Add Uuid import
use crate::models::ChatMessage; // We'll need ChatMessage for migration
use std::collections::HashMap; // We'll need HashMap for migration

pub struct QueryManager;

impl QueryManager {
    pub fn new() -> Self {
        Self {}
    }
    // Helper function to generate a unique filename for QueryData
    pub fn generate_query_filename(&self) -> String {
        format!("{}.json", Uuid::new_v4())
    }

    // Helper function to get the queries directory path
    pub fn get_queries_dir(&self, project_dir: &Path) -> PathBuf {
        project_dir.join("queries")
    }

    // Function to save a QueryData to a file
    pub fn save_query_data(&self, project_dir: &Path, query_data: &QueryData, filename: &str) -> Result<(), String> {
        let queries_dir = self.get_queries_dir(project_dir);

        // Create the queries directory if it doesn't exist
        create_dir_all(&queries_dir)
            .map_err(|e| format!("Failed to create queries directory: {}", e))?;
        let query_file_path = queries_dir.join(filename);
        let query_json = serde_json::to_string_pretty(query_data)
            .map_err(|e| format!("Failed to serialize query data: {}", e))?;

        write(&query_file_path, query_json)
            .map_err(|e| format!("Failed to write query data to file: {}", e))
    }

    // Function to load a QueryData from a file
    pub fn load_query_data(&self, project_dir: &Path, filename: &str) -> Result<QueryData, String> {
        let queries_dir = self.get_queries_dir(project_dir);
        let query_file_path = queries_dir.join(filename);

        let query_json = read_to_string(&query_file_path)
            .map_err(|e| format!("Failed to read query data from file: {}", e))?;

        serde_json::from_str::<QueryData>(&query_json)
            .map_err(|e| format!("Failed to deserialize query data: {}", e))
    }

    
    pub fn get_most_recent_query_file(&self, project_dir: &Path) -> Result<Option<PathBuf>, String> {
        let queries_dir = self.get_queries_dir(project_dir);

        if !queries_dir.exists() {
            return Ok(None); // No queries directory, so no recent file
        }

        let mut files: Vec<DirEntry> = read_dir(queries_dir)
            .map_err(|e| format!("Failed to read queries directory: {}", e))?
            .filter_map(Result::ok) // Ignore any errors reading directory entries
            .collect();

        if files.is_empty() {
            return Ok(None); // No files in the directory
        }

        files.sort_by(|a, b| {
            // Sort by last modified time (newest first)
            let a_time = a.metadata().map(|m| m.modified()).unwrap_or(Ok(SystemTime::UNIX_EPOCH));
            let b_time = b.metadata().map(|m| m.modified()).unwrap_or(Ok(SystemTime::UNIX_EPOCH));

            //Handle possible error return value from metadata()
            match (a_time, b_time) {
                (Ok(a_time), Ok(b_time)) => b_time.cmp(&a_time), // Newest first
                (Err(_), Ok(_)) => std::cmp::Ordering::Less,    // a has an error, so b is better
                (Ok(_), Err(_)) => std::cmp::Ordering::Greater,   // b has an error, so a is better
                (Err(_), Err(_)) => std::cmp::Ordering::Equal,    // both have errors, so they are equal
            }
        });

        // Get the path of the most recent file
        let most_recent_file = files.first().map(|entry| entry.path());
        Ok(most_recent_file)
    }


    // Will handle logic from Project::load_query_data_by_filename
    pub fn load_query_data_by_filename(
        &self,
        project_dir: &Path,
        filename: &str,
    ) -> Result<Option<QueryData>, String> {
        if filename.is_empty() {
            return self.load_most_recent_query_data(project_dir);
        }
        match self.load_query_data(project_dir, filename) {
            Ok(query_data) => Ok(Some(query_data)),
            Err(e) => {
                println!("filename: {}", filename);
                eprintln!("Error loading query data: {}", e);
                // Fallback to most recent query data
                self.load_most_recent_query_data(project_dir) 
            }
        }
    }

    // Will handle logic from Project::load_most_recent_query_data
    pub fn load_most_recent_query_data(
        &self,
        project_dir: &Path,
    ) -> Result<Option<QueryData>, String> {
        match self.get_most_recent_query_file(project_dir) {
            Ok(most_recent_file) => {
                match most_recent_file {
                    Some(file_path) => {
                        let file_name =
                            file_path.file_name().unwrap().to_str().unwrap().to_string();
                        match self.load_query_data(project_dir, &file_name) {
                            Ok(query_data) => Ok(Some(query_data)),
                            Err(e) => {
                                eprintln!("Error loading query data: {}", e);
                                Ok(None)
                            }
                        }
                    }
                    None => Ok(None),
                }
            }
            Err(e) => {
                eprintln!("Error getting most recent query file: {}", e);
                Err(e) // Propagate the error
            }
        }
    }

    // Will handle logic from Project::update_query_data
    pub fn update_query_data_in_project<F>(
        &self,
        project_dir: &Path,
        query_filename: &str,
        update_fn: F,
    ) -> Result<(), String>
    where
        F: FnOnce(&mut QueryData),
    {
        println!("Updating query data for file: {}", query_filename);
        // Try to load existing query data or create new
        let (mut query_data, filename) = match self.load_query_data_by_filename(project_dir, query_filename) {
            Ok(Some(qd)) => {
                (qd, query_filename.to_string())
            }
            _ => {
                // Create new query data and filename
                (
                    QueryData::default(),
                    self.generate_query_filename(),
                )
            }
        };

        // --- MIGRATION LOGIC START ---
        // If chat_nodes is empty but analysis_chat_history is not,
        // it means we've loaded an old file. Migrate it to the new structure.
        if query_data.chat_nodes.is_empty() && !query_data.analysis_chat_history.is_empty() {
            println!("Migrating old chat history format for query: {}", filename);
            let mut new_chat_nodes: HashMap<Uuid, ChatMessage> = HashMap::new();
            let mut last_message_id: Option<Uuid> = None;

            for mut old_message in query_data.analysis_chat_history.drain(..) { // Drain to empty the old Vec
                let new_id = Uuid::new_v4();
                old_message.id = new_id;
                old_message.parent_id = last_message_id;
                
                new_chat_nodes.insert(new_id, old_message);
                last_message_id = Some(new_id);
            }
            query_data.chat_nodes = new_chat_nodes;
            query_data.current_node_id = last_message_id;
            // The analysis_chat_history Vec is now empty and will not be serialized due to skip_serializing_if
        }
        // --- MIGRATION LOGIC END ---

        // Apply the update function to the potentially migrated data
        update_fn(&mut query_data);

        // Save the updated query data
        self.save_query_data(project_dir, &query_data, &filename)
    }

    // Will handle logic from Project::update_query_title
    pub fn update_query_title(
        &self,
        project_dir: &Path,
        query_filename: &str,
        new_title: &str,
    ) -> Result<(), String> {
        self.update_query_data_in_project(project_dir, query_filename, |query_data| {
            query_data.title = Some(new_title.to_string());
        })
    }

    // Will handle logic from Project::get_query_filenames
    pub fn get_query_filenames(
        &self,
        project_dir: &Path,
    ) -> Result<Vec<(String, String)>, String> { // (filename, display_title)
        let queries_dir = self.get_queries_dir(project_dir);

        if !queries_dir.exists() {
            return Ok(Vec::new());
        }

        let mut files: Vec<PathBuf> = read_dir(queries_dir)
            .map_err(|e| format!("Failed to read queries directory: {}", e))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| {
                path.is_file() && path.extension().map_or(false, |ext| ext == "json")
            })
            .collect();

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
            let timestamp = filename.trim_end_matches(".json").to_string();
    
            match self.load_query_data(project_dir, &filename) {
                Ok(query_data) => {
                    let display_title = query_data.title.unwrap_or(timestamp.clone());
                    filenames.push((filename, display_title));
                }
                _ => {
                    filenames.push((filename.clone(), filename.clone()));
                }
            }
        }
    
        Ok(filenames)
    }

    // Will handle logic from Project::get_recent_query_id
    pub fn get_recent_query_id(
        &self,
        project_dir: &Path,
    ) -> Option<String> {
        match self.get_query_filenames(project_dir) {
            Ok(filenames) => {
                if filenames.is_empty() {
                    None
                } else {
                    let most_recent = filenames.last().unwrap();
                    Some(most_recent.0.clone()) // Return the actual filename
                }
            },
            Err(e) => {
                eprintln!("Failed to get recent query ID: {}", e);
                None
            }
        }
    }

    // Will handle logic from Project::get_query_data_field
    pub fn get_query_data_field(
        &self,
        project_dir: &Path,
        query_filename: &str,
        field: &str,
    ) -> Option<String> {
        match self.load_query_data_by_filename(project_dir, query_filename) {
            Ok(Some(query_data)) => {
                match serde_json::to_value(&query_data) {
                    Ok(value) => match value.get(field) {
                        Some(v) if v.is_null() => None,
                        Some(v) if v.is_string() => v.as_str().map(|s| s.to_string()),
                        Some(v) => Some(v.to_string()), // non-string values (arrays, bools, numbers, objects)
                        None => None,
                    },
                    Err(e) => {
                        eprintln!("Failed to convert query data to JSON: {}", e);
                        None
                    }
                }
            }
            Ok(None) | Err(_) => None,
        }
    }

    // Will handle logic from Project::get_query_vec_field
    pub fn get_query_vec_field(
        &self,
        project_dir: &Path,
        query_filename: &str,
        field: &str,
    ) -> Option<Vec<String>> {
        println!("Getting query vec field: {} from file: {}", field, query_filename);
        match self.load_query_data_by_filename(project_dir, query_filename) {
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

    pub fn get_chat_node(
        &self,
        project_dir: &Path,
        query_filename: &str,
        message_id: &Uuid,
    ) -> Option<ChatMessage> {
        match self.load_query_data_by_filename(project_dir, query_filename) {
            Ok(Some(query_data)) => {
                query_data.chat_nodes.get(message_id).cloned()
            }
            _ => None,
        }
    }
}