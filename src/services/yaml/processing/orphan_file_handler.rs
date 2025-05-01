use std::path::Path;
    use crate::models::Project;

    pub fn handle_orphan_file(
        yaml_path: &Path,
        source_path: &str,
        orphaned_files: &mut Vec<String>,
        project: &mut Project,
        cleanup_needed: &mut bool,
    ) {
        orphaned_files.push(String::from(source_path));

        // Remove the YAML file
        if let Err(e) = std::fs::remove_file(&yaml_path) {
            eprintln!(
                "Failed to remove orphaned YAML file {}: {}",
                yaml_path.display(),
                e
            );
        }

        // Remove from embeddings in project settings
        if project.embeddings.remove(source_path).is_some() {
            *cleanup_needed = true;
        }
    }