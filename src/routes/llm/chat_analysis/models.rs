// src/routes/llm/chat_analysis/models.rs
use serde::Deserialize;
use uuid::Uuid;

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
    pub content: String,
    pub message_id: Uuid,
    pub query_id: Option<String>,
    #[serde(default)] // Default to false if not provided
    pub create_new_branch: bool, // <--- NEW FIELD
}

#[derive(Deserialize)]
pub struct RegenerateChatMessageRequest {
    pub project: String,
    pub query_id: Option<String>,
    pub message_id: Uuid,
}

#[derive(Deserialize)]
pub struct ApplyCodeToFileRequest {
    pub project: String,
    pub file_path: String,
    pub content: String,
}