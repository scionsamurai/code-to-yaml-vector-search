// src/services/template/file_graph.rs
use super::TemplateService;

impl TemplateService {
    pub fn generate_file_graph_html(&self, project_name: &str, file_descriptions: &[(String, String)]) -> String {
        // Sort the files
        let mut sorted_descriptions = file_descriptions.to_vec();
        sorted_descriptions.sort_by_key(|(path, _)| path.clone());

        // Find common prefix
        let common_prefix = sorted_descriptions
            .iter()
            .map(|(path, _)| path.clone())
            .reduce(|a, b| self.common_path_prefix(&a, &b))
            .unwrap_or_default();

        // Build indented list
        let mut indented_lines = vec![];
        for (full_path, description) in &sorted_descriptions {
            let trimmed = full_path.strip_prefix(&common_prefix).unwrap_or(full_path);
            indented_lines.push(format!("{} // {}", trimmed, description));
        }

        format!(
            r#"<div id="graphDiv"><input type="checkbox" id="fileGraph">
    <label for="fileGraph" style="cursor: pointer; font-weight: bold;">Show File Graph</label>
    <pre><button onclick="validateFilePaths('{}')">Validate File Path Comments</button><br>{}</pre></div>"#,
            project_name,
            indented_lines.join("\n")
        )
    }

    fn common_path_prefix(&self, a: &str, b: &str) -> String {
        a.split('/')
            .zip(b.split('/'))
            .take_while(|(x, y)| x == y)
            .map(|(x, _)| x)
            .collect::<Vec<_>>()
            .join("/") + "/"
    }
}