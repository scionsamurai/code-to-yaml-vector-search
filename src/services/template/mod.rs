// src/services/template/mod.rs
mod render_search_results;
mod render_project_page;
mod file_graph;
mod render_analyze_query_page;
mod file_list_generator;

use crate::models::ChatMessage;

pub struct TemplateService;

impl TemplateService {
    pub fn new() -> Self {
        Self {}
    }

    fn generate_query_selector(&self, available_queries: &[(String, String)], current_query_id: &str) -> String {
        let mut options_html = String::new();
        let max_time_stamp = available_queries[available_queries.len() - 1].0.clone();
        for (timestamp, display_title) in available_queries {
            let selected ;
            if current_query_id.is_empty() {
                selected = timestamp == &max_time_stamp;
            } else {
                selected = match current_query_id {
                    id => timestamp == id
                };
            }
            let selected_attr = if selected { "selected" } else { "" };
            options_html.push_str(&format!(
                r#"<option value="{}" {}>{}</option>"#,
                timestamp, selected_attr, display_title
            ));
        }

        format!(
            r#"
            <label for="query-selector">Select Query:</label>
            <select id="query-selector" name="query_id">
                {}
            </select>
            "#,
            options_html
        )
    }

    pub fn gen_chat_message_html(&self, msg: &ChatMessage, index: usize, is_last_model_message: bool) -> String {
        let regenerate_button_html = if is_last_model_message && msg.role == "model" {
            r#"<button class="regenerate-message-btn" title="Regenerate response">Regenerate</button>"#.to_string()
        } else {
            "".to_string()
        };

        format!(
            r#"<div class="chat-message {}-message" data-message-index="{}">
                <div class="message-content">{}</div>
                <div class="message-controls">
                    <button class="edit-message-btn" title="Edit message">Edit</button>
                    <button class="hide-message-btn" title="{} message" data-hidden="{}">{}</button>
                    {}
                </div>
            </div>"#,
            msg.role,
            index,
            msg.content,
            if msg.hidden { "Unhide" } else { "Hide" },
            msg.hidden,
            if msg.hidden { "Unhide" } else { "Hide" },
            regenerate_button_html
        )
    }

}