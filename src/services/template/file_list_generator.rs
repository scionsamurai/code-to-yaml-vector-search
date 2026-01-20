// src/services/template/file_list_generator.rs
use crate::models::Project;
use super::TemplateService;
use crate::services::file::reading::read_exclude_search_files;
use std::collections::HashSet;

impl TemplateService {

    pub fn get_other_files_list_raw(&self, project: &Project, exclude_files: &[String]) -> Vec<String> {
        let mut all_files: Vec<String> = project.embeddings.keys().cloned().collect();

        let excluded_search_files: Vec<String> = read_exclude_search_files(project).into_iter().map(|pf| pf.path).collect();
        all_files.extend(excluded_search_files);

        let exclude_files_set: HashSet<String> = exclude_files.iter().cloned().collect();

        all_files.into_iter()
            .filter(|file| !exclude_files_set.contains(file))
            .collect()
    }

}