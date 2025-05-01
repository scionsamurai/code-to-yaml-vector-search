// src/main.rs
use actix_web::{App, HttpServer, web};
use actix_files::Files;
use dotenv::dotenv;
use routes::llm::{regenerate_yaml, suggest_split, chat_split};
use routes::project::{create, update, delete, get_project};
use routes::ui::{home, update_env};

mod services;
mod routes;
mod models;

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
            .service(create::create)
            .service(get_project::get_project)
            .service(update::update)
            .service(regenerate_yaml)
            .service(delete::delete) 
            .service(update_env)
            .service(suggest_split)
            .service(chat_split)
            .service(Files::new("/static", "./static"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}