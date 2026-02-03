// src/services/agent/file_context.rs

use std::collections::HashMap;
use std::path::Path;
use crate::models::Project;
use crate::services::file::FileService;
use crate::services::yaml::YamlService;
use crate::services::path_utils::PathUtils;

/// Loads file contents into a HashMap.
pub fn load_file_contents(
    project: &Project,
    file_paths: &Vec<String>,
) -> HashMap<String, String> {
    let file_service = FileService {};
    let mut file_contents_map: HashMap<String, String> = HashMap::new();

    for file_path in file_paths {
        if let Some(content) = file_service.read_specific_file(project, file_path) {
            file_contents_map.insert(file_path.clone(), content);
        }
    }

    file_contents_map
}

/// Loads YAML summaries into a HashMap.
pub async fn load_yaml_summaries(
    project: &Project,
    file_paths: &Vec<String>,
    project_dir: &Path
) -> HashMap<String, String> {
    let yaml_service = YamlService::new();
    let mut yaml_summaries: HashMap<String, String> = HashMap::new();

    let convert_path_to_yaml_location = |path: &str| {
        project_dir.join(format!("{}.yml", path.replace("/", "*")))
    };

    for file_path in file_paths {
        let yaml_loc = convert_path_to_yaml_location(file_path);
        if let Ok(yaml_data) = yaml_service.management.get_parsed_yaml_for_file_sync(project, &yaml_loc.to_string_lossy(), project_dir) {
            yaml_summaries.insert(file_path.clone(), yaml_data.description);
        }
    }

    yaml_summaries
}

pub fn normalize_and_deduplicate_paths(
    project: &Project,
    file_paths: &Vec<String>,
    thoughts: &mut Vec<String>,
) -> Vec<String> {
    let mut normalized_paths: Vec<String> = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    for file_path in file_paths {
        if let Some(normalized_path) = PathUtils::normalize_project_path(file_path, project) {
            if !seen_paths.contains(&normalized_path) {
                normalized_paths.push(normalized_path.clone());
                seen_paths.insert(normalized_path);
            }
        } else {
            thoughts.push(format!("Could not normalize path: {}", file_path));
        }
    }

    normalized_paths
}