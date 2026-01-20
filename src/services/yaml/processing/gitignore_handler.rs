// src/services/yaml/processing/gitignore_handler.rs
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf}; // Added PathBuf

// This function remains public as it's used by `read_exclude_search_files`
// and possibly other places where a specific directory's ignore file is read.
pub fn read_ignore_file(dir_to_scan_for_ignore_file: &Path, filename: &str) -> Vec<String> {
    let mut ignore_paths = Vec::new();
    if let Ok(ignore_file) = File::open(dir_to_scan_for_ignore_file.join(filename)) {
        for line in BufReader::new(ignore_file).lines() {
            if let Ok(path) = line {
                let normalized_path = path.trim().trim_start_matches('/').to_string();
                if !normalized_path.is_empty() && !normalized_path.starts_with('#') { // Ignore comments
                    ignore_paths.push(normalized_path);
                }
            }
        }
    }
    ignore_paths
}

// Updated is_file_ignored signature: takes project_root and file_path_to_check
// This function will check if `file_path_to_check` should be ignored based on ignore files
// located at `project_root`.
pub fn is_file_ignored(project_root: &Path, file_path_to_check: &Path) -> bool {
    // Canonicalize paths for robust comparison
    let canonical_project_root = project_root.canonicalize().unwrap_or_else(|_| project_root.to_path_buf());
    let canonical_file_path = file_path_to_check.canonicalize().unwrap_or_else(|_| file_path_to_check.to_path_buf());

    // Get the relative path from the project root
    let relative_path = match canonical_file_path.strip_prefix(&canonical_project_root) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => {
            // If the file is not within the project root, it cannot be ignored by project-level ignore rules.
            return false;
        }
    };

    // Read ignore patterns from the project root using the existing `read_ignore_file`
    let assistantignore_patterns = read_ignore_file(&canonical_project_root, ".assistantignore");
    let gitignore_patterns = read_ignore_file(&canonical_project_root, ".gitignore");
    let assistantexcludesearch_patterns = read_ignore_file(&canonical_project_root, ".assistantexcludesearch");

    // Helper function to check if the relative path matches any pattern
    let matches_pattern = |patterns: &[String]| {
        patterns.iter().any(|ignore_pattern| {
            // Handle directory patterns like "foo/*" or "foo/"
            if ignore_pattern.ends_with("/*") {
                let dir_pattern = ignore_pattern.trim_end_matches("/*");
                relative_path.starts_with(dir_pattern)
            } else if ignore_pattern.ends_with('/') {
                let dir_pattern = ignore_pattern.trim_end_matches('/');
                relative_path.starts_with(dir_pattern)
            }
            // Handle specific file or directory patterns
            else {
                relative_path == *ignore_pattern || relative_path.starts_with(&format!("{}/", ignore_pattern))
            }
        })
    };

    // Check patterns in order of precedence: .assistantignore, .assistantexcludesearch, .gitignore
    if matches_pattern(&assistantignore_patterns) {
        return true;
    }
    if matches_pattern(&assistantexcludesearch_patterns) {
        return true;
    }
    // Only apply .gitignore if not already ignored by a higher precedence file.
    if matches_pattern(&gitignore_patterns) {
        return true;
    }

    false
}