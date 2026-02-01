// src/services/path_utils.rs
use crate::models::Project;
use std::path::{Path, PathBuf};

pub struct PathUtils;

impl PathUtils {
    /// Normalizes a raw file path string into a canonical project-relative path
    /// that can be used as a key in `Project.file_descriptions` or `Project.embeddings`.
    ///
    /// This function tries several heuristics to match the raw path to an existing
    /// file key in the project's metadata, acknowledging that YAML files might
    /// use different path conventions (e.g., stripping `src/`, leading slashes).
    ///
    /// It prefers paths already present in the project metadata.
    pub fn normalize_project_path(
        raw_path: &str,
        project: &Project,
    ) -> Option<String> {
        let raw_path = raw_path.replace('\\', "/"); // Standardize separators

        // 1. Check for exact match first
        if project.file_descriptions.contains_key(&raw_path) || project.embeddings.contains_key(&raw_path) {
            return Some(raw_path);
        }

        // Helper to check if a candidate path exists in project maps
        let check_candidate = |candidate: &str| {
            if project.file_descriptions.contains_key(candidate) || project.embeddings.contains_key(candidate) {
                Some(candidate.to_string())
            } else {
                None
            }
        };

        // 2. Try stripping a leading slash
        if let Some(stripped) = raw_path.strip_prefix('/') {
            if let Some(normalized) = check_candidate(stripped) {
                return Some(normalized);
            }
        }

        // 3. Try prepending project.source_dir if the raw path looks like a relative path within source_dir
        if !raw_path.starts_with(&project.source_dir) {
            let candidate = PathBuf::from(&project.source_dir).join(&raw_path);
            if let Some(normalized) = check_candidate(&candidate.to_string_lossy()) {
                return Some(normalized);
            }
        }

        // 4. If raw_path contains "src/" but project.source_dir might be "src"
        //    (e.g., raw="/src/models/mod.rs", project.source_dir="src")
        if let Some(src_idx) = raw_path.find("src/") {
            let after_src = &raw_path[src_idx + 4..]; // Skip "src/"
            let candidate = PathBuf::from(&project.source_dir).join(after_src);
            if let Some(normalized) = check_candidate(&candidate.to_string_lossy()) {
                return Some(normalized);
            }
        }

        // 5. Check if the raw_path is a suffix of any known file path (e.g., "mod.rs" might match "src/models/mod.rs")
        //    This is more expensive, so do it last.
        for (file_path, _) in project.file_descriptions.iter() {
            if file_path.ends_with(&raw_path) {
                return Some(file_path.clone());
            }
        }
        for (file_path, _) in project.embeddings.iter() {
            if file_path.ends_with(&raw_path) {
                return Some(file_path.clone());
            }
        }

        None // No match found
    }
}