use crate::models::Project;
use super::TemplateService;

impl TemplateService {
    pub fn generate_file_list(&self, files: &[String], selected_files: &[String]) -> String {
        files.iter()
            .map(|file| {
                format!(
                    r#"<div class="file-item">
                        <input type="checkbox" class="file-checkbox" value="{}" {}> {}
                    </div>"#,
                    file,
                    if selected_files.contains(file) { "checked" } else { "" },
                    file
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
        
        self.generate_file_list(&other_files, selected_files)
    }
}