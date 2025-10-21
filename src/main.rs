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
            .app_data(app_state.clone())
            .configure(routes::ui::configure)
            .configure(routes::project::configure)
            .configure(routes::llm::configure) 
            .configure(routes::query::configure)
            .configure(routes::git::configure)
            .service(Files::new("/static", "./static"))
            
    })
    .bind(format!("{}:{}", IP_ADDRESS, PORT))?
    .run()
    .await
}
