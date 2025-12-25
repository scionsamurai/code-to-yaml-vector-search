// src/main.rs
use actix_files::Files;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;

mod models;
mod routes;
mod services;
pub mod shared;

// const IP_ADDRESS: &str = "0.0.0.0"; // Listen on all interfaces (for deployment or local network testing)
const IP_ADDRESS: &str = "127.0.0.1"; // Localhost for testing on host machine
const PORT: u16 = 8080;

use actix_web::{HttpResponse, Responder, get};
use serde::Serialize;
use std::fs;

pub fn render_svelte<T: Serialize>(
    component: &str, 
    title: Option<&str>, 
    extra_data: Option<T>
) -> impl Responder {
    // 1. Load the single "master" template
    let mut html = fs::read_to_string("static/index.html")
        .expect("Failed to find static/index.html");

    // 2. Prepare dynamic strings
    let page_title = title.unwrap_or("Svelte 5 App");
    let json_data = match extra_data {
        Some(data) => serde_json::to_string(&data).unwrap_or_else(|_| "null".to_string()),
        None => "null".to_string(),
    };

    // 3. Simple String Replacement (The "No-Tera" approach)
    html = html.replace("{{title}}", page_title);
    html = html.replace("{{component}}", component);
    html = html.replace("{{extra_data}}", &json_data);

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[derive(Serialize)]
struct UserInfo {
    id: i32,
    name: String,
}

#[get("/profile")]
async fn profile_page() -> impl Responder {
    let data = UserInfo { id: 42, name: "Rustacean".to_string() };
    
    // Usage: render_svelte(component, title, extra_data)
    render_svelte("Profile", Some("Test Title Input"), Some(data))
}

#[get("/index")]
async fn index_page() -> impl Responder {
    render_svelte("Index", Some("Home Page"), None::<()>)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    // std::env::set_var("RUST_LOG", "debug"); // debugging
    // env_logger::init(); // debugging

    let app_state = web::Data::new(models::AppState {
        output_dir: "output".to_string(),
    });

    println!("Starting server at http://{}:{}", IP_ADDRESS, PORT);
    HttpServer::new(move || {
        App::new()
            .service(Files::new("/static", "./static"))
            .app_data(app_state.clone())
            .configure(routes::ui::configure)
            .configure(routes::project::configure)
            .configure(routes::llm::configure)
            .configure(routes::query::configure)
            .configure(routes::git::configure)
            .configure(routes::analyze::configure) // ADD THIS LINE
            .service(profile_page)
            .service(index_page)
            
    })
    .bind(format!("{}:{}", IP_ADDRESS, PORT))?
    .run()
    .await
}
