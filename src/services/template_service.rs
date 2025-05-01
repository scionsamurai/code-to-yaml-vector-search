// src/services/template_service.rs
use crate::models::Project;

pub struct TemplateService;

impl TemplateService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render_search_results(&self, query_text: &str, similar_files: &[(String, String, f32)], llm_analysis: &str, project_name: &str) -> String {
        let mut search_results_html = format!(
            r#"<div class="search-results">
                <h2>Search Results for: "{}"</h2>
                <div class="result-files">"#,
            query_text
        );
        
        for (file_path, _yaml_content, score) in similar_files {
            search_results_html.push_str(&format!(
                r#"<div class="result-file">
                    <h3>{} (Score: {:.4})</h3>
                </div>"#,
                file_path,
                score
            ));
        }
        
        // Add LLM analysis section
        search_results_html.push_str(&format!(
            r#"</div>
            <div class="llm-analysis">
                <h2>Analysis</h2>
                <div class="analysis-content">
                    {}
                </div>
            </div>"#,
            llm_analysis.replace("\n", "<br>")
        ));
        
        search_results_html.push_str(
            r#"<form action="/analyze-query" method="post">
                <input type="hidden" name="project" value="{}">
                <input type="hidden" name="query" value="{}">
                <button type="submit">Analyze Query</button>
            </form>
            </div>"#
        );
        
        search_results_html.replace("{}", project_name)
            .replace("{}", query_text)
    }

    pub fn render_project_page(&self, 
            project: &Project, 
            search_results_html: &str, 
            yaml_files: &str,
            query_value: &str) -> String {
        format!(
        r#"
        <html>
            <head>
                <title>{}</title>
                <link rel="stylesheet" href="/static/project.css">
                <link rel="stylesheet" href="/static/split-chat.css">
                <link rel="stylesheet" href="/static/split-modal.css">
                <script src="/static/project.js"></script>
                <script src="/static/split-file.js"></script>
            </head>
            <body>
                <div class="head">
                    <h1>{}</h1>
                    <p>Languages: {}</p>
                    <p>Source Directory: {}</p>
                    <p>Model: {}</p>

                    <!-- Search Form -->
                    <div class="search-form">
                        <form action="/projects/{}" method="get">
                            <input type="text" name="q" placeholder="Enter your query..." value="{}">
                            <button type="submit">Search</button>
                        </form>
                    </div>

                    {}

                    <h2>YAML Representations</h2>
                </div>
                <a href="/" class="center">Go Back</a>
                <input type="checkbox" id="trigger-checkbox">
                <label for="trigger-checkbox">Hide Regen Buttons</label>
                {}
            </body>
        </html>
        "#,
        project.name,
        project.name,
        project.languages,
        project.source_dir,
        project.model,
        project.name,
        query_value,
        search_results_html,
        yaml_files
        )
    }

    // Added from project_service.rs
    pub fn generate_file_graph_html(&self, file_descriptions: &[(String, String)]) -> String {
        // Sort the files
        let mut sorted_descriptions = file_descriptions.to_vec();
        sorted_descriptions.sort_by_key(|(path, _)| path.clone());
        
        // Find common prefix
        let common_prefix = sorted_descriptions.iter()
            .map(|(path, _)| path.clone())
            .reduce(|a, b| self.common_path_prefix(&a, &b))
            .unwrap_or_default();
        
        // Build indented list
        let mut indented_lines = vec![];
        for (full_path, description) in &sorted_descriptions {
            let trimmed = full_path.strip_prefix(&common_prefix).unwrap_or(full_path);
            indented_lines.push(format!(
                "{} // {}",
                trimmed, description
            ));
        }
        
        format!(
            r#"<div id="graphDiv"><input type="checkbox" id="fileGraph">
    <label for="fileGraph" style="cursor: pointer; font-weight: bold;">Show File Graph</label>
    <style>
        #graphDiv {{
            display: flex;
            flex-direction: column;
            align-items: center;
        }}
        #fileGraph ~ pre {{
            display: none;
        }}
        #fileGraph:checked ~ pre {{
            display: block;
            background: white;
            overflow: scroll;
            width: 80%;
            padding: 1rem;
        }}
    </style>
    <pre>{}</pre></div>"#,
            indented_lines.join("\n")
        )
    }

    // Helper function moved from project_service.rs
    fn common_path_prefix(&self, a: &str, b: &str) -> String {
        a.split('/')
            .zip(b.split('/'))
            .take_while(|(x, y)| x == y)
            .map(|(x, _)| x)
            .collect::<Vec<_>>()
            .join("/")
    }
}