// src/services/template/mod.rs
mod render_search_results;
mod render_project_page;
mod file_graph;
mod render_analyze_query_page;
mod file_list_generator;

use crate::models::{ChatMessage, BranchDisplayData};
use uuid::Uuid; // <--- ADD THIS LINE

pub struct TemplateService;

impl TemplateService {
    pub fn new() -> Self {
        Self {}
    }

    fn generate_query_selector(&self, available_queries: &[(String, String)], current_query_id: &str) -> String {
        let mut options_html = String::new();
        // The logic here assumes `available_queries` is sorted by timestamp and `current_query_id`
        // is the filename. The `max_time_stamp` logic for default selection is a bit
        // convoluted, usually `current_query_id` would be provided directly from the request.
        // For now, I'll keep the existing behavior for selecting an option,
        // but note this might need review if query selection behavior is unexpected.
        let max_time_stamp_filename = available_queries.last().map(|(filename, _)| filename.clone()).unwrap_or_default();

        for (filename, display_title) in available_queries {
            let selected ;
            if current_query_id.is_empty() {
                selected = filename == &max_time_stamp_filename; // Compare filename, not timestamp
            } else {
                selected = filename == current_query_id; // Compare filename directly
            }
            let selected_attr = if selected { "selected" } else { "" };
            options_html.push_str(&format!(
                r#"<option value="{}" {}>{}</option>"#,
                filename, selected_attr, display_title
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

    // --- UPDATED gen_chat_message_html ---
    // Now accepts NO BranchDisplayData, only message and is_last_model_message
    pub fn gen_chat_message_html(&self, msg: &ChatMessage, is_last_model_message: bool, navigation_html: &str) -> String {
        let regenerate_button_html = if is_last_model_message && msg.role == "model" {
            r#"<button class="regenerate-message-btn" title="Regenerate response">Regenerate</button>"#.to_string()
        } else {
            "".to_string()
        };

        format!(
            r#"<div class="chat-message {}-message" data-message-id="{}">
                <div class="message-content">{}</div>
                <div class="message-controls">
                    <button class="edit-message-btn" title="Edit message">Edit</button>
                    <button class="hide-message-btn" title="{} message" data-hidden="{}">{}</button>
                    {}
                </div>
                {}
            </div>"#,
            msg.role,
            msg.id,
            msg.content, // Assuming content is already HTML escaped for display
            if msg.hidden { "Unhide" } else { "Hide" },
            msg.hidden,
            if msg.hidden { "Unhide" } else { "Hide" },
            regenerate_button_html,
            navigation_html
        )
    }

    // --- NEW gen_branch_navigation_html ---
    pub fn gen_branch_navigation_html(&self, bd: &BranchDisplayData) -> String {
        if bd.total_siblings > 1 {
            let prev_index = bd.current_index.checked_sub(1);
            let next_index = if bd.current_index + 1 < bd.total_siblings { Some(bd.current_index + 1) } else { None };

            let prev_button = if let Some(idx) = prev_index {
                format!(
                    r#"<button class="branch-nav-btn" data-nav-target-id="{}" data-nav-direction="prev" title="Previous Branch">&larr;</button>"#,
                    bd.sibling_ids[idx]
                )
            } else {
                r#"<button class="branch-nav-btn disabled" disabled title="No Previous Branch">&larr;</button>"#.to_string()
            };
            let next_button = if let Some(idx) = next_index {
                format!(
                    r#"<button class="branch-nav-btn" data-nav-target-id="{}" data-nav-direction="next" title="Next Branch">&rarr;</button>"#,
                    bd.sibling_ids[idx]
                )
            } else {
                r#"<button class="branch-nav-btn disabled" disabled title="No Next Branch">&rarr;</button>"#.to_string()
            };

            format!(
                r#"<div class="branch-navigation">
                    {}
                    <span>{} of {}</span>
                    {}
                </div>"#,
                prev_button,
                bd.current_index + 1, // 1-indexed display
                bd.total_siblings,
                next_button
            )
        } else {
            "".to_string()
        }
    }

}