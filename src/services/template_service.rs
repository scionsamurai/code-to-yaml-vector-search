// src/services/template_service.rs
use crate::models::Project;

pub struct TemplateService;

impl TemplateService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render_search_results(
        &self,
        query_text: &str,
        similar_files: &[(String, String, f32)],
        llm_analysis: &str,
        project_name: &str,
    ) -> String {
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
                file_path, score
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

        // Update the analyze button to use the new endpoint
        search_results_html.push_str(&format!(
            r#"<form action="/analyze-query" method="post">
            <input type="hidden" name="project" value="{}">
            <input type="hidden" name="query" value="{}">
            <button type="submit" class="analyze-button">Chat with Analysis</button>
        </form>
        </div>"#,
            project_name, query_text
        ));
        search_results_html
    }

    pub fn render_project_page(
        &self,
        project: &Project,
        search_results_html: &str,
        yaml_files: &str,
        query_value: &str,
    ) -> String {
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
                    
                    <!-- Project Settings Form -->
                    <div class="project-settings">
                        <form action="/update/{}/settings" method="post">
                            <div class="form-group">
                                <label for="languages">File Extensions (comma-separated):</label>
                                <input type="text" id="languages" name="languages" value="{}" required>
                            </div>
                            <div class="form-group">
                                <label for="model">Model:</label>
                                <select name="model" id="model">
                                    <option value="gemini" {}> Gemini</option>
                                    <option value="openai" {}> OpenAI</option>
                                    <option value="anthropic" {}> Anthropic</option>
                                </select>
                            </div>
                            <button type="submit">Update Settings</button>
                        </form>
                    </div>
                    
                    <p>Source Directory: {}</p>

                    <!-- Search Form -->
                    <div class="search-form">
                        <form action="/projects/{}" method="get">
                            <input type="text" name="q" placeholder="Enter your query..." value="{}">
                            <button type="submit">Search</button>
                        </form>
                    </div>

                    {}

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
            project.name,
            project.languages,
            if project.model == "gemini" { "selected" } else { "" },
            if project.model == "openai" { "selected" } else { "" },
            if project.model == "anthropic" { "selected" } else { "" },
            project.source_dir,
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


    pub fn render_analyze_query_page(
        &self,
        project_name: &str,
        query: &str,
        relevant_files: &[String],
        saved_context_files: &[String],
        project: &Project,
        existing_chat_html: &str
    ) -> String {
        // Generate file lists
        let relevant_files_html = self.generate_file_list(relevant_files, saved_context_files);
        let other_files_html = self.generate_other_files_list(project, relevant_files, saved_context_files);
        
        format!(
            r#"
            <html>
                <head>
                    <title>Code Analysis - {}</title>
                    <link rel="stylesheet" href="/static/project.css">
                    <link rel="stylesheet" href="/static/analyze-query.css">
                    <link rel="stylesheet" href="/static/split-chat.css">
                    <script src="/static/analyze-query.js" type="module"></script>
                </head>
                <body>
                <div class="head">
                    <h1>Code Analysis</h1>
                    <p>Project: {}</p>
                    <p>Query: {}</p>
                </div>
                

                <div class="analysis-container">
                    <div class="file-snippets">
                        <h2>Files for Analysis</h2>
                        
                        <div id="context-status" style="display: none; margin: 10px 0; padding: 5px; 
                            background-color: #f0f0f0; border-radius: 4px; transition: opacity 0.5s;">
                        </div>
                        
                        <div class="file-list">
                            <h3>
                                Relevant Files 
                                <button id="toggle-relevant-files" class="toggle-button">Toggle All</button>
                            </h3>
                            <div id="relevant-files-list">
                                {}
                            </div>
                        </div>
                        
                        <div class="file-list">
                            <h3>
                                Other Project Files
                                <button id="toggle-other-files" class="toggle-button">Toggle All</button>
                            </h3>
                            <div id="other-files-list">
                                {}
                            </div>
                        </div>
                        
                    </div>
                    
                    <div class="chat-interface">
                        <h2>Analysis Chat</h2>
                        <input type="hidden" id="project-name" value="{}">
                        <input type="hidden" id="query-text" value="{}">
                        
                        <div id="analysis-chat-container" class="chat-container">
                            {}
                        </div>
                        
                        <div class="chat-input">
                            <input type="text" id="analysis-message-input" placeholder="Ask a question about the code...">
                            <button id="analysis-send-button">Send</button>
                            <button id="analysis-reset-button" class="secondary">Reset Chat</button>
                        </div>
                    </div>
                </div>
                
                <div class="actions">
                    <a href="/projects/{}" class="button">Back to Project</a>
                </div>
                </body>
            </html>
            "#,
            project_name,
            project_name,
            query,
            relevant_files_html,
            other_files_html,
            project_name,
            query,
            existing_chat_html,
            project_name
        )
    }
    
    fn generate_file_list(&self, files: &[String], selected_files: &[String]) -> String {
        files.iter()
            .map(|file| {
                format!(
                    r#"<div class="file-item">
                        <input type="checkbox" class="file-checkbox" value="{}" {}> {}
                    </div>"#,
                    file,
                    if selected_files.contains(file) { "checked" } else { "" },
                    file
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn generate_other_files_list(&self, project: &Project, exclude_files: &[String], selected_files: &[String]) -> String {
        // Get all project files
        let all_files: Vec<String> = match &project.embeddings {
            embeddings => embeddings.keys().cloned().collect(),
        };

        // Filter out the files that are already in the relevant files list
        let other_files: Vec<String> = all_files.into_iter()
            .filter(|file| !exclude_files.contains(file))
            .collect();
        
        self.generate_file_list(&other_files, selected_files)
    }
}
