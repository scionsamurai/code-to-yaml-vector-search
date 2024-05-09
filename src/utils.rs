// src/utils.rs
use crate::models::{Project, ProjectFile};
use std::fs::{metadata, read_dir, read_to_string, File, write};
use std::io::{BufRead, BufReader};
use std::path::Path;


use llm_api_access::{ Access, LLM };
use llm_api_access::structs::Message;

pub fn read_files(project: &Project, gitignore_paths: &mut Vec<String>) -> Vec<ProjectFile> {
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
            files.extend(read_files(
                &Project {
                    name: project.name.clone(),
                    languages: project.languages.clone(),
                    source_dir: path_str.clone(),
                    model: project.model.clone(),
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

fn _read_file(file_path: &str) -> Result<ProjectFile, std::io::Error> {
    let path = Path::new(file_path);
    let content = read_to_string(&path)?;
    let metadata = metadata(&path).unwrap();
    let last_modified = metadata.modified().unwrap().elapsed().unwrap().as_secs();

    Ok(ProjectFile {
        path: file_path.to_owned(),
        content,
        last_modified,
    })
}

pub async fn convert_to_yaml(file: &ProjectFile, llm: &str) -> String {
    let target_model: LLM;
    let api_role: &str;

    match llm.as_ref() {
        "gemini" => {
            target_model = LLM::Gemini;
            api_role = "model";
        },
        "openai" => {
            target_model = LLM::OpenAI;
            api_role = "system";
        },
        "anthropic" => {
            target_model = LLM::Anthropic;
            api_role = "assistant";
        },
        _ => {
            target_model = LLM::Gemini;
            api_role = "model";
        },
    }

    let current_dir = std::env::current_dir().unwrap();
    let user_prompt_path = current_dir.join("src/prompts/user.txt");
    let model_prompt_path = current_dir.join("src/prompts/model.txt");

    let user_prompt = match read_to_string(user_prompt_path) {
        Ok(content) => content,
        Err(error) => {
            println!("Error reading user prompt file: {}", error);
            String::new()
        }
    };

    let model_prompt = match read_to_string(model_prompt_path) {
        Ok(content) => content,
        Err(error) => {
            println!("Error reading model prompt file: {}", error);
            String::new()
        }
    };

    let messages = vec![
        Message {
            role: "user".to_string(),
            content: user_prompt.clone(),
        },
        Message {
            role: api_role.to_string(),
            content: model_prompt.clone(),
        },
        Message {
            role: "user".to_string(),
            content: format!("```\n{:?}\n``` This is the code i would like converted to yaml. YOU ARE A FUNCTION, YOU JUST PRINT THE CODE. JUST GIVE ME THE YAML REPRESENTATION OF THE CODE WITHOUT ANY OF THE ACTUAL SOURCE CODE!", file.content),
        },
    ];

    let llm_rspns = target_model.send_convo_message(messages).await;

    // let llm_rspns = target_model.send_convo_message(messages).await;
    let mut yaml_content = llm_rspns.unwrap();

    // Remove the triple backticks and "yaml" prefix if present
    if yaml_content.starts_with("```yaml\n") {
        yaml_content = yaml_content.trim_start_matches("```yaml\n").trim_end_matches("\n```").to_string();
    }

    yaml_content
}

pub async fn save_yaml_files(project: &Project, app_state: &crate::models::AppState) {
    println!("Model {:?}", &project.model);
    let output_dir = Path::new(&app_state.output_dir).join(&project.name);
    std::fs::create_dir_all(&output_dir).unwrap();

    let mut gitignore_paths = vec![];
    let files = read_files(project, &mut gitignore_paths);

    for file in files {
        println!("Generating yaml for {} into output directory", &file.path);
        let source_path = Path::new(&file.path);
        let yaml_path = output_dir.join(format!("{}.yml", file.path.replace("/", "*")));

        let should_convert = match metadata(&yaml_path) {
            Ok(yaml_metadata) => {
                let source_metadata = metadata(&source_path).unwrap();
                source_metadata.modified().unwrap()
                    > yaml_metadata.modified().unwrap()
            }
            Err(_) => true, // YAML file doesn't exist, convert
        };

        if should_convert {
            let yaml_content = convert_to_yaml(&file, &project.model).await;
            write(yaml_path, yaml_content.as_bytes()).unwrap();
        }
    }
}
