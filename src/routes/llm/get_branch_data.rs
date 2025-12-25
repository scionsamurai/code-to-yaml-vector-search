// src/routes/llm/get_branching_data.rs
use actix_web::{web, get, HttpResponse, Responder};
use crate::models::{AppState, BranchDisplayData};
use crate::services::project_service::ProjectService;
use uuid::Uuid;
use std::collections::HashMap;
use std::path::Path;

#[derive(serde::Deserialize)]
pub struct BranchingDataQuery {
    project_name: String,
    query_id: String,
}

#[get("/get-branching-data")]
pub async fn get_branching_data(
    app_state: web::Data<AppState>,
    query: web::Query<BranchingDataQuery>,
) -> impl Responder {
    let project_name = query.project_name.clone();
    let query_id = query.query_id.clone();

    let project_service = ProjectService::new();
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&project_name);

    let project = match project_service.load_project(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to load project: {}", e))
        }
    };

    let full_query_data = project_service.query_manager.load_query_data(&project_dir, &query_id).unwrap_or_default();

    // Pre-compute branching data for all potential parents
    let mut branch_display_map: HashMap<Uuid, BranchDisplayData> = HashMap::new();
    let current_node_id = full_query_data.current_node_id;

    // Group all messages by their parent_id
    let mut children_map: HashMap<Option<Uuid>, Vec<&crate::models::ChatMessage>> = HashMap::new();
    for msg in full_query_data.chat_nodes.values() {
        children_map.entry(msg.parent_id).or_default().push(msg);
    }

    // Iterate through all messages to find parents with multiple children
    for (parent_id_option, children) in children_map {
        if let Some(parent_id) = parent_id_option { // Only consider actual parents, not root messages
            if children.len() > 1 {
                let mut sorted_siblings: Vec<&crate::models::ChatMessage> = children.clone();
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

    match serde_json::to_string(&branch_display_map) {
        Ok(json) => HttpResponse::Ok()
            .content_type("application/json")
            .body(json),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to serialize branch display map: {}", e)),
    }
}