// src/routes/llm/chat_analysis/update_message_visibility.rs
use crate::models::AppState;
use crate::services::project_service::ProjectService;
use actix_web::{post, web, HttpResponse};
use serde_json::Value;
use std::path::Path;
use uuid::Uuid; // <--- ADD THIS LINE

#[post("/update-message-visibility")]
pub async fn update_message_visibility(
    app_state: web::Data<AppState>,
    data: web::Json<Value>,
) -> HttpResponse {
    let project_service = ProjectService::new();

    // Parse request JSON
    let req = data.into_inner();

    let project_name = match req.get("project").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return HttpResponse::BadRequest().body("Missing 'project'"),
    };

    // Changed from index: usize to message_id: Uuid
    let message_id = match req.get("message_id").and_then(|v| v.as_str()) {
        Some(s) => match Uuid::parse_str(s) {
            Ok(uuid) => uuid,
            Err(_) => return HttpResponse::BadRequest().body("Invalid 'message_id' format"),
        },
        None => return HttpResponse::BadRequest().body("Missing 'message_id'"),
    };

    let hidden = match req.get("hidden").and_then(|v| v.as_bool()) {
        Some(b) => b,
        None => return HttpResponse::BadRequest().body("Missing or invalid 'hidden'"),
    };

    let query_id = match req.get("query_id").and_then(|v| v.as_str()) {
        Some(q) => q,
        None => return HttpResponse::BadRequest().body("Missing 'query_id'"),
    };

    // Load the project
    let output_dir = Path::new(&app_state.output_dir);
    let project_dir = output_dir.join(&project_name);

    // Update the message visibility
    let result = project_service.chat_manager.update_message_visibility(&project_service.query_manager, &project_dir, message_id, hidden, query_id);

    match result {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(message) => HttpResponse::BadRequest().body(message),
    }
}