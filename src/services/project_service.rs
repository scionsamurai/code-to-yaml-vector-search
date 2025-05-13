// src/services/project_service.rs
use crate::models::{
    Project,
    QueryData
};
use crate::services::yaml::YamlService;
use crate::services::template::TemplateService;
use std::fs::{read_to_string, read_dir, create_dir_all, write, DirEntry};
use std::path::{Path, PathBuf};
use uuid::Uuid;
use std::time::SystemTime;

pub struct ProjectService;

impl ProjectService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_project(&self, output_dir: &Path) -> Result<Project, String> {
        let project_settings_path = output_dir.join("project_settings.json");
        let project_settings_json = read_to_string(&project_settings_path)
            .map_err(|e| format!("Failed to read project settings: {}", e))?;
        
        serde_json::from_str::<Project>(&project_settings_json)
            .map_err(|e| format!("Failed to parse project settings: {}", e))
    }

    pub fn save_project(&self, project: &Project, output_dir: &Path) -> Result<(), String> {
        let project_settings_path = output_dir.join("project_settings.json");
        let project_settings_json = serde_json::to_string_pretty(project)
            .map_err(|e| format!("Failed to serialize project: {}", e))?;
        
        std::fs::write(&project_settings_path, project_settings_json)
            .map_err(|e| format!("Failed to write project settings: {}", e))
    }

    pub fn get_yaml_files_html(&self, output_dir: &Path, project_name: &str) -> Result<String, String> {
        let mut project = self.load_project(output_dir)?;
        let yaml_service = YamlService::new();
        let template_service = TemplateService::new();
        
        // Delegate YAML processing to YamlService
        let (yaml_html, file_descriptions, cleanup_needed, orphaned_files) = 
            yaml_service.process_yaml_files(output_dir, project_name, &mut project)?;
        
        // Store file descriptions in the project
        project.file_descriptions = file_descriptions.clone().into_iter().collect();
        
        // Handle orphaned files cleanup
        if !orphaned_files.is_empty() {
            yaml_service.clean_up_orphaned_files(project_name, orphaned_files);
        }
        
        // Save project if needed
        if cleanup_needed {
            if let Err(e) = self.save_project(&project, output_dir) {
                eprintln!("Failed to save project after cleaning up: {}", e);
            }
        }
        
        // Generate the file graph HTML using TemplateService
        let graph_html = template_service.generate_file_graph_html(project_name, &file_descriptions);
        
        Ok(format!("{}<h2 style='text-align: center;'>YAML Representations</h2>{}", graph_html, yaml_html))
    }
    
       // Helper function to generate a unique filename for QueryData
    pub fn generate_query_filename(&self) -> String {
        format!("{}.json", Uuid::new_v4())
    }

    // Helper function to get the queries directory path
    fn get_queries_dir(&self, project_dir: &Path) -> PathBuf {
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


}