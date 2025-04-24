// src/services/template_service.rs
use crate::models::Project;

pub struct TemplateService;

impl TemplateService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render_search_results(&self, query_text: &str, similar_files: &[(String, String, f32)], project_name: &str) -> String {
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
        
        search_results_html.push_str(
            r#"</div>
            <form action="/analyze-query" method="post">
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
                    <script src="/static/project.js"></script>
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
}