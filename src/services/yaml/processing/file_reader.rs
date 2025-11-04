// src/services/yaml/processing/file_reader.rs
use super::gitignore_handler;
use super::html_generator;
use super::orphan_file_handler;
use crate::models::Project;
use std::fs;
use std::path::Path;

pub fn read_directory(
    output_dir: &Path,
    project: &mut Project,
    file_descriptions: &mut Vec<(String, String)>,
    orphaned_files: &mut Vec<String>,
    cleanup_needed: &mut bool,
    project_name: &str,
) -> Result<String, String> {
    let string_vec = fs::read_dir(output_dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?
        .filter_map(|entry| {
            process_yaml_entry(
                entry,
                project,
                file_descriptions,
                orphaned_files,
                cleanup_needed,
                project_name,
            )
        })
        .collect::<Result<Vec<String>, String>>()?;
    
    Ok(string_vec.join(""))
}

fn process_yaml_entry(
    entry: Result<std::fs::DirEntry, std::io::Error>,
    project: &mut Project,
    file_descriptions: &mut Vec<(String, String)>,
    orphaned_files: &mut Vec<String>,
    cleanup_needed: &mut bool,
    project_name: &str,
) -> Option<Result<String, String>> {
    let entry = entry
        .map_err(|e| format!("Failed to read entry: {}", e))
        .ok()?;
    let yaml_path = entry.path();

    // Skip project_settings.json
    if yaml_path.file_name().unwrap().to_string_lossy() == "project_settings.json" {
        return Some(Ok(String::new()));
    }

    // Check if file is a YAML file
    if yaml_path.extension().and_then(|ext| ext.to_str()) != Some("yml") {
        return Some(Ok(String::new()));
    }

    // Extract the original source file path
    let file_name = yaml_path.file_name()?.to_string_lossy();
    let source_path = file_name.replace(".yml", "").replace("*", "/");

    // Check if source file exists and is not gitignored
    let original_source_path = Path::new(&project.source_dir).join(&source_path);

    if !original_source_path.exists()
        || gitignore_handler::is_file_ignored(
            &project.source_dir,
            &source_path,
            &original_source_path,
        )
    {
        // Source file doesn't exist or is in an ignore file, mark it for cleanup
        orphan_file_handler::handle_orphan_file(
            &yaml_path,
            &source_path,
            orphaned_files,
            project,
            cleanup_needed,
        );
        return Some(Ok(String::new()));
    }

    // Process existing file
    match process_yaml_file(
        &yaml_path,
        &source_path,
        file_descriptions,
        project_name,
        project
    ) {
        Ok(html) => Some(Ok(html)),
        Err(e) => Some(Err(e)),
    }
}

fn process_yaml_file(
    yaml_path: &Path,
    source_path: &str,
    file_descriptions: &mut Vec<(String, String)>,
    project_name: &str,
    project: &Project,
) -> Result<String, String> {
    // Read file content
    let content =
        fs::read_to_string(yaml_path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Generate HTML for this file
    let html = html_generator::generate_html(
        yaml_path,
        source_path,
        content,
        project_name,
        file_descriptions,
        project
    );
    Ok(html)
}