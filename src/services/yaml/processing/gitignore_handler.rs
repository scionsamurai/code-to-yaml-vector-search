use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::Path;

    pub fn is_file_gitignored(source_dir: &str, source_path: &str, original_source_path: &Path) -> bool {
        let mut gitignore_paths = Vec::new();
        let source_dir_path = Path::new(source_dir);

        // Read .gitignore file
        if let Ok(gitignore_file) = File::open(source_dir_path.join(".gitignore")) {
            for line in BufReader::new(gitignore_file).lines() {
                if let Ok(path) = line {
                    // Normalize the gitignore path (remove leading / if present)
                    let normalized_path = path.trim().trim_start_matches('/').to_string();
                    if !normalized_path.is_empty() {
                        gitignore_paths.push(normalized_path);
                    }
                }
            }
        }

        // Get the relative path from the project source directory
        let relative_path = if let Ok(rel_path) = original_source_path.strip_prefix(source_dir_path) {
            rel_path.to_string_lossy().to_string()
        } else {
            source_path.to_string()
        };

        // Check if any gitignore path applies to this file
        gitignore_paths.iter().any(|ignore_pattern| {
            // For directory patterns that end with /, check if the file is in that directory
            if ignore_pattern.ends_with('/') {
                let dir_pattern = ignore_pattern.trim_end_matches('/');
                relative_path.starts_with(dir_pattern)
            }
            // For directory name patterns without trailing /, check both directory and filename
            else {
                relative_path == *ignore_pattern
                    || relative_path.starts_with(&format!("{}/", ignore_pattern))
            }
        })
    }