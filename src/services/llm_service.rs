// src/services/llm_service.rs
use crate::models::ProjectFile;
use llm_api_access::{Access, LLM};
use llm_api_access::structs::Message;
use std::fs::read_to_string;
use std::path::Path;

pub struct LlmService;
// src/services/llm_service.rs

impl LlmService {
    pub fn new() -> Self {
        LlmService {}
    }
    
    pub async fn get_analysis(&self, prompt: &str, llm: &str) -> String {
        // Determine the target model based on llm string
        let target_model = match llm.to_lowercase().as_str() {
            "openai" => LLM::OpenAI,
            "anthropic" => LLM::Anthropic,
            "gemini" | _ => LLM::Gemini,
        };

        // Send single message to the LLM
        let llm_response = target_model.send_single_message(prompt).await;

        match llm_response {
            Ok(content) => content,
            Err(e) => format!("Error during analysis: {}", e)
        }
    }
    
    pub async fn convert_to_yaml(&self, file: &ProjectFile, llm: &str) -> String {
        // Determine the target model based on llm string
        let target_model = match llm.to_lowercase().as_str() {
            "openai" => LLM::OpenAI,
            "anthropic" => LLM::Anthropic,
            "gemini" | _ => LLM::Gemini,
        };

        // Read the prompt files
        let user_prompt_path = Path::new("src/prompts/user.txt");
        let model_prompt_path = Path::new("src/prompts/model.txt");
        
        let user_prompt = read_to_string(user_prompt_path).unwrap_or_else(|_| String::new());
        let model_prompt = read_to_string(model_prompt_path).unwrap_or_else(|_| String::new());
        
        // Construct the messages
        let messages = vec![
            Message {
                role: "user".to_string(),
                content: user_prompt,
            },
            Message {
                role: "model".to_string(),
                content: model_prompt,
            },
            Message {
                role: "user".to_string(),
                content: format!("```\n{}\n```", file.content),
            },
        ];
        
        // Send the conversation to the LLM
        let llm_response = target_model.send_convo_message(messages).await;
        
        let yaml_content = match llm_response {
            Ok(content) => {
                // Clean up the response
                let mut cleaned = content;
                if cleaned.starts_with("```yaml") {
                    cleaned = cleaned.replacen("```yaml", "", 1);
                } else if cleaned.starts_with("```") {
                    cleaned = cleaned.replacen("```", "", 1);
                }
                
                if cleaned.ends_with("```") {
                    cleaned = cleaned.replacen("```", "", 1);
                }
                
                cleaned.trim().to_string()
            },
            Err(e) => format!("Error during conversion: {}", e),
        };
        
        yaml_content
    }
}