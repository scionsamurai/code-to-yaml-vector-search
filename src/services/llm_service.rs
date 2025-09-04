// src/services/llm_service.rs
use crate::models::{ProjectFile, ChatMessage};
use llm_api_access::structs::general::Message;
use llm_api_access::llm::{Access, LLM};
use std::fs::read_to_string;
use std::path::Path;
use crate::routes::llm::chat_analysis::utils::escape_html;

pub struct LlmService;

impl LlmService {
    pub fn new() -> Self {
        LlmService {}
    }

    // Updated signature: takes provider and specific_model as separate arguments
    pub async fn get_analysis(&self, prompt: &str, provider: &str, specific_model: Option<&str>) -> String {
        // Determine the target model based on provider string
        let target_model = match provider.to_lowercase().as_str() {
            "openai" => LLM::OpenAI,
            "anthropic" => LLM::Anthropic,
            "gemini" | _ => LLM::Gemini,
        };

        // Pass specific_model directly to the llm_api_access crate
        let llm_response = target_model.send_single_message(prompt, specific_model).await;

        match llm_response {
            Ok(content) => {
                let escaped_content = escape_html(content).await;
                escaped_content
            },
            Err(e) => format!("Error during analysis: {}", e),
        }
    }

    // Updated signature: takes provider and specific_model as separate arguments
    pub async fn send_conversation(&self, messages: &[ChatMessage], provider: &str, specific_model: Option<&str>) -> String {
        // Determine the target model based on provider string
        let target_model = match provider.to_lowercase().as_str() {
            "openai" => LLM::OpenAI,
            "anthropic" => LLM::Anthropic,
            "gemini" | _ => LLM::Gemini,
        };
        
        // Convert your ChatMessage format to the LLM API's Message format
        let api_messages: Vec<Message> = messages
            .iter()
            .map(|msg| Message {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect();

        // Pass specific_model directly to the llm_api_access crate
        let llm_response = target_model.send_convo_message(api_messages, specific_model).await;

        match llm_response {
            Ok(content) => {
                let escaped_content = escape_html(content).await;
                escaped_content
            },
            Err(e) => format!("Error during conversation: {}", e),
        }
    }

    // Updated signature: takes provider and specific_model as separate arguments
    pub async fn convert_to_yaml(&self, file: &ProjectFile, provider: &str, specific_model: Option<&str>) -> String {
        // Determine the target model based on provider string
        let target_model = match provider.to_lowercase().as_str() {
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
                content: format!("&grave;&grave;&grave;\n{}\n&grave;&grave;&grave;", file.content),
            },
        ];

        // Pass specific_model directly to the llm_api_access crate
        let llm_response = target_model.send_convo_message(messages, specific_model).await;

        // remove backticks and extract the YAML content
        let yaml_content = match llm_response {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let mut start_index: Option<usize> = None;
                let mut end_index: Option<usize> = None;
                let mut delimiter_line_indices: Vec<usize> = Vec::new();

                // Find all delimiter lines
                for (i, line) in lines.iter().enumerate() {
                    let trimmed_line = line.trim();
                    if trimmed_line == "&grave;&grave;&grave;" || trimmed_line == "&grave;&grave;&grave;yaml" || trimmed_line == "&grave;&grave;&grave;yml" {
                        delimiter_line_indices.push(i);
                    }
                }

                // Determine start and end based on number of delimiters found
                if delimiter_line_indices.len() >= 2 {
                    // Case 1: Two or more delimiters found (assume block)
                    start_index = Some(delimiter_line_indices[0]);
                    // Find the last delimiter as the end (in case there are multiple blocks or extra &grave;&grave;&grave;)
                    end_index = Some(delimiter_line_indices[delimiter_line_indices.len() - 1]);
                }

                let final_content_lines: Vec<&str> = if let (Some(s_idx), Some(e_idx)) = (start_index, end_index) {
                    // Block found: content between the first and last delimiters
                    if s_idx == e_idx {
                        // This case should not be hit if `delimiter_line_indices.len() >= 2`
                        // is properly handled, but as a safeguard.
                        lines.iter().enumerate()
                            .filter(|&(i, _)| i != s_idx) // Remove the single delimiter line
                            .map(|(_, line)| *line) // Dereference here
                            .collect()
                    } else {
                        lines[s_idx + 1..e_idx].to_vec()
                    }
                } else if delimiter_line_indices.len() == 1 {
                    // Exactly one delimiter found, remove that line and keep the rest
                    let single_delimiter_idx = delimiter_line_indices[0];
                    lines.iter().enumerate()
                         .filter(|&(i, _)| i != single_delimiter_idx) // Keep all lines except the delimiter
                         .map(|(_, line)| *line) // Dereference here
                         .collect()
                } else {
                    // No delimiters found, assume the entire content is the YAML
                    lines.to_vec()
                };

                let mut cleaned_content = final_content_lines.join("\n");

                // If the first line of the extracted content is "yml" or "yaml", remove it.
                if let Some(first_line) = cleaned_content.lines().next() {
                    let trimmed_first_line = first_line.trim();
                    if trimmed_first_line == "yml" || trimmed_first_line == "yaml" {
                        cleaned_content = cleaned_content.lines().skip(1).collect::<Vec<&str>>().join("\n");
                    }
                }

                let trimmed_final = cleaned_content.trim().to_string();
                let escaped_content = escape_html(trimmed_final).await;
                escaped_content
            }
            Err(e) => format!("Error during conversion: {}", e),
        };

        yaml_content
    }
}