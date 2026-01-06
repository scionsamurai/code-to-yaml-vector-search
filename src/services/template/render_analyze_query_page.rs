// src/services/template/render_analyze_query_page.rs
use crate::models::{Project, ChatMessage, QueryData, BranchDisplayData};
use super::TemplateService;
use crate::shared;
use std::collections::HashMap; // <--- ADD this for HashMap
use uuid::Uuid; // <--- ADD this for Uuid

impl TemplateService {
    pub fn render_analyze_query_page(
        &self,
        project_name: &str,
        query: &str,
        relevant_files: &[String],
        saved_context_files: &[String],
        project: &Project,
        existing_chat_history: &[ChatMessage],
        full_query_data: &QueryData, // <--- NEW: Pass the full QueryData
        available_queries: &[(String, String)],
        current_query_id: &str,
        include_file_descriptions: bool,
        llm_suggested_files: &[String],
        auto_commit: bool,
        branch_name: String,
        all_branches: Vec<String>,
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

        // Pre-compute branching data for all potential parents
        let mut branch_display_map: HashMap<Uuid, BranchDisplayData> = HashMap::new();
        let current_node_id = full_query_data.current_node_id;

        // Group all messages by their parent_id
        let mut children_map: HashMap<Option<Uuid>, Vec<&ChatMessage>> = HashMap::new();
        for msg in full_query_data.chat_nodes.values() {
            children_map.entry(msg.parent_id).or_default().push(msg);
        }

        // Iterate through all messages to find parents with multiple children
        for (parent_id_option, children) in children_map {
            if let Some(parent_id) = parent_id_option { // Only consider actual parents, not root messages
                if children.len() > 1 {
                    let mut sorted_siblings: Vec<&ChatMessage> = children.clone();
                    // Sort siblings by timestamp to maintain a consistent order across page loads
                    sorted_siblings.sort_by_key(|s_msg| s_msg.timestamp.unwrap_or_else(chrono::Utc::now));

                    let sibling_ids: Vec<Uuid> = sorted_siblings.iter().map(|s_msg| s_msg.id).collect();
                    let total_siblings = sorted_siblings.len();

                    // Determine the current_index for this specific branch selector
                    // The current_index is the position of the sibling that is an ancestor of the active current_node_id.
                    let mut current_index_for_branch = 0;
                    for (idx, sibling) in sorted_siblings.iter().enumerate() {
                        // Check if this sibling is on the active path to current_node_id
                        let mut temp_current_id = current_node_id;
                        while let Some(ancestor_id) = temp_current_id {
                            if ancestor_id == sibling.id {
                                current_index_for_branch = idx;
                                break;
                            }
                            temp_current_id = full_query_data.chat_nodes.get(&ancestor_id).and_then(|m| m.parent_id);
                            if temp_current_id.is_none() && ancestor_id != sibling.id { // Reached root without finding sibling, break loop
                                break;
                            }
                        }
                    }

                    let branch_data = BranchDisplayData {
                        current_index: current_index_for_branch,
                        total_siblings,
                        sibling_ids,
                    };
                    branch_display_map.insert(parent_id, branch_data);
                }
            }
        }


        let mut chat_messages_html_parts = Vec::new();
        let last_model_message_id = existing_chat_history.iter().rev()
                                         .find(|msg| msg.role == "model")
                                         .map(|msg| msg.id);

        for msg in existing_chat_history {
            let is_last_model_message = last_model_message_id.map_or(false, |id| id == msg.id);

            // After the message, check if it's a parent that branches, and if so, add the navigation
            if let Some(bd) = branch_display_map.get(&msg.id) {
                // Generate the message HTML itself
                chat_messages_html_parts.push(self.gen_chat_message_html(msg, is_last_model_message, self.gen_branch_navigation_html(bd).as_str())); // <--- UPDATED CALL, no branch_data
            } else {
                // Just generate the message HTML without navigation
                chat_messages_html_parts.push(self.gen_chat_message_html(msg, is_last_model_message, ""));
            }
        }
        let chat_messages_html = chat_messages_html_parts.join("\n");


        // Determine if the checkbox should be checked
        let descriptions_checked_attr = if include_file_descriptions { "checked" } else { "" };

        // Add these lines to determine auto-commit and branch name state
        let auto_commit_checked_attr = if auto_commit { "checked" } else { "" };

        // Generate branch options for the dropdown
        let branch_options_html: String = all_branches.iter().map(|branch| {
            let selected = if &branch_name == branch { "selected" } else { "" };
            format!(r#"<option value="{}" {}>{}</option>"#, branch, selected, branch)
        }).collect();

        // Git Branch Selector HTML (to be placed in chat header)
        let git_branch_selector_html = if project.git_integration_enabled {
            format!(
                r#"
                <div class="git-branch-selector-container">
                    <label for="git-branch-selector">Current Repo Branch:</label>
                    <select id="git-branch-selector">
                        {}
                    </select>
                </div>
                "#,
                branch_options_html
            )
        } else {
            "".to_string()
        };

        // "Start New Branch" button logic: disabled if a feature branch is currently active
        let disable_new_branch_button = if project.git_integration_enabled && branch_name != "main" { "disabled" } else { "" };

        // "Merge to Main" button logic: visible only if current branch is not the default branch
        let merge_button_html = if project.git_integration_enabled {
            // Assuming "main" or "master" are common default branches.
            // This check could be more robust by fetching the actual default branch name.
            if !branch_name.is_empty() && branch_name != "main" && branch_name != "master" {
                r#"<button id="merge-to-main-button">Merge to Main</button>"#.to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

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

        let git_actions_html = if project.git_integration_enabled {
            format!(
                r#"
                <div class="git-actions">
                    <button id="commit-button">Commit</button>
                    <button id="push-changes-button">Push Changes</button>
                    <label>
                        Enable Auto-Commit for this Chat:
                        <input type="checkbox" id="auto-commit-checkbox" {} >
                    </label>
                    <button id="start-new-branch-button" {} >Start New Branch</button>
                    {} <!-- Merge button -->
                    <div id="git-action-message" class="git-message" style="display:none;"></div>
                </div>
                "#,
                auto_commit_checked_attr,
                disable_new_branch_button,
                merge_button_html
            )
        } else {
            "".to_string()
        };

        format!(
            r#"
            <!DOCTYPE html>
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
                                <input type="checkbox" class="descriptions-checkbox" id="include-descriptions-checkbox" {} />
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
                                </h3>
                                <div id="other-files-list">
                                    {}
                                </div>
                            </div>

                        </div>
                    </div>

                    <div class="chat-interface">
                        <div class="chat-header">
                            <h2>Chat about your code</h2>
                            {} <!-- Git Branch Selector -->
                            {}
                            {}
                        </div>
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
            git_branch_selector_html,
            git_actions_html, // Insert Git Actions HTML
            if project.git_integration_enabled {
                "<script src=\"/static/analyze-query/git-branch-actions.js\" type=\"module\"></script>".to_string()
             } else {
                 "".to_string()
             },
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