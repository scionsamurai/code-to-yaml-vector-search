// src/routes/project/git_env_settings.rs
use actix_web::{get, post, web, HttpResponse, Responder};
use crate::models::AppState;
use crate::shared; // For favicon HTML
use std::path::Path;
use std::fs::{read_to_string, write};
use std::collections::HashMap; // Needed for manual parsing

#[derive(serde::Deserialize)]
pub struct GitEnvForm {
    git_author_name: String,
    git_author_email: String,
    git_username: String,
    git_password: String,
}

// Helper function to parse .env content into a HashMap
fn parse_env_content(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue; // Skip comments and empty lines
        }
        if let Some((key, value)) = line.split_once('=') {
            map.insert(key.to_string(), value.to_string());
        }
    }
    map
}

#[get("/projects/{name}/git-env")]
pub async fn get_git_env_settings(app_state: web::Data<AppState>, name: web::Path<String>) -> impl Responder {
    let project_name = name.into_inner();
    let project_dir = Path::new(&app_state.output_dir).join(&project_name);
    let project_env_path = project_dir.join(".env");

    let mut git_author_name = String::new();
    let mut git_author_email = String::new();
    let mut git_username = String::new();
    let mut git_password = String::new();

    if project_env_path.exists() {
        let env_content = read_to_string(&project_env_path).unwrap_or_default();
        let parsed_env = parse_env_content(&env_content); // Use our custom parser

        if let Some(name) = parsed_env.get("GIT_AUTHOR_NAME") {
            git_author_name = name.to_string();
        }
        if let Some(email) = parsed_env.get("GIT_AUTHOR_EMAIL") {
            git_author_email = email.to_string();
        }
        if let Some(username) = parsed_env.get("GIT_USERNAME") {
            git_username = username.to_string();
        }
        if let Some(password) = parsed_env.get("GIT_PASSWORD") {
            git_password = password.to_string();
        }
    }

    let html = format!(
        r#"
        <html>
            <head>
                <title>Git Environment Settings for {project_name}</title>
                {}
                <link rel="stylesheet" href="/static/home.css">
            </head>
            <body>
                <h1>Git Environment Settings for {project_name}</h1>
                <a href="/projects/{project_name}" class="center">Go Back to Project</a>
                <form action="/projects/{project_name}/git-env" method="post">
                    <label for="git_author_name">Git Author Name:</label>
                    <input type="text" id="git_author_name" name="git_author_name" value="{git_author_name}">
                    <br>
                    <label for="git_author_email">Git Author Email:</label>
                    <input type="email" id="git_author_email" name="git_author_email" value="{git_author_email}">
                    <br>
                    <label for="git_username">Git Username:</label>
                    <input type="text" id="git_username" name="git_username" value="{git_username}">
                    <br>
                    <label for="git_password">Git Password:</label>
                    <input type="password" id="git_password" name="git_password" value="{git_password}">
                    <br>
                    <button type="submit">Save Git Settings</button>
                </form>
            </body>
        </html>
        "#,
        shared::FAVICON_HTML_STRING,
        project_name = project_name,
        git_author_name = git_author_name,
        git_author_email = git_author_email,
        git_username = git_username,
        git_password = git_password
    );

    HttpResponse::Ok().body(html)
}

#[post("/projects/{name}/git-env")]
pub async fn post_git_env_settings(
    app_state: web::Data<AppState>,
    name: web::Path<String>,
    form_data: web::Form<GitEnvForm>,
) -> HttpResponse {
    let project_name = name.into_inner();
    let project_dir = Path::new(&app_state.output_dir).join(&project_name);
    let project_env_path = project_dir.join(".env");

    let mut current_env_vars = HashMap::new();
    if project_env_path.exists() {
        if let Ok(existing_content) = read_to_string(&project_env_path) {
            current_env_vars = parse_env_content(&existing_content);
        }
    }

    // Update or insert the Git specific variables
    if !form_data.git_author_name.is_empty() {
        current_env_vars.insert("GIT_AUTHOR_NAME".to_string(), form_data.git_author_name.clone());
    } else {
        current_env_vars.remove("GIT_AUTHOR_NAME");
    }
    if !form_data.git_author_email.is_empty() {
        current_env_vars.insert("GIT_AUTHOR_EMAIL".to_string(), form_data.git_author_email.clone());
    } else {
        current_env_vars.remove("GIT_AUTHOR_EMAIL");
    }
    if !form_data.git_username.is_empty() {
        current_env_vars.insert("GIT_USERNAME".to_string(), form_data.git_username.clone());
    } else {
        current_env_vars.remove("GIT_USERNAME");
    }
    if !form_data.git_password.is_empty() {
        current_env_vars.insert("GIT_PASSWORD".to_string(), form_data.git_password.clone());
    } else {
        current_env_vars.remove("GIT_PASSWORD");
    }

    // Reconstruct the .env file content
    let mut env_content = String::new();
    for (key, value) in current_env_vars {
        env_content.push_str(&format!("{}={}\n", key, value));
    }

    if let Err(e) = write(&project_env_path, env_content) {
        eprintln!("Failed to write project .env file: {}", e);
        return HttpResponse::InternalServerError().body(format!("Failed to save Git settings: {}", e));
    }

    // Redirect back to the project settings page or a confirmation
    HttpResponse::SeeOther()
        .append_header(("Location", format!("/projects/{}", project_name)))
        .finish()
}