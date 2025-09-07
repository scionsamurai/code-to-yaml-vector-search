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
    pub hidden: Option<bool>,
}

#[derive(Deserialize)]
pub struct RegenerateChatMessageRequest { // New struct for regenerate request
    pub project: String,
    pub query_id: Option<String>,
    pub index: usize, // Index of the model message to regenerate
}

#[derive(Deserialize)]
pub struct ApplyCodeToFileRequest {
    pub project: String,
    pub file_path: String,
    pub content: String,
}