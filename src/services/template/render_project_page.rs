// src/services/template/render_project_page.rs
use crate::models::Project;
use super::TemplateService;


impl TemplateService {
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
                <script src="/static/split-file.js" type="module"></script>
            </head>
            <body>
                <div id="validationModal" class="path-comment-modal">
                    <div class="modal-content">
                        <span class="close">&times;</span>
                        <h2>Validation Results</h2>
                        <ul id="validationList"></ul>
                    </div>
                </div>
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
                            <textarea name="q" placeholder="Enter your query..." value="{}"></textarea>
                            <button type="submit">Search</button>
                        </form>
                    </div>

                    {}

                </div>
                <a href="/" class="center">Go Back</a>
                <input type="checkbox" id="trigger-checkbox">
                <label for="trigger-checkbox">Hide Regen Buttons</label>
                {}
                <script type="module">
                    import {{ suggestSplit }} from '/static/split-file.js';

                    //Make suggestSplit accessible globally by assigning it to window
                    window.suggestSplit = suggestSplit;
                </script>
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
}