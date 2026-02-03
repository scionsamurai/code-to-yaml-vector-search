// src/services/agent/search_results_handler.rs

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::models::Project;
use crate::services::path_utils::PathUtils;

use crate::services::search_service::SearchResult;
use crate::services::agent::file_context; // Import the file_context module

/// Consolidates search results from various sources into a single set of file paths.
pub fn consolidate_search_results(
    project: &Project,
    vector_search_results: &Vec<SearchResult>, // Use the full path to SearchResult
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
    // let file_service = FileService {}; // No longer needed directly here
    // let yaml_service = YamlService::new(); // No longer needed directly here
    let mut file_contents_map: HashMap<String, String> = HashMap::new();
    let mut initial_proactive_yaml_summaries: HashMap<String, String> = HashMap::new();

    // Prepare lists of paths for each type of loading
    let mut paths_for_full_content: Vec<String> = Vec::new();
    let mut paths_for_yaml_summaries: Vec<String> = Vec::new();

    for file_path in all_relevant_file_paths {
        if suggested_vector_files.contains(file_path) {
            paths_for_full_content.push(file_path.clone());
        } else {
            paths_for_yaml_summaries.push(file_path.clone());
        }
    }

    // Load full file contents using file_context
    thoughts.push(format!("Loading full contents for {} files via file_context.", paths_for_full_content.len()));
    let loaded_full_contents = file_context::load_file_contents(project, &paths_for_full_content, thoughts);
    for (path, content) in loaded_full_contents {
        file_contents_map.insert(path, content);
    }
    thoughts.push(format!("Successfully loaded {} files with full content.", file_contents_map.len()));

    // Load YAML summaries using file_context
    thoughts.push(format!("Loading YAML summaries for {} files via file_context.", paths_for_yaml_summaries.len()));
    let loaded_yaml_summaries = file_context::load_yaml_summaries(project, &paths_for_yaml_summaries, project_dir, thoughts).await;
    for (path, summary) in loaded_yaml_summaries {
        // Only insert if we don't already have the full content for this path
        if !file_contents_map.contains_key(&path) {
            initial_proactive_yaml_summaries.insert(path, summary);
        }
    }
    thoughts.push(format!("Successfully loaded {} YAML summaries.", initial_proactive_yaml_summaries.len()));

    // Edge case: If a suggested_vector_file failed to load full content, try to load its YAML
    // This part is a bit more involved now. We should check which `paths_for_full_content`
    // didn't make it into `file_contents_map` and then try to load their YAMLs.
    let mut failed_full_content_paths: Vec<String> = Vec::new();
    for path in paths_for_full_content {
        if !file_contents_map.contains_key(&path) {
            failed_full_content_paths.push(path);
        }
    }
    if !failed_full_content_paths.is_empty() {
        thoughts.push(format!("Attempting to load YAML descriptions for {} files where full content fetch failed.", failed_full_content_paths.len()));
        let fallback_yaml_summaries = file_context::load_yaml_summaries(project, &failed_full_content_paths, project_dir, thoughts).await;
        for (path, summary) in fallback_yaml_summaries {
            // Ensure we don't accidentally overwrite if a file magically appeared, though unlikely
            if !file_contents_map.contains_key(&path) && !initial_proactive_yaml_summaries.contains_key(&path) {
                initial_proactive_yaml_summaries.insert(path.clone(), summary);
                thoughts.push(format!("Added YAML description (fallback for failed full source): {}", path));
            }
        }
    }


    (file_contents_map, initial_proactive_yaml_summaries)
}