// src/services/llm_service.rs
use crate::models::{ProjectFile, ChatMessage};
use llm_api_access::structs::general::Message;
use llm_api_access::llm::{Access, LLM};
use std::fs::read_to_string;
use std::path::Path;
use crate::services::utils::html_utils::escape_html;
use std::io::Error; // Import std::io::Error
use llm_api_access::config::LlmConfig;

#[derive(Debug, Clone, Default)]
pub struct LlmServiceConfig {
    pub temperature: Option<f64>,
    pub thinking_budget: Option<i32>,
    pub grounding_with_search: Option<bool>,
}

impl LlmServiceConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_thinking_budget(mut self, thinking_budget: i32) -> Self {
        self.thinking_budget = Some(thinking_budget);
        self
    }

    pub fn with_grounding_with_search(mut self, grounding_with_search: bool) -> Self {
        self.grounding_with_search = Some(grounding_with_search);
        self
    }

    //Converts to the llm_api_access LlmConfig type.
    pub fn to_llm_config(&self) -> Option<LlmConfig> {
        let mut config = LlmConfig::new();

        if let Some(temperature) = self.temperature {
            config = config.with_temperature(temperature);
        }

        if let Some(thinking_budget) = self.thinking_budget {
            config = config.with_thinking_budget(thinking_budget);
        }

        if let Some(grounding_with_search) = self.grounding_with_search {
            config = config.with_grounding_with_search(grounding_with_search);
        }

        if config.temperature.is_some() || config.thinking_budget.is_some() || config.grounding_with_search.is_some() {
            Some(config)
        } else {
            None
        }
    }
}


pub struct LlmService;

impl LlmService {
    pub fn new() -> Self {
        LlmService {}
    }

    // Updated signature to accept LlmServiceConfig
    pub async fn get_analysis(&self, prompt: &str, provider: &str, specific_model: Option<&str>, config: Option<LlmServiceConfig>) -> String {
        // Determine the target model based on provider string
        let target_model = match provider.to_lowercase().as_str() {
            "openai" => LLM::OpenAI,
            "anthropic" => LLM::Anthropic,
            "gemini" | _ => LLM::Gemini,
        };

        // Convert LlmServiceConfig to LlmConfig
        let llm_config = config.and_then(|c| c.to_llm_config());

        // Pass specific_model and LlmConfig to the llm_api_access crate
        let llm_response = target_model.send_single_message(prompt, specific_model, llm_config.as_ref()).await;

        match llm_response {
            Ok(content) => {
                let escaped_content = escape_html(content).await;
                escaped_content
            },
            Err(e) => format!("Error during analysis: {}", e),
        }
    }

    // Updated signature to accept LlmServiceConfig
    pub async fn send_conversation(&self, messages: &[ChatMessage], provider: &str, specific_model: Option<&str>, config: Option<LlmServiceConfig>) -> String {
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

        // Convert LlmServiceConfig to LlmConfig
        let llm_config = config.and_then(|c| c.to_llm_config());


        // Pass specific_model and LlmConfig to the llm_api_access crate
        let llm_response = target_model.send_convo_message(api_messages, specific_model, llm_config.as_ref()).await;

        match llm_response {
            Ok(content) => {
                let escaped_content = escape_html(content).await;
                escaped_content
            },
            Err(e) => format!("Error during conversation: {}", e),
        }
    }

