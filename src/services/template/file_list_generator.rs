// src/services/template/file_list_generator.rs
use crate::models::Project;
use super::TemplateService;

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

    pub fn generate_other_files_list(&self, project: &Project, exclude_files: &[String], selected_files: &[String]) -> String {
        // Get all project files
        let all_files: Vec<String> = match &project.embeddings {
            embeddings => embeddings.keys().cloned().collect(),
        };

        // Filter out the files that are already in the relevant files list
        let other_files: Vec<String> = all_files.into_iter()
            .filter(|file| !exclude_files.contains(file))
            .collect();
        
        self.generate_file_list(&other_files, selected_files, project)
    }
}