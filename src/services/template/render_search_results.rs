// src/services/template/render_search_results.rs
use super::TemplateService;

impl TemplateService {
    pub fn render_search_results(
        &self,
        query_text: &str,
        similar_files: &[(String, String, f32, std::option::Option<Vec<f32>>)],
        llm_analysis: &str,
        project_name: &str,
        query_id: &str,
    ) -> String {
        let mut search_results_html = format!(
            r#"<div class="search-results">
            <h2>Search Results for: "{}"</h2>
            <div class="result-files">"#,
            query_text
        );

        for (file_path, _yaml_content, score, _) in similar_files {
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

        if query_id != "transient_query_id" {
            // --- MODIFIED START ---
            // Update the form to use method="get" and build the URL with path parameters
            search_results_html.push_str(&format!(
                r#"<div class="query-actions">
                <form action="/analyze-query/{}/{}" method="get">
                    <input type="hidden" name="query" value="{}">
                    <button type="submit" class="analyze-button">Chat with Analysis</button>
                </form>
            </div>"#,
                project_name, query_id, query_text // project_name and query_id are now in the URL path
            ));
            // --- MODIFIED END ---
        }

        search_results_html
    }
}