// src/routes/ui/update_env.rs
use actix_web::{get, post, web, HttpResponse, Responder};
use std::env;
use std::fs::{read_to_string, write};
use crate::shared;

#[get("/update-env")]
pub async fn update_env() -> impl Responder {
    let env_path = env::current_dir().unwrap().join(".env");
    let env_content = read_to_string(&env_path).unwrap_or_default();

    let mut open_ai_key = "";
    let mut open_ai_org = "";
    let mut gemini_api_key = "";
    let mut anthropic_api_key = "";

    for line in env_content.lines() {
        if line.starts_with("OPEN_AI_KEY=") {
            open_ai_key = line.split('=').nth(1).unwrap_or("");
        } else if line.starts_with("GEMINI_API_KEY=") {
            gemini_api_key = line.split('=').nth(1).unwrap_or("");
        } else if line.starts_with("ANTHROPIC_API_KEY=") {
            anthropic_api_key = line.split('=').nth(1).unwrap_or("");
        } else if line.starts_with("OPEN_AI_ORG=") {
            open_ai_org = line.split('=').nth(1).unwrap_or("");
        }
    }

    let html = format!(
        r#"
        <html>
            <head>
                <title>Update Environment Variables</title>
                {}
            </head>
            <body>
                <h1>Update Environment Variables</h1>
                <p>These variables can be set by adding a ".env" file in this applications install directory. See <a target=”_blank” href="https://crates.io/crates/llm_api_access">llm_api_access</a> for example ".env" file structure.</p>
                <a href="/" class="center">Go Back</a>
                <form action="/update-env" method="post">
                    <label for="open_ai_org">OpenAI ORG:</label>
                    <input type="text" id="open_ai_org" name="open_ai_org" value="{open_ai_org}">
                    <br>
                    <label for="open_ai_key">OpenAI Key:</label>
                    <input type="text" id="open_ai_key" name="open_ai_key" value="{open_ai_key}">
                    <br>
                    <label for="gemini_api_key">Gemini API Key:</label>
                    <input type="text" id="gemini_api_key" name="gemini_api_key" value="{gemini_api_key}">
                    <br>
                    <label for="anthropic_api_key">Anthropic API Key:</label>
                    <input type="text" id="anthropic_api_key" name="anthropic_api_key" value="{anthropic_api_key}">
                    <br>
                    <button type="submit">Save</button>
                </form>
            </body>
        </html>
        "#,
        shared::FAVICON_HTML_STRING,
        open_ai_org = open_ai_org,
        open_ai_key = open_ai_key,
        gemini_api_key = gemini_api_key,
        anthropic_api_key = anthropic_api_key
    );

    HttpResponse::Ok().body(html)
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct EnvSetForm {
    open_ai_key: String,
    open_ai_org: String,
    gemini_api_key: String,
    anthropic_api_key: String,
}

#[post("/update-env")]
pub async fn save_env(form_data: web::Form<EnvSetForm>) -> HttpResponse {
    let form_data = form_data.into_inner();
    let env_path = env::current_dir().unwrap().join(".env");

    let mut env_content = String::new();

    if !form_data.open_ai_key.is_empty() {
        env_content.push_str(&format!("OPEN_AI_KEY={}\n", form_data.open_ai_key));
        env::set_var("OPEN_AI_KEY", &form_data.open_ai_key);
    }

    if !form_data.open_ai_org.is_empty() {
        env_content.push_str(&format!("OPEN_AI_ORG={}\n", form_data.open_ai_org));
        env::set_var("OPEN_AI_ORG", &form_data.open_ai_org);
    }

    if !form_data.gemini_api_key.is_empty() {
        env_content.push_str(&format!("GEMINI_API_KEY={}\n", form_data.gemini_api_key));
        env::set_var("GEMINI_API_KEY", &form_data.gemini_api_key);
    }

    if !form_data.anthropic_api_key.is_empty() {
        env_content.push_str(&format!("ANTHROPIC_API_KEY={}\n", form_data.anthropic_api_key));
        env::set_var("ANTHROPIC_API_KEY", &form_data.anthropic_api_key);
    }

    write(&env_path, env_content).unwrap();

    // Redirect to the home page
    HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish()
}