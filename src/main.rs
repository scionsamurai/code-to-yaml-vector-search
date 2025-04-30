// src/main.rs
use actix_web::{App, HttpServer, web};
use actix_files::Files;
use dotenv::dotenv;
use routes::{create_project, get_project, home, update_project, regenerate_yaml, delete_project, update_env, execute_query, suggest_split};

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
            .service(home::home)
            .service(create_project::create_project)
            .service(get_project::get_project)
            .service(update_project::update_project)
            .service(regenerate_yaml::regenerate_yaml)
            .service(delete_project::delete_project) 
            .service(update_env::update_env)
            .service(update_env::save_env)
            .service(execute_query::execute_query)
            .service(suggest_split::suggest_split)
            .service(Files::new("/static", "./static"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}