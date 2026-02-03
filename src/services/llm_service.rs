// src/services/llm_service.rs
use crate::models::{ProjectFile, ChatMessage};
use llm_api_access::structs::general::Message;
use llm_api_access::llm::{Access, LLM};
use std::fs::read_to_string;
use std::path::Path;
use crate::services::utils::html_utils::escape_html; // Keep for other functions, but remove from convert_to_yaml's core logic
use std::io::Error;
use llm_api_access::config::LlmConfig;
use serde_yaml; // For YAML deserialization
use crate::services::yaml::FileYamlData; // Import the FileYamlData struct

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

    // Converts to the llm_api_access LlmConfig type.
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

    /// Reads the user prompt template, extracts Rust struct definitions from `src/services/yaml/mod.rs`,
    /// and injects them into the template, replacing "ReplaceWithStructCode".
    async fn get_prompt_with_structs(&self, user_prompt_template_path: &Path, struct_source_path: &Path) -> Result<String, std::io::Error> {
        let user_prompt_template = std::fs::read_to_string(user_prompt_template_path)?;
        let struct_source_content = std::fs::read_to_string(struct_source_path)?;

        let mut extracted_structs = String::new();
        let target_struct_names = [
            "FileYamlData",
            "Function",
            "Parameter",
            "Class",
            "DataStructure",
        ];

        let mut lines = struct_source_content.lines().peekable();

        while let Some(line) = lines.next() {
            let trimmed_line = line.trim();

            if trimmed_line.starts_with("#[derive(") {
                let mut temp_struct_buffer = String::new();
                temp_struct_buffer.push_str(line);
                temp_struct_buffer.push('\n');

                let mut brace_count = 0;
                let mut found_target_struct = false;
                let mut added_pub_struct_line = false;

                while let Some(peeked_line) = lines.peek() {
                    let trimmed_peeked_line = peeked_line.trim();
                    if trimmed_peeked_line.starts_with("#[") {
                        temp_struct_buffer.push_str(lines.next().unwrap());
                        temp_struct_buffer.push('\n');
                    } else if trimmed_peeked_line.starts_with("pub struct") { // Replaced starts_backwards_compat to starts_with
                        if target_struct_names.iter().any(|name| trimmed_peeked_line.contains(name)) {
                            found_target_struct = true;
                        }
                        temp_struct_buffer.push_str(lines.next().unwrap());
                        temp_struct_buffer.push('\n');
                        added_pub_struct_line = true;
                        break;
                    } else {
                        break;
                    }
                }

                if found_target_struct && added_pub_struct_line {
                    for c in temp_struct_buffer.chars() {
                        if c == '{' { brace_count += 1; }
                        else if c == '}' { brace_count -= 1; }
                    }

                    while brace_count > 0 {
                        if let Some(struct_body_line) = lines.next() {
                            temp_struct_buffer.push_str(struct_body_line);
                            temp_struct_buffer.push('\n');
                            for c in struct_body_line.chars() {
                                if c == '{' { brace_count += 1; }
                                else if c == '}' { brace_count -= 1; }
                            }
                        } else {
                            eprintln!("Warning: Reached end of file while parsing struct. Partial struct: {}", temp_struct_buffer);
                            break;
                        }
                    }
                    extracted_structs.push_str(&temp_struct_buffer);
                    extracted_structs.push('\n');
                }
            }
        }
        Ok(user_prompt_template.replace("ReplaceWithStructCode", &extracted_structs))
    }

    pub async fn get_analysis(&self, prompt: &str, provider: &str, specific_model: Option<&str>, config: Option<LlmServiceConfig>) -> String {
        let target_model = match provider.to_lowercase().as_str() {
            "openai" => LLM::OpenAI,
            "anthropic" => LLM::Anthropic,
            "gemini" | _ => LLM::Gemini,
        };

        let llm_config = config.and_then(|c| c.to_llm_config());
        let llm_response = target_model.send_single_message(prompt, specific_model, llm_config.as_ref()).await;

        match llm_response {
            Ok(content) => escape_html(content).await,
            Err(e) => format!("Error during analysis: {}", e),
        }
    }

    pub async fn send_conversation(&self, messages: &[ChatMessage], provider: &str, specific_model: Option<&str>, config: Option<LlmServiceConfig>) -> String {
        let target_model = match provider.to_lowercase().as_str() {
            "openai" => LLM::OpenAI,
            "anthropic" => LLM::Anthropic,
            "gemini" | _ => LLM::Gemini,
        };
        
        let api_messages: Vec<Message> = messages
            .iter()
            .map(|msg| Message {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect();

        let llm_config = config.and_then(|c| c.to_llm_config());
        let llm_response = target_model.send_convo_message(api_messages, specific_model, llm_config.as_ref()).await;

        match llm_response {
            Ok(content) => escape_html(content).await,
            Err(e) => format!("Error during conversation: {}", e),
        }
    }

    pub async fn get_optimized_prompt(
        &self,
        original_prompt: &str,
        optimization_direction: Option<&str>,
        chat_history_str: Option<&str>,
        file_context_str: Option<&str>,
        provider: &str,
        specific_model: Option<&str>,
        config: Option<LlmServiceConfig>,
    ) -> Result<String, Error> {
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

        let llm_config = config.and_then(|c| c.to_llm_config());
        let llm_response = target_model.send_convo_message(messages, specific_model, llm_config.as_ref()).await;

        match llm_response {
            Ok(content) => Ok(escape_html(content).await),
            Err(e) => Err(Error::new(std::io::ErrorKind::Other, format!("LLM API error: {}", e))),
        }
    }

    /// Helper function for a single LLM call, YAML extraction, and validation.
    /// Returns `Ok(validated_yaml_string)` or `Err((error_message, raw_extracted_yaml_string_from_llm))`
    async fn _generate_and_validate_yaml(
        &self,
        messages: Vec<Message>,
        provider: &str,
        model_to_use: Option<&str>,
        config: Option<&LlmServiceConfig>,
        file_path: &str,
    ) -> Result<String, (String, String)> {
        let target_model = match provider.to_lowercase().as_str() {
            "openai" => LLM::OpenAI,
            "anthropic" => LLM::Anthropic,
            "gemini" | _ => LLM::Gemini,
        };

        let llm_config = config.and_then(|c| c.to_llm_config());

        let llm_response_content = target_model.send_convo_message(messages, model_to_use, llm_config.as_ref()).await
            .map_err(|e| (format!("Error during LLM conversion for file '{}': {}", file_path, e), String::new()))?;

        // Extract and clean the YAML content
        let lines: Vec<&str> = llm_response_content.lines().collect();
        let mut start_index: Option<usize> = None;
        let mut end_index: Option<usize> = None;
        let mut delimiter_line_indices: Vec<usize> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed_line = line.trim();
            if trimmed_line == "```" || trimmed_line == "```yaml" || trimmed_line == "```yml" {
                delimiter_line_indices.push(i);
            }
        }

        let final_content_lines: Vec<&str> = if delimiter_line_indices.len() >= 2 {
            let s_idx = delimiter_line_indices[0];
            let e_idx = delimiter_line_indices[delimiter_line_indices.len() - 1];
            lines[s_idx + 1..e_idx].to_vec()
        } else {
            lines.to_vec()
        };

        let mut cleaned_content = final_content_lines.join("\n");

        cleaned_content = cleaned_content
            .lines()
            .filter(|line| {
                let trimmed_line = line.trim();
                !trimmed_line.starts_with("```") && trimmed_line != "yaml" && trimmed_line != "yml"
            })
            .collect::<Vec<&str>>()
            .join("\n");

        let trimmed_final = cleaned_content.trim().to_string();

        // Attempt to deserialize the cleaned YAML content into FileYamlData
        match serde_yaml::from_str::<FileYamlData>(&trimmed_final) {
            Ok(_) => {
                // If deserialization is successful, the YAML is valid. Return the raw string.
                Ok(trimmed_final)
            }
            Err(e) => {
                // If deserialization fails, return an error with details and the raw generated YAML
                Err((format!("LLM generated invalid YAML. Parsing error: {}", e), trimmed_final))
            }
        }
    }

    /// Converts a ProjectFile's content into YAML format, with retry mechanism on parsing failure.
    /// It communicates parsing errors back to the LLM to facilitate correction.
    /// Returns raw, unescaped YAML string on success, or an error string if all attempts fail.
    pub async fn convert_to_yaml(&self, file: &ProjectFile, provider: &str, chat_model: Option<&str>, yaml_model: Option<&str>, config: Option<LlmServiceConfig>) -> Result<String, String> {
        let max_attempts: u8 = 3; // Define maximum retry attempts
        let model_to_use = yaml_model.or(chat_model);

        let user_prompt_template_path = Path::new("src/prompts/user.txt");
        let struct_source_path = Path::new("src/services/yaml/mod.rs");
        let model_prompt_path = Path::new("src/prompts/model.txt");

        let final_user_prompt_content = match self.get_prompt_with_structs(user_prompt_template_path, struct_source_path).await {
            Ok(prompt) => prompt,
            Err(e) => {
                return Err(format!("Error preparing user prompt with structs: {}", e));
            }
        };
        let model_initial_response = read_to_string(model_prompt_path).unwrap_or_else(|_| String::new());

        let mut last_failed_yaml: Option<String> = None;
        let mut last_error_message: Option<String> = None;
        let mut final_successful_yaml: Option<String> = None;

        for attempt in 1..=max_attempts {
            println!("Attempt {} to generate YAML for file: {}", attempt, file.path);

            let mut messages = vec![
                Message {
                    role: "user".to_string(),
                    content: final_user_prompt_content.clone(),
                },
                Message {
                    role: "model".to_string(),
                    content: model_initial_response.clone(),
                },
            ];

            if let (Some(failed_yaml), Some(error_msg)) = (&last_failed_yaml, &last_error_message) {
                // Add feedback messages for retry attempts
                messages.push(Message {
                    role: "user".to_string(),
                    content: format!(
                        "The YAML you previously generated for the *same code* below failed to parse with the following error:\n\n```\n{}\n```\n\nHere was the invalid YAML:\n\n```yaml\n{}\n```\n\nPlease correct the YAML to adhere strictly to the schema and guidelines provided, learning from the parsing error provided. Remember the strict formatting rules. Provide ONLY the corrected YAML block.",
                        error_msg,
                        failed_yaml
                    ),
                });
                // After user feedback, it's appropriate for the model to acknowledge before the next task
                messages.push(Message {
                    role: "model".to_string(),
                    content: "Understood. I will use this feedback to correct the YAML generation and provide the corrected YAML block now.".to_string(),
                });
            }

            // Always provide the code content in the last user message
            messages.push(Message {
                role: "user".to_string(),
                content: format!("```\n{}\n```", file.content),
            });

            match self._generate_and_validate_yaml(messages, provider, model_to_use, config.as_ref(), &file.path).await {
                Ok(yaml_content) => {
                    println!("Successfully generated valid YAML on attempt {} for file: {}", attempt, file.path);
                    final_successful_yaml = Some(yaml_content);
                    break; // Success, exit retry loop
                }
                Err((error_msg, raw_extracted_yaml)) => {
                    eprintln!("Attempt {} failed for file '{}'. Parsing error: {}", attempt, file.path, error_msg);
                    last_error_message = Some(error_msg);
                    last_failed_yaml = Some(raw_extracted_yaml); // Store the raw generated YAML for feedback
                }
            }
        }

        if let Some(yaml) = final_successful_yaml {
            Ok(yaml)
        } else {
            Err(format!(
                "Failed to generate valid YAML after {} attempts for file '{}'. Last error: {}",
                max_attempts,
                file.path,
                last_error_message.unwrap_or_else(|| "Unknown parsing error".to_string())
            ))
        }
    }
}