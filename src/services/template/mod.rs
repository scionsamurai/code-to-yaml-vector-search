// src/services/template/mod.rs
mod render_search_results;
mod render_project_page;
mod file_graph;
mod file_list_generator;

pub struct TemplateService;

impl TemplateService {
    pub fn new() -> Self {
        Self {}
    }
}
