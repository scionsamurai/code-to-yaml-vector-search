use crate::models::ProjectFile;
use llm_api_access::{Access, LLM};
use llm_api_access::structs::Message;
use std::fs::read_to_string;
use std::path::Path;

pub struct LlmService;

impl LlmService {
    pub async fn convert_to_yaml(&self, file: &ProjectFile, llm: &str) -> String {
        let target_model: LLM;
        let api_role: &str;

        match llm {
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
                content: format!("```\n{:?}\n``` This is the code i would like converted to YAML, do not forget imports from any file. YOU ARE A FUNCTION, YOU JUST PRINT THE CODE. JUST GIVE ME THE YAML REPRESENTATION OF THE CODE WITHOUT ANY OF THE ACTUAL SOURCE CODE!", file.content),
            },
        ];

        let llm_rspns = target_model.send_convo_message(messages).await;
        let mut yaml_content = llm_rspns.unwrap();

        // Remove the triple backticks and "yaml" prefix if present
        if yaml_content.starts_with("```yaml\n") {
            yaml_content = yaml_content.trim_start_matches("```yaml\n").trim_end_matches("\n```").to_string();
        }

        yaml_content
    }
}