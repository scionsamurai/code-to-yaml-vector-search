// src/services/file_service.rs
use crate::models::{Project, ProjectFile};
use std::fs::{metadata, read_dir, read_to_string, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct FileService;

impl FileService {
    
    pub fn validate_file_paths(&self, project: &Project) -> Vec<(String, bool)> {
        let mut results = Vec::new();
        
        for (file_path, _) in &project.file_descriptions {
            let is_valid = self.is_valid_path_comment(project, file_path);
            if !is_valid {
                results.push((file_path.clone(), is_valid));
            }
        }
        
        results
     }

     fn is_valid_path_comment(&self, project: &Project, file_path: &str) -> bool {
        if let Some(content) = self.read_specific_file(project, file_path) {
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

    // Check if a file needs update based on timestamps
    pub fn needs_yaml_update(&self, source_path: &str, yaml_path: &str) -> bool {
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

    pub fn read_specific_file(&self, project: &Project, file_path: &str) -> Option<String> {
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

    // Read all files from a project directory
    pub fn read_project_files(&self, project: &Project) -> Vec<ProjectFile> {
        let mut gitignore_paths = Vec::new();
        self.read_files(project, &mut gitignore_paths)
    }

    // Read files recursively from directory
    pub fn read_files(&self, project: &Project, gitignore_paths: &mut Vec<String>) -> Vec<ProjectFile> {
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
                files.extend(self.read_files(
                    &Project {
                        name: project.name.clone(),
                        languages: project.languages.clone(),
                        source_dir: path_str.clone(),
                        model: project.model.clone(),
                        saved_queries: project.saved_queries.clone(),
                        embeddings: project.embeddings.clone(),
                        file_descriptions: project.file_descriptions.clone(),
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

    // Check if any files in a project need to be updated
    pub fn project_needs_update(&self, project: &Project, output_dir: &str) -> bool {
        let files = self.read_project_files(project);
        let output_path = Path::new(output_dir).join(&project.name);

        files.iter().any(|file| {
            let source_path = &file.path;
            let yaml_path = output_path.join(format!("{}.yml", file.path.replace("/", "*")));
            self.needs_yaml_update(source_path, yaml_path.to_str().unwrap())
        })
    }
}