// src/routes/llm/chat_analysis/models.rs
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ChatAnalysisRequest {
    pub project: String,
    pub message: String,
    pub query_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ResetAnalysisChatRequest {
    pub project: String,
    pub query_id: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateChatMessageRequest {
    pub project: String,
    pub role: String,
    pub content: String,
    pub index: usize,
    pub query_id: Option<String>,
}
