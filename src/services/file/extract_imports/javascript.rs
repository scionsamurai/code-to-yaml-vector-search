// src/services/file/extract_imports/javascript.rs
use regex::Regex;

pub fn javascript_imports(file_content: &str) -> (Vec<String>, String) {
    let mut imports = Vec::new();
    let mut cleaned_content = String::new();
    let mut in_multiline_comment = false;
    let mut skip_line = false;

    // Default import: "import Name from 'module'"
    let default_import_regex = Regex::new(r#"import\s+([A-Za-z0-9_$]+)\s+from\s+['"]([^'"]+)['"]"#).unwrap();

    // Named imports: "import { name1, name2 } from 'module'"
    let named_import_regex = Regex::new(r#"import\s+\{\s*([^}]+)\s*\}\s+from\s+['"]([^'"]+)['"]"#).unwrap();

    // Namespace import: "import * as name from 'module'"
    let namespace_import_regex = Regex::new(r#"import\s+\*\s+as\s+([A-Za-z0-9_$]+)\s+from\s+['"]([^'"]+)['"]"#).unwrap();

    // Side effect import: "import 'module'"
    let side_effect_import_regex = Regex::new(r#"import\s+['"]([^'"]+)['"]"#).unwrap();

    // Dynamic import: "import('./module')"
    let dynamic_import_regex = Regex::new(r#"import\s*\(\s*['"]([^'"]+)['"]"#).unwrap();

    for line in file_content.lines() {
        let trimmed_line = line.trim();
        
        // Handle multiline comments
        if in_multiline_comment {
            if trimmed_line.contains("*/") {
                in_multiline_comment = false;
            }
            continue;
        }
        
        // Skip single line comments
        if trimmed_line.starts_with("//") {
            cleaned_content.push_str(line);
            cleaned_content.push('\n');
            continue;
        }
        
        // Check for start of multiline comment
        if trimmed_line.contains("/*") && !trimmed_line.contains("*/") {
            in_multiline_comment = true;
            continue;
        }
        
        // Process imports
        let mut is_import_line = false;
        
        // Default imports
        if default_import_regex.is_match(trimmed_line) {
            if let Some(captures) = default_import_regex.captures(trimmed_line) {
                if let (Some(name), Some(module)) = (captures.get(1), captures.get(2)) {
                    imports.push(format!("default:{} from {}", name.as_str(), module.as_str()));
                }
            }
            is_import_line = true;
        }
        
        // Named imports
        if named_import_regex.is_match(trimmed_line) {
            if let Some(captures) = named_import_regex.captures(trimmed_line) {
                if let (Some(named_imports), Some(module)) = (captures.get(1), captures.get(2)) {
                    let module_path = module.as_str();
                    
                    for import_name in named_imports.as_str().split(',') {
                        let parts: Vec<&str> = import_name.trim().split(" as ").collect();
                        let original_name = parts[0].trim();
                        let imported_as = if parts.len() > 1 { parts[1].trim() } else { original_name };
                        
                        if !original_name.is_empty() {
                            imports.push(format!("{}:{} from {}", original_name, imported_as, module_path));
                        }
                    }
                }
            }
            is_import_line = true;
        }
        
        // Namespace imports
        if namespace_import_regex.is_match(trimmed_line) {
            if let Some(captures) = namespace_import_regex.captures(trimmed_line) {
                if let (Some(namespace), Some(module)) = (captures.get(1), captures.get(2)) {
                    imports.push(format!("*:{} from {}", namespace.as_str(), module.as_str()));
                }
            }
            is_import_line = true;
        }
        
        // Side effect imports
        if side_effect_import_regex.is_match(trimmed_line) && !trimmed_line.contains(" from ") {
            if let Some(captures) = side_effect_import_regex.captures(trimmed_line) {
                if let Some(module) = captures.get(1) {
                    imports.push(format!("side-effect:{}", module.as_str()));
                }
            }
            is_import_line = true;
        }
        
        // Dynamic imports
        if dynamic_import_regex.is_match(trimmed_line) {
            if let Some(captures) = dynamic_import_regex.captures(trimmed_line) {
                if let Some(module) = captures.get(1) {
                    imports.push(format!("dynamic:{}", module.as_str()));
                }
            }
            is_import_line = true;
        }
        
        if !is_import_line && !skip_line {
            cleaned_content.push_str(line);
            cleaned_content.push('\n');
        } else {
            skip_line = true;
        }
    }
    
    (imports, cleaned_content)
}
