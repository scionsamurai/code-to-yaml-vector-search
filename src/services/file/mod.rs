// src/services/file/mod.rs
pub mod extract_imports;
pub mod reading;
pub mod update_checker;
pub mod validation;

use crate::models::{Project, ProjectFile};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

// Define a custom error type for your specific application errors
#[derive(Debug)]
pub enum FileServiceError {
    TraversalAttempt,
    InvalidPath,
    Io(std::io::Error),
}

impl fmt::Display for FileServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileServiceError::TraversalAttempt => write!(f, "Attempted directory traversal"),
            FileServiceError::InvalidPath => write!(f, "Invalid file path"),
            FileServiceError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl Error for FileServiceError {}

impl From<std::io::Error> for FileServiceError {
    fn from(err: std::io::Error) -> Self {
        FileServiceError::Io(err)
    }
}

// Result alias for convenience
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct FileService;

impl FileService {
    pub fn extract_imports(&self, file_content: &str, language: &str) -> (Vec<String>, String) {
        extract_imports::extract_imports(file_content, language)
    }

    pub fn validate_file_paths(&self, project: &Project) -> Vec<(String, bool)> {
        validation::validate_file_paths(project)
    }

    pub fn read_project_files(&self, project: &Project) -> Vec<ProjectFile> {
        reading::read_project_files(project)
    }

    pub fn project_needs_update(&self, project: &Project, output_dir: &str) -> bool {
        update_checker::project_needs_update(project, output_dir)
    }

    pub fn read_specific_file(&self, project: &Project, file_path: &str) -> Option<String> {
        reading::read_specific_file(project, file_path)
    }

    pub fn needs_yaml_update(&self, source_path: &str, yaml_path: &str) -> bool {
        update_checker::needs_yaml_update(source_path, yaml_path)
    }

    pub fn write_file_content(
        &self,
        project: &Project,
        relative_file_path: &str,
        content: &str,
    ) -> Result<()> {
        let source_dir = PathBuf::from(&project.source_dir);

        let target_path = source_dir.join(relative_file_path);

        if !target_path.starts_with(&source_dir) {
            println!("FileService Error: Attempted directory traversal detected. Target path: {:?} is not within source directory: {:?}", target_path, source_dir); // DEBUG
            return Err(Box::new(FileServiceError::TraversalAttempt));
        }

        let parent_dir = target_path.parent().ok_or_else(|| {
            println!(
                "FileService Error: Invalid target path, no parent directory found for {:?}",
                target_path
            ); // DEBUG
            Box::new(FileServiceError::InvalidPath) as Box<dyn Error>
        })?;

        fs::create_dir_all(parent_dir)?;
        fs::write(&target_path, content)?;

        Ok(())
    }
}
