// src/services/project_service.rs
use crate::models::Project;
use std::fs::{read_dir, read_to_string, write};
use std::path::Path;

pub struct ProjectService;



impl ProjectService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_project(&self, output_dir: &Path) -> Result<Project, String> {
        let project_settings_path = output_dir.join("project_settings.json");
        let project_settings_json = read_to_string(&project_settings_path)
            .map_err(|e| format!("Failed to read project settings: {}", e))?;
        
        serde_json::from_str::<Project>(&project_settings_json)
            .map_err(|e| format!("Failed to parse project settings: {}", e))
    }

    pub fn save_project(&self, project: &Project, output_dir: &Path) -> Result<(), String> {
        let project_settings_path = output_dir.join("project_settings.json");
        let project_settings_json = serde_json::to_string_pretty(project)
            .map_err(|e| format!("Failed to serialize project: {}", e))?;
        
        write(&project_settings_path, project_settings_json)
            .map_err(|e| format!("Failed to write project settings: {}", e))
    }


    pub fn get_yaml_files_html(&self, output_dir: &Path, project_name: &str) -> Result<String, String> {
        let mut file_descriptions: Vec<(String, String)> = Vec::new(); // (full_path, relative_path, description)
    
        let yaml_html = read_dir(output_dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?
            .map(|entry| {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let yaml_path = entry.path();
    
                if yaml_path.file_name().unwrap().to_string_lossy() == "project_settings.json" {
                    return Ok(String::new());
                }
    
                let content = read_to_string(&yaml_path)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
    
                let description = parse_description(&content)
                    .unwrap_or_else(|| "No description.".to_string())
                    .replace(|c: char| c == '\n' || c == '\r', " ")
                    .trim()
                    .to_string();
                let relative_path = yaml_path.strip_prefix(output_dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace(".yml", "")
                    .replace("*", "/");
    
                file_descriptions.push((relative_path.clone(), description.clone()));
    
                Ok(format!(
                    "<div class=\"page\"><p>---</p><h3>path: {}</h3><pre>{}</pre><button onclick=\"regenerate('{}', '{}')\">Regenerate</button></div>",
                    relative_path,
                    content.replace("---\n", "").replace("```", ""),
                    project_name,
                    yaml_path.display()
                ))
            })
            .collect::<Result<Vec<_>, String>>()?
            .join("");
    
        // Sort and find common prefix
        file_descriptions.sort_by_key(|(path, _)| path.clone());
        let common_prefix = file_descriptions.iter()
            .map(|(path, _)| path.clone())
            .reduce(|a, b| common_path_prefix(&a, &b))
            .unwrap_or_default();
    
        // Build indented list
        let mut indented_lines = vec![];
        for (full_path, description) in &file_descriptions {
            let trimmed = full_path.strip_prefix(&common_prefix).unwrap_or(full_path);
            indented_lines.push(format!(
                "{} // {}",
                trimmed, description
            ));
        }
    
        let indented_html = format!(
            r#"<div id="graphDiv"><input type="checkbox" id="fileGraph">
    <label for="fileGraph" style="cursor: pointer; font-weight: bold;">Show File Graph</label>
    <style>
        #graphDiv {{
            display: flex;
            flex-direction: column;
            align-items: center;
        }}
        #fileGraph ~ pre {{
            display: none;
        }}
        #fileGraph:checked ~ pre {{
            display: block;
            background: white;
            overflow: scroll;
            width: 80%;
            padding: 1rem;
        }}
    </style>
    <pre>{}</pre></div>"#,
            indented_lines.join("\n")
        );
    
        Ok(format!("{}{}", indented_html, yaml_html))
    }
    
}

fn parse_description(content: &str) -> Option<String> {
    let mut lines = content.lines();
    // 1) Must start with '---'
    if lines.next()? != "---" { return None; }

    let mut in_block = false;
    let mut desc = String::new();

    for line in lines {
        let trimmed = line.trim_start();

        // 2) if we hit the end of front-matter, stop
        if trimmed == "---" {
            break;
        }

        if !in_block {
            // 3) look for the `description:` key at top-level
            if let Some(rest) = trimmed.strip_prefix("description:") {
                let rest = rest.trim();
                match rest.chars().next() {
                    // block scalar start
                    Some('|') | Some('>') => {
                        in_block = true;
                        continue;
                    }
                    // inline scalar on the same line
                    _ if !rest.is_empty() => {
                        // strip optional quotes
                        let s = rest.trim_matches('"').to_string();
                        return Some(s);
                    }
                    // exactly `description:` with no value → treat as block
                    _ => {
                        in_block = true;
                        continue;
                    }
                }
            }
        } else {
            // 4) we're inside a block — collect indented lines
            // YAML spec: block-scalar content must be indented at least one space
            if line.starts_with(' ') || line.starts_with('\t') {
                // drop only the leading indent
                desc.push_str(line.trim_start());
                desc.push('\n');
            } else {
                // non-indented → end of block
                break;
            }
        }
    }

    if desc.is_empty() {
        None
    } else {
        // trim the final newline
        Some(desc.trim_end().to_string())
    }
}

fn common_path_prefix(a: &str, b: &str) -> String {
    a.split('/')
        .zip(b.split('/'))
        .take_while(|(x, y)| x == y)
        .map(|(x, _)| x)
        .collect::<Vec<_>>()
        .join("/")
}
