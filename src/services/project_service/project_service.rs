// src/services/project_service.rs
use crate::models::Project;
use crate::services::yaml::YamlService;
use crate::services::template::TemplateService;
use std::fs::read_to_string; 
use std::path::Path;

pub mod query_management;
pub mod chat_management;

use self::query_management::QueryManager;
use self::chat_management::ChatManager;  

pub struct ProjectService {
    pub query_manager: QueryManager,
    pub chat_manager: ChatManager,  
}

impl ProjectService {
    pub fn new() -> Self {
        Self {
            query_manager: QueryManager::new(),
            chat_manager: ChatManager::new(),  
        }
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
    
}