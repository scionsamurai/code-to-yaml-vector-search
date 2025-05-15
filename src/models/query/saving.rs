// src/models/query/saving.rs
use crate::models::{AppState, Project, QueryData};
use actix_web::web;
use crate::services::project_service::ProjectService;

impl Project {
    pub fn save_new_query_data(
        &self,
        app_state: &web::Data<AppState>,
        query_data: QueryData,
    ) -> String {
        let project_service = ProjectService::new();
        let project_dir = self.get_project_dir(app_state);
        let filename = project_service.generate_query_filename();

        let _ = project_service.save_query_data(&project_dir, &query_data, &filename);
        filename
    }

    // Update an existing query data or create a new one
    pub fn update_query_data(
        &self,
        app_state: &web::Data<AppState>,
        query_filename: &str,
        update_fn: impl FnOnce(&mut QueryData)
    ) -> Result<(), String> {
        let project_service = ProjectService::new();
        let project_dir = self.get_project_dir(app_state);

        // Try to load existing query data or create new
        let (mut query_data, filename) = match self.load_query_data_by_filename(app_state, query_filename) {
            Ok(Some(qd)) => {
                (qd, query_filename.to_string())
            }
            _ => {
                // Create new query data and filename
                (
                    QueryData::default(),
                    project_service.generate_query_filename(),
                )
            }
        };

        // Apply the update function
        update_fn(&mut query_data);

        // Save the updated query data
        project_service.save_query_data(&project_dir, &query_data, &filename)
    }
}