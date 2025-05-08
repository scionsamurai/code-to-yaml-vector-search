// src/routes/llm/chat_analysis/models.rs
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ChatAnalysisRequest {
    pub project: String,
    pub query: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct ResetAnalysisChatRequest {
    pub project: String,
}

#[derive(Deserialize)]
pub struct SaveAnalysisHistoryRequest {
    pub project: String,
    pub history: Vec<crate::models::ChatMessage>,
}

#[derive(Deserialize)]
pub struct UpdateChatMessageRequest {
    pub project: String,
    pub role: String,
    pub content: String,
    pub index: usize
}