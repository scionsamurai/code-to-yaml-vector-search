// src/main.rs
use actix_files::Files;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;

mod models;
mod routes;
mod services;
pub mod shared;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    // std::env::set_var("RUST_LOG", "debug"); // debugging
    // env_logger::init(); // debugging

    let app_state = web::Data::new(models::AppState {
        output_dir: "output".to_string(),
    });

    println!("Starting server at http://127.0.0.1:8080");
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
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
