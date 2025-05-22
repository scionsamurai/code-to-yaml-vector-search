// src/services/file/extract_imports/rust.rs

pub fn rust_imports(file_content: &str) -> (Vec<String>, String) {
    let mut imports = Vec::new();
    let mut cleaned_content = String::new();
    let mut lines_to_skip: Vec<usize> = Vec::new();
    
    for (line_num, line) in file_content.lines().enumerate() {
        let trimmed = line.trim();
        
        // Check if this line contains a use statement
        if trimmed.starts_with("use ") || trimmed.starts_with("pub use ") {
            lines_to_skip.push(line_num);
            
            // Remove "pub use" or "use" from the beginning
            let import_path = trimmed
                .trim_start_matches("pub use ")
                .trim_start_matches("use ")
                .trim_end_matches(";")
                .trim();
                
            // Handle brace imports
            if import_path.contains("{") {
                let base_path = import_path.split("{").next().unwrap().trim();
                let base = base_path.trim_end_matches("::");
                
                // Extract items inside braces
                if let Some(items_str) = import_path.split("{").nth(1) {
                    let items = items_str.trim_end_matches("}").split(",");
                    
                    for item in items {
                        let item = item.trim();
                        if item.is_empty() {
                            continue;
                        }
                        
                        // Handle "as" aliases by taking the item before "as"
                        let actual_item = if item.contains(" as ") {
                            item.split(" as ").next().unwrap().trim()
                        } else {
                            item
                        };
                        
                        // Form the full import path
                        let full_path = if base.is_empty() {
                            actual_item.to_string()
                        } else if actual_item.starts_with("::") {
                            format!("{}{}", base, actual_item)
                        } else {
                            format!("{}::{}", base, actual_item)
                        };
                        
                        imports.push(full_path.trim_start_matches("::").to_string());
                    }
                }
            } else {
                // Simple import without braces
                imports.push(import_path.trim_start_matches("::").to_string());
            }
        }
    }
    
    // Generate cleaned content
    for (line_num, line) in file_content.lines().enumerate() {
        if !lines_to_skip.contains(&line_num) {
            cleaned_content.push_str(line);
            cleaned_content.push('\n');
        }
    }
    
    (imports, cleaned_content)
}