// src/services/file/reading.rs
use crate::models::{Project, ProjectFile};
use std::fs::{read_dir, read_to_string, metadata};
use std::path::Path; // Added PathBuf
use crate::services::yaml::processing::gitignore_handler::{read_ignore_file, is_file_ignored}; // Import the function

pub fn read_project_files(project: &Project) -> Vec<ProjectFile> {
    let project_root = Path::new(&project.source_dir); // Define project root here
    // Pass the project_root (constant for all recursion) and the initial project config.
    read_files(project, project_root)
}

// Read files recursively from directory
// `current_project_config` is the Project struct reflecting the *current* directory being scanned (its `source_dir` field).
// `project_root` is the *absolute path to the project's main source directory*, which remains constant throughout recursion.
pub fn read_files(current_project_config: &Project, project_root: &Path) -> Vec<ProjectFile> {
    let mut files = Vec::new();
    let current_scan_dir = Path::new(&current_project_config.source_dir);

    // Split the languages string into a vector of extensions
    let allowed_extensions: Vec<&str> = current_project_config
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

    for entry in read_dir(current_scan_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path(); // This is the absolute path to the current file/dir being examined

        // FIRST check if the path (file or directory) should be ignored based on project_root
        if is_file_ignored(project_root, &path) {
            // println!("Skipping ignored path: {:?}", path); // Optional: add logging
            continue; // Skip this file/directory and its contents if it's ignored
        }

        if path.is_dir() {
            // Recursively read files from subdirectories
            files.extend(read_files(
                &Project {
                    source_dir: path.to_string_lossy().to_string(), // Update source_dir for the recursive call
                    // Ensure other project settings (like languages) are passed down, as they are project-wide.
                    // This creates a *new* Project struct for the recursion, but its core properties like languages, provider, etc., remain the same.
                    ..current_project_config.clone()
                },
                project_root, // Always pass the original project_root
            ));
        } else {
            // This is a file, and it was NOT ignored by the `is_file_ignored` check above.
            // Now, check its extension.
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if allowed_extensions.iter().any(|&ext| ext == extension) {
                match read_to_string(&path) {
                    Ok(content) => {
                        let metadata = metadata(&path).unwrap();
                        let last_modified = metadata.modified().unwrap().elapsed().unwrap().as_secs();

                        files.push(ProjectFile {
                            path: path.to_string_lossy().to_string(),
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

// read_exclude_search_files: This function is specifically for *listing* files that are
// designated as "excluded from search" via .assistantexcludesearch. It should not use
// the general `is_file_ignored` as its purpose is to identify these files, not skip them.
// The primary fix targets `read_files` to ensure these files are *not* included in the
// regular project file list.
// However, the recursive logic for directories within `recursively_read_excluded_files` is flawed.
// It should only recurse into directories if they themselves are matching an exclude pattern
// or are necessary to find excluded files.

// Revised `read_exclude_search_files` for clarity and correctness in recursion for exclude patterns
pub fn read_exclude_search_files(project: &Project) -> Vec<ProjectFile> {
    let mut files = Vec::new();
    let project_root = Path::new(&project.source_dir);

    // Read the exclude patterns from the project root
    let exclude_patterns = read_ignore_file(project_root, ".assistantexcludesearch");

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

    // Helper function to check if a given path matches any of the exclude patterns
    // This logic is similar to `is_file_ignored` but only for `.assistantexcludesearch` and identifies what *should be included* in the exclude list.
    let is_explicitly_excluded = |path: &Path| -> bool {
        let canonical_project_root = project_root.canonicalize().unwrap_or_else(|_| project_root.to_path_buf());
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        let relative_path = match canonical_path.strip_prefix(&canonical_project_root) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => return false, // Path is outside project root
        };

        exclude_patterns.iter().any(|pattern| {
            if pattern.ends_with("/*") {
                let dir_pattern = pattern.trim_end_matches("/*");
                relative_path.starts_with(dir_pattern)
            } else if pattern.ends_with('/') {
                let dir_pattern = pattern.trim_end_matches('/');
                relative_path.starts_with(dir_pattern)
            } else {
                relative_path == *pattern || relative_path.starts_with(&format!("{}/", pattern))
            }
        })
    };


    fn recursively_read_files_for_exclusion_list(
        current_scan_dir: &Path,
        project_root: &Path,
        is_explicitly_excluded: &impl Fn(&Path) -> bool, // Pass the closure
        allowed_extensions: &Vec<&str>,
        files: &mut Vec<ProjectFile>,
    ) {
        for entry in read_dir(current_scan_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path(); // Absolute path

            if path.is_dir() {
                // We need to recurse into directories, but only if they themselves
                // are explicitly excluded, or if their *contents* might be.
                // A simpler approach: always recurse, and let the file check handle the exclusion.
                // This means the `is_explicitly_excluded` on directories is mainly for performance.
                // For correctness, we often need to recurse unless the directory is *definitely* not relevant.
                recursively_read_files_for_exclusion_list(
                    &path,
                    project_root,
                    is_explicitly_excluded,
                    allowed_extensions,
                    files,
                );
            } else {
                // If the file itself is explicitly excluded AND has an allowed extension
                let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                if allowed_extensions.iter().any(|&ext| ext == extension) && is_explicitly_excluded(&path) {
                    match read_to_string(&path) {
                        Ok(content) => {
                            let metadata = metadata(&path).unwrap();
                            let last_modified = metadata.modified().unwrap().elapsed().unwrap().as_secs();

                            files.push(ProjectFile {
                                path: path.to_string_lossy().to_string(),
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

    recursively_read_files_for_exclusion_list(
        project_root,
        project_root, // The initial scan starts from project_root
        &is_explicitly_excluded,
        &allowed_extensions,
        &mut files,
    );

    files
}