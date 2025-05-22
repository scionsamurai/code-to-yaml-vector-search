// src/services/file/update_checker.rs
use crate::models::Project;
use std::fs::metadata;
use std::path::Path;

// Check if any files in a project need to be updated
pub fn project_needs_update(project: &Project, output_dir: &str) -> bool {
    use super::reading::read_project_files;

    let files = read_project_files(project);
    let output_path = Path::new(output_dir).join(&project.name);

    files.iter().any(|file| {
        let source_path = &file.path;
        let yaml_path = output_path.join(format!("{}.yml", file.path.replace("/", "*")));
        needs_yaml_update(source_path, yaml_path.to_str().unwrap())
    })
}

// Check if a file needs update based on timestamps
pub fn needs_yaml_update(source_path: &str, yaml_path: &str) -> bool {
    match metadata(yaml_path) {
        Ok(yaml_metadata) => {
            let source_metadata = metadata(source_path).unwrap();
            source_metadata.modified().unwrap() > yaml_metadata.modified().unwrap()
        }
        Err(_) => {
            println!("Path not found {:?}", yaml_path);
            true // YAML file doesn't exist
        }
    }
}