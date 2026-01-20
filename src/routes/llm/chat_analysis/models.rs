// src/routes/llm/chat_analysis/models.rs

use crate::models::ChatMessage;
use serde::{Deserialize, Serialize};
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
    pub message_id: Uuid,
    pub content: String,
    pub query_id: Option<String>,
    pub create_new_branch: bool, // New field for branching on edit
}

#[derive(Deserialize)]
pub struct RegenerateChatMessageRequest {
    pub project: String,
    pub query_id: Option<String>,
    pub message_id: Uuid, // The ID of the MODEL message to regenerate from
}

#[derive(Deserialize)]
pub struct ApplyCodeToFileRequest {
    pub project_name: String,
    pub file_path: String,
    pub code_content: String,
}

// --- NEW RESPONSE STRUCTS FOR DYNAMIC UPDATES ---

#[derive(Serialize)]
pub struct ChatAnalysisResponse {
    pub success: bool,
    pub user_message: ChatMessage,
    pub model_message: ChatMessage,
    pub new_current_node_id: Uuid,
    // Optionally, could include updated branch_display_data for the user message's parent
    // pub branch_data_for_user_parent: Option<BranchDisplayData>,
}

#[derive(Serialize)]
pub struct ResetAnalysisChatResponse {
    pub success: bool,
    pub initial_chat_history: Vec<ChatMessage>,
    pub new_current_node_id: Uuid, // The ID of the new root message after reset
}

#[derive(Serialize)]
pub struct UpdateChatMessageResponse {
    pub success: bool,
    // If branching, this is the new message. If in-place, this is the updated one.
    pub message: ChatMessage,
    pub new_current_node_id: Uuid, // Will be the same as old for in-place, new for branch
    pub parent_message_id: Option<Uuid>, // Needed to identify where to potentially update branch UI
    // pub branch_data_for_parent: Option<BranchDisplayData>, // If we were to update branch UI dynamically
}

#[derive(Serialize)]
pub struct RegenerateChatMessageResponse {
    pub success: bool,
    pub new_model_message: ChatMessage,
    pub new_current_node_id: Uuid,
    pub user_message_id: Uuid, // The user message this model message is a child of
    // pub branch_data_for_user_message: Option<BranchDisplayData>, // If we were to update branch UI dynamically
}