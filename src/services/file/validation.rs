// src/services/file/validation.rs
use crate::models::Project;
use std::path::Path;

pub fn validate_file_paths(project: &Project) -> Vec<(String, bool)> {
    let mut results = Vec::new();

    for (file_path, _) in &project.file_descriptions {
        let is_valid = is_valid_path_comment(project, file_path);
        if !is_valid {
            results.push((file_path.clone(), is_valid));
        }
    }

    results
}

fn is_valid_path_comment(project: &Project, file_path: &str) -> bool {
    use super::reading::read_specific_file;

    if let Some(content) = read_specific_file(project, file_path) {
        let source_path = Path::new(&project.source_dir);
        let file_path_path = Path::new(file_path);

        if let Ok(rel_path) = file_path_path.strip_prefix(source_path) {
            let relative_path_str = rel_path.display().to_string();
            let mut lines = content.lines();
            if let Some(first_line) = lines.next() {
                // Check if the first line is a comment containing the file path
                let expected_comment1 = format!("// {}", file_path);
                let expected_comment2 = format!("// {}", relative_path_str);
                return first_line == expected_comment1 || first_line == expected_comment2;
            }
        } else {
            // If stripping the prefix fails, fall back to comparing against the full path
            let mut lines = content.lines();
            if let Some(first_line) = lines.next() {
                let expected_comment = format!("// {}", file_path);
                return first_line == expected_comment;
            }
        }
    }
    false
}