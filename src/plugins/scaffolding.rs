// src/plugins/scaffolding.rs

pub fn create_scaffolding_plugin() -> ChatPlugin {
    ChatPlugin {
        id: "project_scaffolding".to_string(),
        name: "Project Scaffolding Assistant".to_string(),
        description: "Guide you through the initial stages of project planning".to_string(),
        steps: vec![
            // Step 1: Metadata Collection
            PluginStep {
                id: "metadata".to_string(),
                prompt_template: include_str!("../../context/metadata.md").to_string(),
                ui_type: UIComponentType::Form,
                required_inputs: vec!["PROJECT_NAME".to_string(), "PROJECT_DESCRIPTION".to_string()],
                optional_inputs: vec![],
                system_message: Some("You are a helpful project planning assistant. Your job is to collect basic information about the project the user wants to build.".to_string()),
            },
            // Step 2: Project Definition
            PluginStep {
                id: "project_definition".to_string(),
                prompt_template: include_str!("../../context/project_definition.md").to_string(),
                ui_type: UIComponentType::Form,
                required_inputs: vec!["TARGET_AUDIENCE".to_string(), "FEATURES".to_string(), 
                                    "TECH_STACK_BACKEND".to_string(), "TECH_STACK_FRONTEND".to_string()],
                optional_inputs: vec!["CONSTRAINTS".to_string()],
                system_message: None,
            },
            // Additional steps for other phases...
        ],
        state: HashMap::new(),
    }
}