// src/routes/execute_query.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::models::AppState;
use crate::services::llm_service::LlmService;

#[derive(serde::Deserialize)]
pub struct ExecuteQueryForm {
    project: String,
    query: String,
    code: String,
}

#[post("/execute-query")]
pub async fn execute_query(
    _app_state: web::Data<AppState>,
    form: web::Form<ExecuteQueryForm>,
) -> impl Responder {
    // 1. Create the final prompt with code
    let final_prompt = format!(
        "Query: {}\n\nRelevant code:\n{}\n\nPlease provide a detailed answer to the query based on the provided code.",
        form.query, form.code
    );
    
    // 2. Send to LLM
    let llm_service = LlmService::new();
    let response = llm_service.get_analysis(&final_prompt, "anthropic").await;
    
    // 3. Return the response
    let html = format!(
        r#"
        <html>
            <head>
                <title>Query Result</title>
                <link rel="stylesheet" href="/static/project.css">
            </head>
            <body>
                <div class="head">
                    <h1>Query Result</h1>
                    <p>Project: {}</p>
                    <p>Query: {}</p>
                </div>
                
                <div class="result-container">
                    <h2>Response</h2>
                    <div class="response-content">
                        {}
                    </div>
                    
                    <h3>Code Used</h3>
                    <pre>{}</pre>
                    
                    <div class="actions">
                        <a href="/projects/{}" class="button">Back to Project</a>
                    </div>
                </div>
            </body>
        </html>
        "#,
        form.project,
        form.query,
        response.replace("\n", "<br>"),
        form.code,
        form.project
    );
    
    HttpResponse::Ok().body(html)
}