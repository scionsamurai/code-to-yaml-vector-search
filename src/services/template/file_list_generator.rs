// src/services/template/file_list_generator.rs
use crate::models::Project;
use super::TemplateService;
use crate::services::file::reading::read_exclude_search_files;
use std::collections::HashSet;

impl TemplateService {
    pub fn generate_file_list(&self, files: &[String], selected_files: &[String], project: &Project) -> String {
        files.iter()
            .map(|file| {
                let file_path = std::path::Path::new(file);
                let file_extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                
                let yaml_checkbox = if file_extension != "md" {
                    let use_yaml = project.file_yaml_override.get(file).map(|&b| b).unwrap_or(project.default_use_yaml);
                    format!(
                        r#"<span class="right">
                            <input type="checkbox" class="yaml-checkbox" value="{}" {}> YAML
                        </span>"#,
                        file,
                        if use_yaml { "checked" } else { "" },
                    )
                } else {
                    "".to_string()
                };
                format!(
                    r#"<div class="file-item">
                        <span class="left">
                            <input type="checkbox" class="file-checkbox" value="{}" {}> 
                            <span>{}</span>
                        </span>
                        {}
                    </div>"#,
                    file,
                    if selected_files.contains(file) { "checked" } else { "" },
                    file,
                    yaml_checkbox
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn get_other_files_list_raw(&self, project: &Project, exclude_files: &[String]) -> Vec<String> {
        let mut all_files: Vec<String> = project.embeddings.keys().cloned().collect();

        let excluded_search_files: Vec<String> = read_exclude_search_files(project).into_iter().map(|pf| pf.path).collect();
        all_files.extend(excluded_search_files);

        let exclude_files_set: HashSet<String> = exclude_files.iter().cloned().collect();

        all_files.into_iter()
            .filter(|file| !exclude_files_set.contains(file))
            .collect()
    }

    pub fn generate_llm_suggested_files_list(&self, llm_suggested_files: &[String], context_files: &[String], project: &Project) -> String {
        // Filter out files that don't exist in project embeddings (just in case LLM suggests a non-existent file)
        let existing_llm_suggested_files: Vec<String> = llm_suggested_files.iter()
            .filter(|file| project.embeddings.contains_key(*file))
            .map(|file| file.clone())
            .collect();

        self.generate_file_list(&existing_llm_suggested_files, context_files, project)
    }


    pub fn generate_other_files_list(&self, project: &Project, exclude_files: &[String], selected_files: &[String]) -> String {
                // Get all project files
        let mut all_files: Vec<String> = match &project.embeddings {
            embeddings => embeddings.keys().cloned().collect(),
        };

        // Get all project files that are excluded from search
        let excluded_files: Vec<String> = read_exclude_search_files(project).into_iter().map(|pf| pf.path).collect();
        all_files.extend(excluded_files.clone());

        // Convert exclude_files to a HashSet for faster lookups
        let exclude_files_set: HashSet<String> = exclude_files.iter().cloned().collect();

        // Filter out the files that are already in the combined exclusion list
        let other_files: Vec<String> = all_files.into_iter()
            .filter(|file| !exclude_files_set.contains(file))
            .collect();
        
        self.generate_file_list(&other_files, selected_files, project)
    }

    pub fn generate_relevant_files_list(&self, context_files: &[String], vector_files: &[String], project: &Project) -> String {
        // vector_files are already filtered in the analyze_query handler to exclude LLM suggested files
        let existing_vector_files: Vec<String> = vector_files.iter()
            .filter(|file| project.embeddings.contains_key(*file))
            .map(|file| file.clone())
            .collect();

        self.generate_file_list(&existing_vector_files, context_files, project)
    }
}