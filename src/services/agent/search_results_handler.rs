// src/services/agent/search_results_handler.rs

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::models::Project;
use crate::services::file::FileService;
use crate::services::yaml::YamlService;
use crate::services::path_utils::PathUtils;
use crate::services::agent::file_context;


/// Consolidates search results from various sources into a single set of file paths.
pub fn consolidate_search_results(
    project: &Project,
    vector_search_results: &Vec<crate::models::SearchResult>, // Use the full path to SearchResult
    bm25f_results: &Vec<(String, f32)>,
    suggested_vector_files: &HashSet<String>,
    thoughts: &mut Vec<String>,
) -> HashSet<String> {
    let mut all_relevant_file_paths: HashSet<String> = HashSet::new();

    // Add paths from suggested_vector_files (most critical)
    for file_path in suggested_vector_files {
        if let Some(normalized_path) = PathUtils::normalize_project_path(file_path, project) {
            all_relevant_file_paths.insert(normalized_path);
        } else {
            thoughts.push(format!("Could not normalize suggested path from vector search: {}", file_path));
        }
    }

    // Add paths from vector_search_results (the raw list from vector DB)
    for search_result in vector_search_results {
        if let Some(normalized_path) = PathUtils::normalize_project_path(&search_result.file_path, project) {
            all_relevant_file_paths.insert(normalized_path);
        } else {
            thoughts.push(format!("Could not normalize path from raw vector search results: {}", search_result.file_path));
        }
    }

    // Add paths from bm25f_results
    for (file_path, _) in bm25f_results {
        // split on project_name/ and get the second item of the split
        let yaml_target_path = file_path.split(&format!("{}/", project.name))
            .nth(1);
        let final_yaml_target_path = match yaml_target_path {
            Some(p) => {
                p.replace("*", "/").replace(".yml", "") // Reverse the earlier replacement to get original path
            },
            None => {
                thoughts.push(format!("Could not extract target path from YAML hit: {}", file_path));
                continue; // Skip this hit
            },
        };
        all_relevant_file_paths.insert(final_yaml_target_path.clone());
    }

    all_relevant_file_paths
}


/// Populates file_contents_map (Full Source) and initial_proactive_yaml_summaries (YAML descriptions).
pub async fn populate_file_context(
    project: &Project,
    all_relevant_file_paths: &HashSet<String>,
    suggested_vector_files: &HashSet<String>,
    project_dir: &Path,
    thoughts: &mut Vec<String>,
) -> (HashMap<String, String>, HashMap<String, String>) {
    let file_service = FileService {};
    let yaml_service = YamlService::new();
    let mut file_contents_map: HashMap<String, String> = HashMap::new();
    let mut initial_proactive_yaml_summaries: HashMap<String, String> = HashMap::new();

    let convert_path_to_yaml_location = |path: &str| {
        project_dir.join(format!("{}.yml", path.replace("/", "*")))
    };

    // Priority 1: Fetch full source for suggested_vector_files
    for file_path in all_relevant_file_paths {
        if suggested_vector_files.contains(file_path) { // Prioritize files suggested by LLM analysis for full source
            if let Some(content) = file_service.read_specific_file(project, file_path) {
                file_contents_map.insert(file_path.clone(), content);
                thoughts.push(format!("Added full source (from LLM vector suggestion): {}", file_path));
            } else {
                thoughts.push(format!("Failed to read suggested file for full source: {}. Trying YAML description.", file_path));
                // If source cannot be read, fall back to YAML description
                let yaml_loc = convert_path_to_yaml_location(file_path);
                if let Ok(yaml_data) = yaml_service.management.get_parsed_yaml_for_file_sync(project, &yaml_loc.to_string_lossy(), project_dir) {
                    initial_proactive_yaml_summaries.insert(file_path.clone(), yaml_data.description);
                    thoughts.push(format!("Added YAML description (suggested source failed): {}", file_path));
                } else {
                    thoughts.push(format!("Failed to get YAML description for {}: {}", file_path, yaml_loc.display()));
                }
            }
        }
    }

    // Priority 2: Get YAML descriptions for other relevant files (if not already full source)
    for file_path in all_relevant_file_paths {
        if !file_contents_map.contains_key(file_path) { // Only process if we don't have full source yet
            let yaml_loc = convert_path_to_yaml_location(file_path);
            if let Ok(yaml_data) = yaml_service.management.get_parsed_yaml_for_file_sync(project, &yaml_loc.to_string_lossy(), project_dir) {
                initial_proactive_yaml_summaries.insert(file_path.clone(), yaml_data.description);
            } else {
                thoughts.push(format!("Failed to get YAML description for {}: {}", file_path, yaml_loc.display()));
            }
        }
    }

    (file_contents_map, initial_proactive_yaml_summaries)
}