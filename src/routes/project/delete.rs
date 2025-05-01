use actix_web::{delete, web, HttpResponse, Responder};
use crate::models::AppState;
use std::fs::remove_dir_all;
use std::path::Path;

#[delete("/delete/{name}")]
pub async fn delete(app_state: web::Data<AppState>, name: web::Path<String>) -> impl Responder {
    let name = name.into_inner();
    let output_dir = Path::new(&app_state.output_dir).join(&name);

    if output_dir.exists() {
        if let Err(e) = remove_dir_all(&output_dir) {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to delete project '{}': {}", name, e));
        }
        // Option 2: Return a success response without refreshing the page
        return HttpResponse::Ok().body("Project deleted successfully");
    } else {
        return HttpResponse::NotFound().body(format!("Project '{}' not found", name));
    }
}