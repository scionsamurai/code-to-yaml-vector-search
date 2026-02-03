// src/services/agent/file_context.rs

use std::collections::HashMap;
use std::path::Path;
use crate::models::Project;
use crate::services::file::FileService;
use crate::services::yaml::YamlService;

/// Loads file contents into a HashMap.
pub fn load_file_contents(
    project: &Project,
    file_paths: &Vec<String>,
    thoughts: &mut Vec<String>, // ADD THIS
) -> HashMap<String, String> {
    let file_service = FileService {};
    let mut file_contents_map: HashMap<String, String> = HashMap::new();

    for file_path in file_paths {
        if let Some(content) = file_service.read_specific_file(project, file_path) {
            file_contents_map.insert(file_path.clone(), content);
            // thoughts.push(format!("Loaded full source: {}", file_path));
        } else {
            thoughts.push(format!("Failed to read file: {}", file_path));
        }
    }

    file_contents_map
}

/// Loads YAML summaries into a HashMap.
pub async fn load_yaml_summaries(
    project: &Project,
    file_paths: &Vec<String>,
    project_dir: &Path,
    thoughts: &mut Vec<String>, // ADD THIS
) -> HashMap<String, String> {
    let yaml_service = YamlService::new();
    let mut yaml_summaries: HashMap<String, String> = HashMap::new();

    for file_path in file_paths {
        if let Ok(yaml_data) = yaml_service.management.get_parsed_yaml_for_file_sync(project, &file_path, project_dir) {
            yaml_summaries.insert(file_path.clone(), yaml_data.description);
            // thoughts.push(format!("Loaded YAML summary: {}", file_path));
        } else {
            thoughts.push(format!("Failed to load YAML for: {}", file_path,));
        }
    }

    yaml_summaries
}