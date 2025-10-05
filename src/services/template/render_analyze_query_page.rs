// src/services/template/render_analyze_query_page.rs
use crate::models::{Project, ChatMessage};
use super::TemplateService;
use crate::shared;

impl TemplateService {
    pub fn render_analyze_query_page(
        &self,
        project_name: &str,
        query: &str,
        relevant_files: &[String], // These are now the vector search results, minus LLM suggestions
        saved_context_files: &[String],
        project: &Project,
        existing_chat_history: &[ChatMessage],
        available_queries: &[(String, String)],
        current_query_id: &str,
        include_file_descriptions: bool,
        llm_suggested_files: &[String],
    ) -> String {
        let vector_files: Vec<String> = relevant_files.iter()
            .filter(|file| project.embeddings.contains_key(*file))
            .cloned()
            .collect();
        let query_id = if current_query_id.is_empty() {
            available_queries.last().map(|(id, _)| id.as_str()).unwrap_or("")
        } else {
            current_query_id
        };
        
        let llm_suggested_files_html = self.generate_llm_suggested_files_list(llm_suggested_files, saved_context_files, project);
        let relevant_files_html = self.generate_relevant_files_list(saved_context_files, &vector_files, project);
        
        // Combine all excluded files for the 'other files' list
        let mut all_excluded_files: Vec<String> = Vec::new();
        all_excluded_files.extend(llm_suggested_files.iter().cloned());
        // Use the `relevant_files` passed in (which is already filtered) for further exclusion
        all_excluded_files.extend(relevant_files.iter().cloned()); 

        let other_files_html = self.generate_other_files_list(project, &all_excluded_files, saved_context_files);
        
        let query_selector_html = self.generate_query_selector(available_queries, query_id);
        let last_model_message_index = existing_chat_history.iter().rposition(|msg| msg.role == "model");
        let chat_messages_html = existing_chat_history.iter().enumerate().map(|(index, msg)| {
            self.gen_chat_message_html(msg, index, last_model_message_index.map(|i| i == index).unwrap_or(false))
        }).collect::<Vec<_>>().join("\n");

        // Determine if the checkbox should be checked
        let descriptions_checked_attr = if include_file_descriptions { "checked" } else { "" };

        let llm_suggested_files_section = if !llm_suggested_files.is_empty() {
            format!(
                r#"
                <div class="file-list">
                    <h3>
                        LLM Suggested Files 
                        <button id="toggle-llm-suggested-files" class="toggle-button">Toggle All</button>
                    </h3>
                    <div id="llm-suggested-files-list">
                        {}
                    </div>
                </div>
                "#,
                llm_suggested_files_html
            )
        } else {
            "".to_string()
        };


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

                            <div class="checkbox-container">
                                <input type="checkbox" class="file-checkbox" id="include-descriptions-checkbox" {} />
                                <label for="include-descriptions-checkbox">Include file paths and descriptions in prompt</label>
                            </div>

                            {} <!-- LLM Suggested Files Section -->
                            
                            <div class="file-list">
                                <h3>
                                    Other Relevant Files?
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
                        <input type="hidden" id="project-source-dir" value="{}">
                        
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
            "<label>Initial Query: </label>".to_string() + query,
            descriptions_checked_attr, // Pass the checked attribute here
            llm_suggested_files_section, // <--- Insert the LLM suggested files HTML here
            relevant_files_html,
            other_files_html,
            query_id,
            project_name,
            query,
            project.source_dir, // Pass project.source_dir here
            chat_messages_html, // Use the generated chat HTML
            project_name,
            query
        )
    }
}