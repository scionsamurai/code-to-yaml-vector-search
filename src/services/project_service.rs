// src/services/project_service.rs
use crate::models::Project;
use crate::services::yaml::YamlService;
use crate::services::template_service::TemplateService;
use std::fs::read_to_string;
use std::path::Path;

pub struct ProjectService {
    yaml_service: YamlService,
    template_service: TemplateService,
}

impl ProjectService {
    pub fn new() -> Self {
        Self {
            yaml_service: YamlService::new(),
            template_service: TemplateService::new(),
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
        
        // Delegate YAML processing to YamlService
        let (yaml_html, file_descriptions, cleanup_needed, orphaned_files) = 
            self.yaml_service.process_yaml_files(output_dir, project_name, &mut project)?;
        
        // Store file descriptions in the project
        project.file_descriptions = file_descriptions.clone().into_iter().collect();
        
        // Handle orphaned files cleanup
        if !orphaned_files.is_empty() {
            self.yaml_service.clean_up_orphaned_files(project_name, orphaned_files);
        }
        
        // Save project if needed
        if cleanup_needed {
            if let Err(e) = self.save_project(&project, output_dir) {
                eprintln!("Failed to save project after cleaning up: {}", e);
            }
        }
        
        // Generate the file graph HTML using TemplateService
        let graph_html = self.template_service.generate_file_graph_html(&file_descriptions);
        
        Ok(format!("{}{}", graph_html, yaml_html))
    }
}