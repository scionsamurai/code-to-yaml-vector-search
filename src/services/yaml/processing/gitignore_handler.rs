// src/services/yaml/processing/gitignore_handler.rs
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn read_ignore_file(source_dir: &Path, filename: &str) -> Vec<String> {
    let mut ignore_paths = Vec::new();
    if let Ok(ignore_file) = File::open(source_dir.join(filename)) {
        for line in BufReader::new(ignore_file).lines() {
            if let Ok(path) = line {
                let normalized_path = path.trim().trim_start_matches('/').to_string();
                if !normalized_path.is_empty() {
                    ignore_paths.push(normalized_path);
                }
            }
        }
    }
    ignore_paths
}
pub fn is_file_ignored(source_dir: &str, source_path: &str, original_source_path: &Path) -> bool {
    let source_dir_path = Path::new(source_dir);

    // Read .assistantignore and .gitignore files
    let assistantignore_paths = read_ignore_file(source_dir_path, ".assistantignore");
    let gitignore_paths = read_ignore_file(source_dir_path, ".gitignore");
    let exclude_paths = read_ignore_file(source_dir_path, ".assistantexcludesearch");

    // Get the relative path from the project source directory
    let relative_path = if let Ok(rel_path) = original_source_path.strip_prefix(source_dir_path) {
        rel_path.to_string_lossy().to_string()
    } else {
        source_path.to_string()
    };

    // Helper function to check if a file is ignored based on the ignore patterns
    let is_ignored = |ignore_patterns: &Vec<String>| {
        ignore_patterns.iter().any(|ignore_pattern| {
            if ignore_pattern.ends_with("/*") {
                let dir_pattern = ignore_pattern.trim_end_matches("/*");
                relative_path.starts_with(dir_pattern)
            } else if ignore_pattern.ends_with('/') {
                let dir_pattern = ignore_pattern.trim_end_matches('/');
                relative_path.starts_with(dir_pattern)
            } else {
                relative_path == *ignore_pattern
                    || relative_path.starts_with(&format!("{}/", ignore_pattern))
            }
        })
    };

    // Check .assistantignore first, then .gitignore
    if is_ignored(&assistantignore_paths) {
        return true;
    }

    if is_ignored(&exclude_paths) {
        return true;
    }

    is_ignored(&gitignore_paths)
}