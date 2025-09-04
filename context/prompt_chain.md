```yaml
id: recipe_planner
name: Recipe Planner
description: Plan meals and generate shopping lists
initial_step: meal_preferences

steps:
  - id: meal_preferences
    name: Meal Preferences
    step_type:
      type: user_input
      message: "Let's plan your meal!"
      fields:
        - name: meal_type
          field_type: select
          label: What type of meal?
          required: true
          options:
            - Breakfast
            - Lunch
            - Dinner
        - name: servings
          field_type: number
          label: Number of servings
          required: true
          validation:
            type: range
            min: 1.0
            max: 20.0
    next:
      type: static
      target: recipe_generation

  - id: recipe_generation
    name: Generate Recipes
    step_type:
      type: llm_call
      system_prompt: You are a helpful cooking assistant.
      user_prompt: "Suggest 3 {meal_type} recipes for {servings} people. Return as JSON array."
      save_as: recipe_options
    next:
      type: static
      target: recipe_selection

  - id: recipe_selection
    name: Choose Recipe
    step_type:
      type: user_input
      message: "Here are your recipe options: {recipe_options}"
      fields:
        - name: chosen_recipe
          field_type: select
          label: Which recipe would you like?
          required: true
          dynamic_options:
            source: variable
            key: recipe_options
    next:
      type: conditional
      conditions:
        - condition:
            type: variable_exists
            key: chosen_recipe
          target: final_recipe
      default: meal_preferences

  - id: final_recipe
    name: Final Recipe
    step_type:
      type: display
      template: "Your chosen recipe: {chosen_recipe}. Enjoy cooking!"
    next:
      type: end
```

This wont be for a recipe planner, that is just an example of my prompt chaining example. I want to integrate prompt chaining but want to design it first. 

```rust
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct QueryData {
    pub query: String,
    pub vector_results: Vec<(String, f32)>,
    pub context_files: Vec<String>,
    pub analysis_chat_history: Vec<ChatMessage>,
    pub llm_analysis: String,
    pub title: Option<String>,
    // Add prompt chain support
    #[serde(default)]
    pub chain_variables: HashMap<String, Value>, // For chain-scoped variables like "recipe_planner.recipe_options"
    pub current_chain_step: Option<String>, // Current step if in a chain
    pub active_chain_id: Option<String>, // Which chain is currently active
}

// Simplified chain context that works with QueryData
#[derive(Debug, Clone)]
pub struct ChainContext<'a> {
    pub query_data: &'a mut QueryData,
    pub chain_id: String,
}

impl<'a> ChainContext<'a> {
    pub fn new(query_data: &'a mut QueryData) -> Self {
        Self { query_data, chain_id: query_data.active_chain_id }
    }
    
    pub fn set_variable(&mut self, key: &str, value: Value) {
        let scoped_key = format!("{}.{}", self.chain_id, key);
        self.query_data.chain_variables.insert(scoped_key, value);
    }
    
    pub fn get_variable(&self, key: &str) -> Option<&Value> {
        let scoped_key = format!("{}.{}", self.chain_id, key);
        self.query_data.chain_variables.get(&scoped_key)
    }
    
    pub fn resolve_template(&self, template: &str) -> String {
        let mut result = template.to_string();
        
        // Replace chain variables
        for (full_key, value) in &self.query_data.chain_variables {
            if let Some(key) = full_key.strip_prefix(&format!("{}.", self.chain_id)) {
                let placeholder = format!("{{{}}}", key);
                let replacement = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => serde_json::to_string(value).unwrap_or_default(),
                };
                result = result.replace(&placeholder, &replacement);
            }
        }
        result
    }
    
    pub fn set_current_step(&mut self, step_id: String) {
        self.query_data.current_chain_step = Some(step_id);
    }
    
    pub fn get_current_step(&self) -> Option<&String> {
        self.query_data.current_chain_step.as_ref()
    }
    
    pub fn clear_chain(&mut self) {
        // Remove all variables for this chain
        self.query_data.chain_variables.retain(|key, _| {
            !key.starts_with(&format!("{}.", self.chain_id))
        });
        self.query_data.current_chain_step = None;
        self.query_data.active_chain_id = None;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptChain {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub initial_step: String,
    pub steps: Vec<Step>,
}

impl PromptChain {
    pub fn from_yaml_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let chain: PromptChain = serde_yaml::from_str(&content)?;
        Ok(chain)
    }
    
    pub fn from_yaml_str(yaml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let chain: PromptChain = serde_yaml::from_str(yaml)?;
        Ok(chain)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub name: String,
    pub step_type: StepConfig,
    pub next: NextStep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepConfig {
    UserInput {
        message: String,
        fields: Vec<InputField>,
    },
    LlmCall {
        system_prompt: String,
        user_prompt: String,
        save_as: String,
    },
    Display {
        template: String,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NextStep {
    Static { target: String },
    Conditional {
        conditions: Vec<Condition>,
        default: Option<String>,
    },
    End,
}

// Chain executor that works with QueryData
pub struct ChainExecutor {
    pub chain: PromptChain,
}

impl ChainExecutor {
    pub fn new(chain: PromptChain) -> Self {
        Self { chain }
    }
    
    pub fn start_chain(&self, query_data: &mut QueryData) {
        query_data.active_chain_id = Some(self.chain.id.clone());
        query_data.current_chain_step = Some(self.chain.initial_step.clone());
    }
    
    pub fn get_current_step(&self, query_data: &QueryData) -> Option<&Step> {
        let step_id = query_data.current_chain_step.as_ref()?;
        self.chain.steps.iter().find(|step| step.id == *step_id)
    }
    
    pub fn advance_to_step(&self, query_data: &mut QueryData, step_id: String) {
        query_data.current_chain_step = Some(step_id);
    }
    
    pub fn end_chain(&self, query_data: &mut QueryData) {
        let mut context = ChainContext::new(query_data, self.chain.id.clone());
        context.clear_chain();
    }
}
```

Usage Example:
```rust
// Load chain
let chain = PromptChain::from_yaml_file("chains/recipe_planner.yaml")?;
let executor = ChainExecutor::new(chain);

// Start chain
executor.start_chain(&mut query_data);

// Work with variables
let mut context = ChainContext::new(&mut query_data, "recipe_planner".to_string());
context.set_variable("recipe_options", json!(["pasta", "salad", "soup"]));

// Get current step
if let Some(step) = executor.get_current_step(&query_data) {
    // Process step...
}
```
