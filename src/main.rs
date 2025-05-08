// src/main.rs
use actix_files::Files;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use routes::llm::{chat_split, regenerate_yaml, suggest_split, update_analysis_context};
use routes::ui::{home, update_env};

mod models;
mod routes;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    // std::env::set_var("RUST_LOG", "debug"); // debugging
    // env_logger::init(); // debugging

    let app_state = web::Data::new(models::AppState {
        output_dir: "output".to_string(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(home)
            .service(regenerate_yaml)
            .service(update_env)
            .service(suggest_split)
            .service(chat_split)
            .service(routes::llm::analyze_query)
            .service(update_analysis_context)
            .service(Files::new("/static", "./static"))
            .configure(routes::project::configure)
            .configure(routes::llm::chat_analysis::configure)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
