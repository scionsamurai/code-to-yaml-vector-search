use crate::models::{ProjectFile, Project};
use crate::services::file_service::FileService;
use crate::services::llm_service::LlmService;
use std::fs::write;
use std::path::Path;

pub struct YamlService {
    file_service: FileService,
    llm_service: LlmService,
}

impl YamlService {
    pub fn new() -> Self {
        Self {
            file_service: FileService {},
            llm_service: LlmService {},
        }
    }

    // Save YAML files for a project
    pub async fn save_yaml_files(&self, project: &Project, output_dir: &str) {
        println!("Using model: {:?}", &project.model);
        println!("Using: {:?}", &project);
        let output_path = Path::new(output_dir).join(&project.name);
        std::fs::create_dir_all(&output_path).unwrap();

        let files = self.file_service.read_project_files(project);

        for file in files {
            println!("Checking if yaml update needed for {}", &file.path);
            let source_path = &file.path;
            let yaml_path = output_path.join(format!("{}.yml", file.path.replace("/", "*")));
            
            if self.file_service.needs_yaml_update(source_path, yaml_path.to_str().unwrap()) {
                println!("Generating yaml for {}", &file.path);
                let yaml_content = self.llm_service.convert_to_yaml(&file, &project.model).await;
                write(yaml_path, yaml_content.as_bytes()).unwrap();
            }
        }
    }
}