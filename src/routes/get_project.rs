// src/routes/get_project.rs
use actix_web::{get, web, HttpResponse, Responder};
use crate::models::{AppState, Project};
use std::fs::{read_dir, read_to_string};
use std::path::Path;

#[get("/projects/{name}")]
pub async fn get_project(app_state: web::Data<AppState>, name: web::Path<String>) -> impl Responder {
    let name = name.into_inner(); // Extract the inner String from the Path wrapper
    let output_dir = Path::new(&app_state.output_dir).join(&name);
    let project_settings_path = output_dir.join("project_settings.json");

    if let Ok(project_settings_json) = read_to_string(project_settings_path) {
        if let Ok(project) = serde_json::from_str::<Project>(&project_settings_json) {
            let yaml_files = read_dir(output_dir)
                .unwrap()
                .map(|entry| {
                    let entry = entry.unwrap();
                    let yaml_path = entry.path();
                    // let source_path = Path::new(&project.source_dir).join(yaml_path.file_name().unwrap());
                    let content = read_to_string(&yaml_path).unwrap();
                    if yaml_path.file_name().unwrap().to_string_lossy() != "project_settings.json" {
                        format!(
                            "<div class=\"page\"><p>---</p><h3>path: {}</h3><pre>{}</pre><button onclick=\"regenerate('{}', '{}')\">Regenerate</button></div>",
                            yaml_path.file_name().unwrap().to_string_lossy().replace("*", "/").replace(".yml", ""),
                            content.replace("---\n", ""),
                            project.name,
                            yaml_path.display()
                        )
                    } else {
                        "".to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("");

            let html = format!(
                r#"
                <html>
                    <head>
                        <title>{}</title>
                        <link rel="stylesheet" href="/static/project.css">
                        <script src="/static/project.js"></script>
                    </head>
                    <body>
                        <div class="head">
                            <h1>{}</h1>
                            <p>Languages: {}</p>
                            <p>Source Directory: {}</p>
                            <p>Model: {}</p>
                            <h2>YAML Representations</h2>
                        </div>
                        <a href="/" class="center">Go Back</a>
                        <input type="checkbox" id="trigger-checkbox">
                        <label for="trigger-checkbox">Hide Regen Buttons</label>
                        {}
                    </body>
                </html>
            "#,
                project.name,
                project.name,
                project.languages,
                project.source_dir,
                project.model,
                yaml_files
            );

            HttpResponse::Ok().body(html)
        } else {
            HttpResponse::InternalServerError().body("Failed to deserialize project settings")
        }
    } else {
        HttpResponse::NotFound().body("Project not found")
    }
}