    // Updated signature to accept LlmServiceConfig
    pub async fn get_optimized_prompt(
        &self,
        original_prompt: &str,
        optimization_direction: Option<&str>,
        chat_history_str: Option<&str>,
        file_context_str: Option<&str>,
        provider: &str,
        specific_model: Option<&str>,
        config: Option<LlmServiceConfig>, // New config parameter
    ) -> Result<String, Error> {
        // Determine the target model based on provider string
        let target_model = match provider.to_lowercase().as_str() {
            "openai" => LLM::OpenAI,
            "anthropic" => LLM::Anthropic,
            "gemini" | _ => LLM::Gemini,
        };

        let mut user_prompt_content = String::new();
        user_prompt_content.push_str("You are an expert query optimization assistant. Your task is to refine user queries. .\n\n");

        if let Some(history) = chat_history_str {
            if !history.is_empty() {
                user_prompt_content.push_str("Here is the relevant chat conversation history:\n");
                user_prompt_content.push_str(history);
                user_prompt_content.push_str("\n\n");
            }
        }

        if let Some(file_context) = file_context_str {
            if !file_context.is_empty() {
                user_prompt_content.push_str("Here are the contents of the selected context files:\n");
                user_prompt_content.push_str(file_context);
                user_prompt_content.push_str("\n\n");
            }
        }

        user_prompt_content.push_str(&format!("Original prompt to optimize: \"{}\"", original_prompt));
        if let Some(direction) = optimization_direction {
            if !direction.is_empty() {
                user_prompt_content.push_str(&format!("\nOptimization Direction: \"{}\"", direction));
            }
        }
        user_prompt_content.push_str("\nYou must only output the optimized query, without any additional conversational text or formatting (e.g., no \"Optimized Query:\", no quotes around the query, no markdown code blocks and no description or summary about the optimization).");
        
        let messages = vec![
            Message {
                role: "user".to_string(),
                content: user_prompt_content,
            },
        ];


        // Convert LlmServiceConfig to LlmConfig
        let llm_config = config.and_then(|c| c.to_llm_config());

        let llm_response = target_model.send_convo_message(messages, specific_model, llm_config.as_ref()).await;

        match llm_response {
            Ok(content) => {
                let escaped_content = escape_html(content).await;
                Ok(escaped_content)
            },
            Err(e) => Err(Error::new(std::io::ErrorKind::Other, format!("LLM API error: {}", e))),
        }
    }


    // Updated signature to accept LlmServiceConfig
    pub async fn convert_to_yaml(&self, file: &ProjectFile, provider: &str, chat_model: Option<&str>, yaml_model: Option<&str>, config: Option<LlmServiceConfig>) -> String { // NEW: Added yaml_model and config
        // Use yaml_model if provided, otherwise fallback to chat_model, then to None (default LLM model for API access if none specified)
        let model_to_use = yaml_model.or(chat_model);

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
                content: format!("```\n{}\n```", file.content),
            },
        ];

        // Convert LlmServiceConfig to LlmConfig
        let llm_config = config.and_then(|c| c.to_llm_config());

        // Pass the determined model and LlmConfig directly to the llm_api_access crate
        let llm_response = target_model.send_convo_message(messages, model_to_use, llm_config.as_ref()).await;

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
                    if trimmed_line == "```" || trimmed_line == "```yaml" || trimmed_line == "```yml" {
                        delimiter_line_indices.push(i);
                    }
                }

                // Determine start and end based on number of delimiters found
                if delimiter_line_indices.len() >= 2 {
                    // Case 1: Two or more delimiters found (assume block)
                    start_index = Some(delimiter_line_indices[0]);
                    // Find the last delimiter as the end (in case there are multiple blocks or extra ```)
                    end_index = Some(delimiter_line_indices[delimiter_line_indices.len() - 1]);
                }

                let final_content_lines: Vec<&str> = if let (Some(s_idx), Some(e_idx)) = (start_index, end_index) {
                    // Block found: content between the first and last delimiters
                    lines[s_idx + 1..e_idx].to_vec()
                } else {
                    lines.to_vec()
                };

                let mut cleaned_content = final_content_lines.join("\n");

                // remove any remaining ```yaml or ```yml or ``` or yml or yaml lines
                cleaned_content = cleaned_content
                    .lines()
                    .filter(|line| {
                        let trimmed_line = line.trim();
                        trimmed_line != "```" && trimmed_line != "```yaml" && trimmed_line != "```yml" && trimmed_line != "yaml" && trimmed_line != "yml"
                    })
                    .collect::<Vec<&str>>()
                    .join("\n");

                let trimmed_final = cleaned_content.trim().to_string();
                let escaped_content = escape_html(trimmed_final).await;
                escaped_content
            }
            Err(e) => format!("Error during conversion: {}", e),
        };

        yaml_content
    }
}