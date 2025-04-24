// src/services/project_service.rs
use crate::models::Project;
use std::fs::{read_dir, read_to_string, write};
use std::path::Path;

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
        
        write(&project_settings_path, project_settings_json)
            .map_err(|e| format!("Failed to write project settings: {}", e))
    }

    pub fn get_yaml_files_html(&self, output_dir: &Path, project_name: &str) -> Result<String, String> {
        let yaml_files = read_dir(output_dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?
            .map(|entry| {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let yaml_path = entry.path();
                
                if yaml_path.file_name().unwrap().to_string_lossy() == "project_settings.json" {
                    return Ok(String::new());
                }
                
                let content = read_to_string(&yaml_path)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                
                Ok(format!(
                    "<div class=\"page\"><p>---</p><h3>path: {}</h3><pre>{}</pre><button onclick=\"regenerate('{}', '{}')\">Regenerate</button></div>",
                    yaml_path.file_name().unwrap().to_string_lossy().replace("*", "/").replace(".yml", ""),
                    content.replace("---\n", "").replace("```", ""),
                    project_name,
                    yaml_path.display()
                ))
            })
            .collect::<Result<Vec<_>, String>>()?
            .join("");
        
        Ok(yaml_files)
    }
}