// src/services/file/extract_imports/mod.rs
pub mod rust;
pub mod python;
pub mod javascript;

pub fn extract_imports(file_content: &str, language: &str) -> (Vec<String>, String) {
    match language {
        "rs" => rust::rust_imports(file_content),
        "py" => python::python_imports(file_content),
        "js" | "ts" => javascript::javascript_imports(file_content),
        _ => (Vec::new(), file_content.to_string()), // No imports extracted
    }
}