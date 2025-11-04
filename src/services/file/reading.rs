// src/services/file/reading.rs
use crate::models::{Project, ProjectFile};
use std::fs::{read_dir, read_to_string, File, metadata};
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn read_project_files(project: &Project) -> Vec<ProjectFile> {
    let mut gitignore_paths = Vec::new();
    read_files(project, &mut gitignore_paths)
}

// Read files recursively from directory
pub fn read_files(project: &Project, gitignore_paths: &mut Vec<String>) -> Vec<ProjectFile> {
    let mut files = Vec::new();
    let source_dir = Path::new(&project.source_dir);

    // Read .gitignore file
    if let Ok(gitignore_file) = File::open(source_dir.join(".gitignore")) {
        for line in BufReader::new(gitignore_file).lines() {
            if let Ok(path) = line {
                gitignore_paths.push(source_dir.join(&path).to_string_lossy().to_string());
            }
        }
    }

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

        if path.is_dir() && !gitignore_paths.iter().any(|p| path_str.ends_with(p)) {
            // Recursively read files from subdirectories
            files.extend(read_files(
                &Project {
                    source_dir: path_str.clone(),
                    ..project.clone()
                },
                gitignore_paths,
            ));
        } else if !gitignore_paths.iter().any(|p| path_str.ends_with(p)) {
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

pub fn read_project_files_paths_only(project: &Project) -> Vec<String> {
    let mut gitignore_paths = Vec::new();
    read_files_paths_only_recursive(project, &mut gitignore_paths)
}

fn read_files_paths_only_recursive(project: &Project, gitignore_paths: &mut Vec<String>) -> Vec<String> {
    let mut files = Vec::new();
    let source_dir = Path::new(&project.source_dir);

    if let Ok(gitignore_file) = File::open(source_dir.join(".gitignore")) {
        for line in BufReader::new(gitignore_file).lines() {
            if let Ok(path) = line {
                gitignore_paths.push(source_dir.join(&path).to_string_lossy().to_string());
            }
        }
    }

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

    if let Ok(entries) = read_dir(source_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                let path_str = path.to_string_lossy().to_string();

                if path.is_dir() && !gitignore_paths.iter().any(|p| path_str.starts_with(p)) {
                    files.extend(read_files_paths_only_recursive(
                        &Project {
                            name: project.name.clone(),
                            languages: project.languages.clone(),
                            source_dir: path_str.clone(),
                            // Other fields are not relevant for path reading, but cloning for Project struct consistency
                            ..project.clone()
                        },
                        gitignore_paths,
                    ));
                } else if !gitignore_paths.iter().any(|p| path_str.starts_with(p)) {
                    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                    if allowed_extensions.iter().any(|&ext| ext == extension) {
                        files.push(path_str);
                    }
                }
            }
        }
    }
    files
}
