// src/services/yaml/processing/mod.rs
pub mod description_parser;
pub mod file_reader;
pub mod gitignore_handler;
pub mod html_generator;
pub mod orphan_file_handler;

use std::path::Path;
use std::result::Result;

use crate::models::Project;

pub struct YamlProcessing;

impl YamlProcessing {
    pub fn new() -> Self {
        YamlProcessing {}
    }

    pub fn process_yaml_files(
        &self,
        output_dir: &Path,
        project_name: &str,
        project: &mut Project,
    ) -> Result<(String, Vec<(String, String)>, bool, Vec<String>), String> {
        let mut file_descriptions: Vec<(String, String)> = Vec::new();
        let mut cleanup_needed = false;
        let mut orphaned_files = Vec::new();

        let yaml_html = file_reader::read_directory(
            output_dir,
            project,
            &mut file_descriptions,
            &mut orphaned_files,
            &mut cleanup_needed,
            project_name,
        )?;

        Ok((yaml_html, file_descriptions, cleanup_needed, orphaned_files))
    }
}
