// src/main.rs
use actix_files::Files;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use routes::llm::{chat_split, regenerate_yaml, suggest_split, update_analysis_context};
use routes::project::{create, delete, get_project, update_settings, update_yaml};
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
            .service(create::create)
            .service(get_project::get_project)
            .service(update_yaml::update)
            .service(regenerate_yaml)
            .service(delete::delete)
            .service(update_env)
            .service(suggest_split)
            .service(chat_split)
            .service(routes::llm::chat_analysis)
            .service(routes::llm::save_analysis_history)
            .service(routes::llm::reset_analysis_chat)
            .service(routes::llm::analyze_query)
            .service(update_settings::update_settings)
            .service(update_analysis_context)
            .service(Files::new("/static", "./static"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
