// src/services/file_service.rs
use crate::models::{Project, ProjectFile};
use std::fs::{metadata, read_dir, read_to_string, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct FileService;

impl FileService {
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