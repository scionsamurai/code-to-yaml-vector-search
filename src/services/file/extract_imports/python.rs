// src/services/file/extract_imports/python.rs
use regex::Regex;

pub fn python_imports(file_content: &str) -> (Vec<String>, String) {
    let mut imports = Vec::new();
    let mut cleaned_content = String::new();
    let mut in_multiline_import = false;
    let mut multiline_buffer = String::new();

    // Standard import regex
    let import_regex = Regex::new(r"^\s*import\s+(.+)$").unwrap();
    
    // From import regex
    let from_import_regex = Regex::new(r"^\s*from\s+([.\w_]+)\s+import\s+(.+)$").unwrap();
    
    // Check for multiline continuation
    let continued_line_regex = Regex::new(r".*\\\s*$").unwrap();

    for line in file_content.lines() {
        let trimmed_line = line.trim();
        
        // Skip comments
        if trimmed_line.starts_with("#") {
            cleaned_content.push_str(line);
            cleaned_content.push('\n');
            continue;
        }
        
        // Handle multiline imports
        if in_multiline_import {
            multiline_buffer.push_str(trimmed_line);
            
            if !continued_line_regex.is_match(trimmed_line) {
                // End of multiline import
                in_multiline_import = false;
                
                // Process the complete import statement
                if multiline_buffer.starts_with("import ") {
                    let import_part = multiline_buffer.trim_start_matches("import ").replace("\\", "").trim().to_string();
                    for import_name in import_part.split(',') {
                        imports.push(import_name.trim().to_string());
                    }
                } else if multiline_buffer.contains(" import ") {
                    if let Some(captures) = from_import_regex.captures(&format!("from {}", multiline_buffer)) {
                        if let (Some(module), Some(names)) = (captures.get(1), captures.get(2)) {
                            let module_name = module.as_str().trim();
                            for name in names.as_str().split(',') {
                                let clean_name = name.trim().split(" as ").next().unwrap_or("").trim();
                                if !clean_name.is_empty() {
                                    imports.push(format!("{}.{}", module_name, clean_name));
                                }
                            }
                        }
                    }
                }
                
                multiline_buffer.clear();
            }
        } else if import_regex.is_match(trimmed_line) {
            // Handle single line "import x" statements
            if let Some(captures) = import_regex.captures(trimmed_line) {
                if let Some(import_names) = captures.get(1) {
                    for name in import_names.as_str().split(',') {
                        let clean_name = name.trim().split(" as ").next().unwrap_or("").trim();
                        if !clean_name.is_empty() {
                            imports.push(clean_name.to_string());
                        }
                    }
                }
            }
            
            // Check if this is the start of a multiline import
            if continued_line_regex.is_match(trimmed_line) {
                in_multiline_import = true;
                multiline_buffer = trimmed_line.to_string();
            }
        } else if from_import_regex.is_match(trimmed_line) {
            // Handle "from x import y" statements
            if let Some(captures) = from_import_regex.captures(trimmed_line) {
                if let (Some(module), Some(names)) = (captures.get(1), captures.get(2)) {
                    let module_name = module.as_str().trim();
                    for name in names.as_str().split(',') {
                        let clean_name = name.trim().split(" as ").next().unwrap_or("").trim();
                        if !clean_name.is_empty() {
                            imports.push(format!("{}.{}", module_name, clean_name));
                        }
                    }
                }
            }
            
            // Check if this is the start of a multiline import
            if continued_line_regex.is_match(trimmed_line) {
                in_multiline_import = true;
                multiline_buffer = trimmed_line.to_string();
            }
        } else {
            // Regular code line
            cleaned_content.push_str(line);
            cleaned_content.push('\n');
        }
    }
    
    (imports, cleaned_content)
}
