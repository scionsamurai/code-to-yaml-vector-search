use crate::models::Project;
use super::TemplateService;

impl TemplateService {
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
}