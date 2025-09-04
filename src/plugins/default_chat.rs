// src/plugins/default_chat.rs

use crate::models::{ChatPlugin, PluginStep, UIComponentType};
use std::collections::HashMap;

pub fn create_default_chat_plugin() -> ChatPlugin {
    ChatPlugin {
        id: "default_chat".to_string(),
        name: "Standard Code Analysis".to_string(),
        description: "Analyze code with AI assistance".to_string(),
        steps: vec![
            PluginStep {
                id: "standard_chat".to_string(),
                prompt_template: "You are an AI assistant helping with code analysis. The user has provided context files which you can refer to. Please help analyze the code and answer any questions.".to_string(),
                ui_type: UIComponentType::None, // Standard chat interface
                required_inputs: vec![],
                optional_inputs: vec![],
                system_message: None, // Will use the context-based system message
            },
        ],
        state: HashMap::new(),
    }
}