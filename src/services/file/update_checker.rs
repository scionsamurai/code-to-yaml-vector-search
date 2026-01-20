// src/services/file/update_checker.rs
use crate::models::Project;
use std::fs::metadata;
use std::path::Path;
use crate::services::git_service::GitService; // Import GitService
use git2::Repository; // Import Repository type
use crate::services::yaml::processing::gitignore_handler::is_file_ignored;

// Check if any files in a project need to be updated
pub fn project_needs_update(project: &Project, output_dir: &str) -> bool {
    use super::reading::read_project_files;

    // It's important that read_project_files itself filters out ignored files.
    let files = read_project_files(project);
    let output_path = Path::new(output_dir).join(&project.name);
    let project_root_path = Path::new(&project.source_dir); // Define project root here

    // Only open the repo once if git integration is enabled
    let repo_result = if project.git_integration_enabled {
        GitService::open_repository(project_root_path)
    } else {
        Err(crate::services::git_service::GitError::Other("Git integration not enabled".to_string()))
    };

    files.iter().any(|file| {
        let source_path = Path::new(&file.path);
        let yaml_path = output_path.join(format!("{}.yml", file.path.replace("/", "*")));
        // needs_yaml_update is now called *only* on files that were *not* ignored by `read_project_files`.
        // So the `is_file_ignored` check inside `needs_yaml_update` becomes a redundancy,
        // but it acts as a safeguard.
        needs_yaml_update(project, &repo_result, source_path, &yaml_path)
    })
}

// Check if a file needs update based on Git blob hash or timestamps
pub fn needs_yaml_update(
    project: &Project,
    repo_result: &Result<Repository, crate::services::git_service::GitError>,
    source_path: &Path,
    yaml_path: &Path,
) -> bool {
    // This check is now largely redundant if `read_project_files` correctly filters ignored files,
    // but it's kept as a robust secondary check and for the logging message.
    if is_file_ignored(&Path::new(&project.source_dir), source_path) {
        println!("File {:?} is ignored, skipping update check.", source_path);
        return false;
    }

    // 1. Check if the YAML file exists first. If not, it definitely needs an update.
    let yaml_file_exists = yaml_path.exists();
    if !yaml_file_exists {
        println!("YAML file not found for source: {:?}", source_path);
        return true; // YAML file doesn't exist, always needs update
    }

    // 2. Determine if Git integration is active and repository is successfully opened
    let use_git_tracking = project.git_integration_enabled && repo_result.is_ok();

    if use_git_tracking {
        let repo = repo_result.as_ref().unwrap(); // We know repo_result is Ok here
        match GitService::get_blob_hash(repo, source_path) {
            Ok(current_blob_hash) => {
                // Check if we have existing embedding metadata for this file
                if let Some(metadata_entry) = project.embeddings.get(source_path.to_str().unwrap_or_default()) {
                    // If a git_blob_hash is stored and matches the current one, no update needed
                    if let Some(stored_blob_hash) = &metadata_entry.git_blob_hash {
                        if stored_blob_hash == &current_blob_hash {
                            return false; // Content hasn't changed according to Git hash
                        } else {
                            println!("Git blob hash mismatch for {:?}. Needs update.", source_path);
                            return true; // Content changed
                        }
                    } else {
                        // Git integration enabled, but no hash stored for this file (bootstrapping).
                        println!("Git integration enabled but no blob hash found for {:?}. Needs update for bootstrapping.", source_path);
                        return true;
                    }
                } else {
                    // No embedding metadata at all for this file (new file or first run with Git)
                    println!("No embedding metadata for {:?}. Needs update.", source_path);
                    return true;
                }
            },
            Err(e) => {
                eprintln!("Failed to get Git blob hash for {:?}. Falling back to timestamp. Error: {}", source_path, e);
                // Fallback to timestamp if we can't get the blob hash for some reason
            }
        }
    }

    // Fallback to timestamp comparison. This path is taken if:
    // - Git integration is not enabled for the project.
    // - Git integration is enabled, but the repository could not be opened.
    // - Git integration is enabled, but getting the blob hash failed.

    let source_metadata = match metadata(source_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to get metadata for source file {:?}: {}", source_path, e);
            return true; // If source file metadata can't be read, assume it needs update
        }
    };

    let yaml_metadata = match metadata(yaml_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to get metadata for YAML file {:?}: {}", yaml_path, e);
            return true;
        }
    };

    // Compare modified times
    source_metadata.modified().unwrap() > yaml_metadata.modified().unwrap()
}