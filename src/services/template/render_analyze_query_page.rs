// src/services/template/render_analyze_query_page.rs
use crate::models::{Project, ChatMessage};
use super::TemplateService;
use crate::shared;

impl TemplateService {
    pub fn render_analyze_query_page(
        &self,
        project_name: &str,
        query: &str,
        relevant_files: &[String],
        saved_context_files: &[String],
        project: &Project,
        existing_chat_history: &[ChatMessage],
        available_queries: &[(String, String)],
        current_query_id: &str,
    ) -> String {
        
        let relevant_files_html = self.generate_file_list(relevant_files, saved_context_files, project);
        let other_files_html = self.generate_other_files_list(project, relevant_files, saved_context_files);
        let query_selector_html = self.generate_query_selector(available_queries, current_query_id);
        let last_model_message_index = existing_chat_history.iter().rposition(|msg| msg.role == "model");
        let chat_messages_html = existing_chat_history.iter().enumerate().map(|(index, msg)| {
            self.gen_chat_message_html(msg, index, last_model_message_index.map(|i| i == index).unwrap_or(false))
        }).collect::<Vec<_>>().join("\n");

        format!(
            r#"
            <html>
                <head>
                    <title>Code Analysis - {}</title>
                    <link rel="stylesheet" href="/static/analysis.css">
                    <link rel="stylesheet" href="/static/global.css">
                    <script type="importmap">
                    {{
                        "imports": {{
                            "shiki": "https://esm.sh/shiki@3.0.0"
                        }}
                    }}
                    </script>
                    <script src="/static/analyze-query.js" type="module"></script>
                    <script src="/static/yaml-checkbox-logic.js"></script>
                    <script src="https://cdn.jsdelivr.net/npm/marked/marked.min.js"></script>
        {}
                </head>
                <body>
                <div class="head">
                    <h1>Code Analysis</h1>
                </div>
                
                <div class="analysis-container">
                    <div class="editable-query">
                        <p>Project: {}</p>
                        <div class="query-selector">
                            {}
                            <button id="edit-title-btn" class="secondary">Edit Title</button>
                        </div>
                        <div class="query-display-container">
                            <p id="query-display">{}</p>
                            <button id="edit-query-btn" class="secondary">Edit Query</button>
                        </div>
                            <h2>Files for Analysis</h2>
                        <div class="file-snippets">
                            
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
                    </div>
                    
                    <div class="chat-interface">
                        <h2>Analysis Chat</h2>
                        <input type="hidden" id="query-id" value="{}">
                        <input type="hidden" id="project-name" value="{}">
                        <input type="hidden" id="query-text" value="{}">
                        <input type="hidden" id="project-source-dir" value="{}"> <!-- ADDED THIS LINE -->
                        
                        <div id="analysis-chat-container" class="chat-container">
                            {}
                        </div>
                        
                        <div class="chat-input">
                            <textarea id="analysis-message-input" placeholder="Ask a question about the code..."></textarea>
                            <button id="analysis-send-button">Send</button>
                            <button id="analysis-reset-button" class="secondary">Reset Chat</button>
                        </div>
                    </div>
                </div>

                <div id="search-results-analysis-modal" class="analysis-search-modal">
                    <div class="analysis-search-modal-content">
                        <div class="modal-header">
                            <h3>Search Results</h3>
                            <span class="close-search-modal">&times;</span>
                        </div>
                        <div id="search-results-content"></div>
                    </div>
                </div>

                <div class="actions">
                    <a href="/projects/{}" class="button">Back to Project</a>
                </div>
                <div id="query-edit-modal" class="modal">
                    <div class="modal-content">
                        <div class="modal-header">
                            <h3>Edit Query</h3>
                            <span class="close-modal">&times;</span>
                        </div>
                        <div class="modal-body">
                            <textarea id="editable-query-text" rows="5" cols="50">{}</textarea>
                        </div>
                        <div class="modal-footer">
                            <button id="update-query-btn" class="primary">Update Query</button>
                            <button id="cancel-query-btn" class="secondary">Cancel</button>
                        </div>
                    </div>
                </div>

                <div id="title-edit-modal" class="modal">
                    <div class="modal-content">
                        <div class="modal-header">
                            <h3>Edit Title</h3>
                            <span class="close-modal">&times;</span>
                        </div>
                        <div class="modal-body">
                            <input type="text" id="editable-title-text" value="">
                        </div>
                        <div class="modal-footer">
                            <button id="update-title-btn" class="primary">Update Title</button>
                            <button id="cancel-title-btn" class="secondary">Cancel</button>
                        </div>
                    </div>
                </div>
                </body>
            </html>
            "#,
            project_name,
            shared::FAVICON_HTML_STRING,
            project_name,
            query_selector_html,
            "<label>Query: </label>".to_string() + query,
            relevant_files_html,
            other_files_html,
            current_query_id,
            project_name,
            query,
            project.source_dir, // Pass project.source_dir here
            chat_messages_html, // Use the generated chat HTML
            project_name,
            query
        )
    }
}