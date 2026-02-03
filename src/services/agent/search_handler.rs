// src/services/agent/search_handler.rs

use crate::models::ChatMessage;

/// Generates a comprehensive query string for semantic (vector) search,
/// combining the initial query, relevant chat history, and the latest user message.
pub fn generate_contextual_vector_query(
    initial_query: &str,
    previous_history_for_llm: &Vec<ChatMessage>, // Use this for history
    user_message_content: &str,
) -> String {
    let mut query = String::new();
    query.push_str("Consider the following initial task:\n");
    query.push_str(initial_query);

    // Take the last few (e.g., 6) messages from history for conciseness and context
    // Ensure system/model intro messages are not included, only user/model turns
    let mut relevant_history_for_search: Vec<&ChatMessage> = previous_history_for_llm.iter()
        .rev()
        .filter(|msg| msg.role == "user" || msg.role == "model")
        .take(6) // Adjust number of messages as needed
        .collect();
    relevant_history_for_search.reverse();

    if !relevant_history_for_search.is_empty() {
        query.push_str("\n\nReview the recent conversation history to understand the current context and goal:\n");
        for msg in relevant_history_for_search {
            query.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }
    }

    query.push_str("\n\nBased on this, and the user's latest message:\n");
    query.push_str(user_message_content);
    query.push_str("\n\nWhat are the most relevant details for addressing the current request and conversation state? Provide a comprehensive and focused query for a semantic search. Your output will fill in the details missing from the latest message (since what you generate here will be prepended to the latest message before sending to the vector search).");
    query
}