use crate::models::Project;
use std::path::Path;
use std::fs::{read_dir, read_to_string};
use std::result::Result;

pub struct YamlProcessing;

impl YamlProcessing {
    pub fn new() -> Self {
        YamlProcessing {}
    }

    pub fn process_yaml_files(&self, output_dir: &Path, project_name: &str, project: &mut Project) 
        -> Result<(String, Vec<(String, String)>, bool, Vec<String>), String> {
        let mut file_descriptions: Vec<(String, String)> = Vec::new();
        let mut cleanup_needed = false;
        let mut orphaned_files = Vec::new();
        
        let yaml_html = read_dir(output_dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?
            .filter_map(|entry| {
                self.process_yaml_entry(
                    entry, 
                    project, 
                    &mut file_descriptions, 
                    &mut orphaned_files, 
                    &mut cleanup_needed,
                    project_name
                )
            })
            .collect::<Result<Vec<_>, String>>()?
            .join("");
            
        Ok((yaml_html, file_descriptions, cleanup_needed, orphaned_files))
    }

    fn process_yaml_entry(&self, 
                          entry: Result<std::fs::DirEntry, std::io::Error>, 
                          project: &mut Project,
                          file_descriptions: &mut Vec<(String, String)>,
                          orphaned_files: &mut Vec<String>,
                          cleanup_needed: &mut bool,
                          project_name: &str) -> Option<Result<String, String>> {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e)).ok()?;
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
        let source_path = file_name
            .replace(".yml", "")
            .replace("*", "/");
        
        // Check if source file exists
        let original_source_path = Path::new(&project.source_dir).join(&source_path);
        if !original_source_path.exists() {
            // Source file doesn't exist, mark it for cleanup
            orphaned_files.push(source_path.clone());
            
            // Remove the YAML file
            if let Err(e) = std::fs::remove_file(&yaml_path) {
                eprintln!("Failed to remove orphaned YAML file {}: {}", yaml_path.display(), e);
            }
            
            // Remove from embeddings in project settings
            if project.embeddings.remove(&source_path).is_some() {
                *cleanup_needed = true;
            }
            
            return Some(Ok(String::new()));
        }
        
        // Process existing file
        match self.process_yaml_file(&yaml_path, &source_path, file_descriptions, project_name) {
            Ok(html) => Some(Ok(html)),
            Err(e) => Some(Err(e))
        }
    }

    fn process_yaml_file(&self, 
                        yaml_path: &Path, 
                        source_path: &str,
                        file_descriptions: &mut Vec<(String, String)>,
                        project_name: &str) -> Result<String, String> {
        // Read file content
        let content = read_to_string(yaml_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        // Extract description
        let description = self.parse_description(&content)
            .unwrap_or_else(|| "No description.".to_string())
            .replace(|c: char| c == '\n' || c == '\r', " ")
            .trim()
            .to_string();
        
        // Store file description
        file_descriptions.push((source_path.to_string(), description.clone()));
        
        // Return HTML for this file
        Ok(format!(
            "<div class=\"page\"><p>---</p><h3>path: {}</h3><pre>{}</pre><button onclick=\"regenerate('{}', '{}')\">Regenerate</button></div>",
            source_path,
            content.replace("---\n", "").replace("```", ""),
            project_name,
            yaml_path.display()
        ))
    }

    pub fn parse_description(&self, content: &str) -> Option<String> {
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
    
}