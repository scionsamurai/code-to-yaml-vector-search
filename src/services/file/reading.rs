// src/services/file/reading.rs
use crate::models::{Project, ProjectFile};
use std::fs::{read_dir, read_to_string, metadata};
use std::path::Path;
use crate::services::yaml::processing::gitignore_handler::{read_ignore_file, is_file_ignored}; // Import the function

pub fn read_project_files(project: &Project) -> Vec<ProjectFile> {
    read_files(project)
}

// Read files recursively from directory
pub fn read_files(project: &Project) -> Vec<ProjectFile> {
    let mut files = Vec::new();
    let source_dir = Path::new(&project.source_dir);
    let source_dir_str = project.source_dir.clone(); // Store source_dir for use in is_file_ignored

    // Split the languages string into a vector of extensions
    let allowed_extensions: Vec<&str> = project
        .languages
        .split(',')
        .flat_map(|s| {
            let mut parts = vec![];
            for part in s.split_whitespace() {
                if !part.is_empty() {
                    parts.push(part);
                }
            }
            parts
        })
        .collect();

    for entry in read_dir(source_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();

        if path.is_dir() {
            // Recursively read files from subdirectories
            let original_source_path = Path::new(&path_str);
            if !is_file_ignored(&source_dir_str, &path_str, original_source_path) {
              files.extend(read_files(
                  &Project {
                      source_dir: path_str.clone(),
                      ..project.clone()
                  },
              ));
            }
        } else {
            let original_source_path = Path::new(&path_str);
            if !is_file_ignored(&source_dir_str, &path_str, original_source_path) {
                // Check if the file extension is allowed
                let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                if allowed_extensions.iter().any(|&ext| ext == extension) {
                    match read_to_string(&path) {
                        Ok(content) => {
                            let metadata = metadata(&path).unwrap();
                            let last_modified = metadata.modified().unwrap().elapsed().unwrap().as_secs();

                            files.push(ProjectFile {
                                path: path_str,
                                content,
                                last_modified,
                            });
                        }
                        Err(e) => {
                            println!("Warning: Unable to read file {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    }

    files
}

pub fn read_specific_file(project: &Project, file_path: &str) -> Option<String> {
    // First try direct path from source directory
    let source_path = Path::new(&project.source_dir).join(file_path);
    if let Ok(content) = read_to_string(&source_path) {
        return Some(content);
    }

    // If direct path fails, try alternative approaches
    // For example, the path might be relative in a different way
    let alt_source_path = Path::new(&project.source_dir).join(file_path.trim_start_matches('/'));
    if let Ok(content) = read_to_string(&alt_source_path) {
        return Some(content);
    }

    None
}



pub fn read_exclude_search_files(project: &Project) -> Vec<ProjectFile> {
    let mut files = Vec::new();
    let source_dir = Path::new(&project.source_dir);
    let source_dir_str = project.source_dir.clone();

    let exclude_paths = read_ignore_file(source_dir, ".assistantexcludesearch");
    let allowed_extensions: Vec<&str> = project
        .languages
        .split(',')
        .flat_map(|s| {
            let mut parts = vec![];
            for part in s.split_whitespace() {
                if !part.is_empty() {
                    parts.push(part);
                }
            }
            parts
        })
        .collect();


    fn recursively_read_excluded_files(
        project: &Project,
        source_dir: &Path,
        source_dir_str: &str,
        exclude_paths: &Vec<String>, // Changed to Vec<String>
        allowed_extensions: &Vec<&str>,
        files: &mut Vec<ProjectFile>,
    ) {
        for entry in read_dir(source_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();
            let relative_path = path_str.trim_start_matches(source_dir_str).trim_start_matches('/').to_string();

            let is_excluded = exclude_paths.iter().any(|pattern| {
                if pattern.ends_with("/*") {
                    let dir_pattern = pattern.trim_end_matches("/*");
                    relative_path.starts_with(dir_pattern)
                } else if pattern.ends_with('/') {
                    let dir_pattern = pattern.trim_end_matches('/');
                    relative_path.starts_with(dir_pattern)
                }
                else {
                     relative_path == *pattern || relative_path.starts_with(&format!("{}/", pattern))
                }
            });

            if path.is_dir() {
                if is_excluded {
                    recursively_read_excluded_files(
                        project,
                        &path,
                        source_dir_str,
                        exclude_paths,
                        allowed_extensions,
                        files,
                    );
                } else {
                    recursively_read_excluded_files(
                        project,
                        &path,
                        source_dir_str,
                        exclude_paths,
                        allowed_extensions,
                        files,
                    );
                }
            } else {
                 // Check if the file extension is allowed
                let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                if allowed_extensions.iter().any(|&ext| ext == extension) && is_excluded {
                    match read_to_string(&path) {
                        Ok(content) => {
                            let metadata = metadata(&path).unwrap();
                            let last_modified = metadata.modified().unwrap().elapsed().unwrap().as_secs();

                            files.push(ProjectFile {
                                path: path_str,
                                content,
                                last_modified,
                            });
                        }
                        Err(e) => {
                            println!("Warning: Unable to read file {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    }

    recursively_read_excluded_files(
        project,
        source_dir,
        &source_dir_str,
        &exclude_paths,
        &allowed_extensions,
        &mut files,
    );

    files
}