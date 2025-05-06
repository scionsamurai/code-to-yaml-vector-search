// src/services/yaml/processing/html_generator.rs
use super::description_parser;
use std::path::Path;

pub fn generate_html(
    yaml_path: &Path,
    source_path: &str,
    content: String,
    project_name: &str,
    file_descriptions: &mut Vec<(String, String)>,
) -> String {
    // Extract description
    let description = description_parser::parse_description(&content)
        .unwrap_or_else(|| "No description.".to_string())
        .replace(|c: char| c == '\n' || c == '\r', " ")
        .trim()
        .to_string();

    // Store file description
    file_descriptions.push((source_path.to_string(), description.clone()));

    // Count lines in content
    let line_count = get_source_file_line_count(source_path);

    // Add split button if file is large (more than 200 lines)
    let split_button = if line_count > 200 {
        format!(
            "<button onclick=\"suggestSplit('{}', '{}')\">Suggest Split</button>",
            project_name, source_path
        )
    } else {
        String::new()
    };

    // Return HTML for this file
    format!(
            "<div class=\"page\"><p>---</p><h3 data-lines=\"{}\">path: {}</h3><pre>{}</pre><button onclick=\"regenerate('{}', '{}')\">Regenerate</button>{}</div>",
            line_count,
            source_path,
            content.replace("---\n", "").replace("```", ""),
            project_name,
            yaml_path.display(),
            split_button
        )
}

fn get_source_file_line_count(source_path: &str) -> usize {
    match std::fs::read_to_string(source_path) {
        Ok(content) => content.lines().count(),
        Err(_) => 0, // Return 0 if we can't read the file
    }
}
