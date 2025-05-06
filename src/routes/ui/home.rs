// src/routes/ui/home.rs
use actix_web::{get, web, HttpResponse, Responder};
use crate::models::{AppState, Project};
use crate::services::file_service::FileService;
use std::fs::read_to_string;
use std::path::Path;

#[get("/")]
pub async fn home(app_state: web::Data<AppState>) -> impl Responder {
    let output_dir = Path::new(&app_state.output_dir);
    let mut projects = Vec::new();
    let file_service = FileService {};

    // Read existing projects from the output directory
    for entry in std::fs::read_dir(output_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            let project_settings_path = path.join("project_settings.json");
            if let Ok(project_settings_json) = read_to_string(project_settings_path) {
                if let Ok(project) = serde_json::from_str::<Project>(&project_settings_json) {
                    let needs_update = file_service.project_needs_update(&project, &app_state.output_dir);
                    projects.push((project, needs_update));
                }
            }
        }
    }

    let html = format!(
        r#"
        <html>
            <head>
                <title>Project Manager</title>
                <link rel="stylesheet" href="/static/home.css">
                <script src="/static/home.js"></script>
            </head>
            <body>
                <h1>Welcome to the Project-to-YAML converter!</h1>
                <p>Here, you can convert projects files to YAML representations.</p>
                <a href="/update-env">Update Environment Variables</a>
                <h2>Projects</h2>
                <ul>
                    {}
                </ul>
                <form action="/projects" method="post" class="form-container">
                    <label for="name">Project Name:</label>
                    <input type="text" id="name" name="name" required>
                    <label for="languages">File Extensions (comma-separated):</label>
                    <input type="text" id="languages" name="languages" required>
                    <label for="source_dir">Source Directory:</label>
                    <input type="text" id="source_dir" name="source_dir" required>
                    <label for="llm_select">Choose a Model:</label>
                    <select name="llms" id="llm_select">
                        <option value="gemini">Gemini</option>
                        <option value="openai">OpenAI</option>
                        <option value="anthropic">Anthropic</option>
                    </select>
                    <button type="submit">Create Project</button>
                </form>
            </body>
        </html>

    "#,
        projects
        .iter()
        .map(|(project, needs_update)| format!(
            r#"<li><a href="/projects/{}">{}</a>{}{}</li>"#,
            project.name,
            project.name,
            if *needs_update {
                format!(r#" <button onclick="window.location.href='/update/{}'" style="background-color: green; color: white;">Update</button>"#, project.name)
            } else {
                "".to_string()
            },
            format!(r#" <button onclick="deleteProject('{}')" style="background-color: red; color: white;">Delete</button>"#, project.name)
        ))
        .collect::<Vec<_>>()
        .join("")
    );

    HttpResponse::Ok().body(html)
}
