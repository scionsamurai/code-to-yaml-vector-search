// src/services/project_service/chat_management.rs

use crate::models::ChatMessage;
use std::path::Path;
use crate::services::project_service::query_management::QueryManager;
use uuid::Uuid;
use std::collections::HashMap;
use chrono::Utc; // Import Utc for timestamp

pub struct ChatManager;

impl ChatManager {
    pub fn new() -> Self {
        Self {}
    }

    /// Reconstructs the active chat branch history as a linear Vec<ChatMessage>
    /// starting from the current_node_id and traversing upwards via parent_id.
    pub fn get_active_chat_branch(
        &self,
        query_data_nodes: &HashMap<Uuid, ChatMessage>,
        current_node_id: Option<Uuid>,
    ) -> Vec<ChatMessage> {
        let mut history = Vec::new();
        let mut current_id = current_node_id;

        while let Some(id) = current_id {
            if let Some(message) = query_data_nodes.get(&id) {
                history.push(message.clone());
                current_id = message.parent_id;
            } else {
                // Should not happen if data integrity is maintained
                eprintln!("Warning: Message with ID {:?} not found in chat_nodes while traversing history.", id);
                break;
            }
        }
        history.reverse(); // To get chronological order (oldest first)
        history
    }

    // Will handle logic from Project::get_analysis_chat_history
    pub fn get_analysis_chat_history(
        &self,
        query_manager: &QueryManager, // Dependency injection
        project_dir: &Path,
        query_filename: &str,
    ) -> Vec<ChatMessage> {
        match query_manager.load_query_data_by_filename(project_dir, query_filename) {
            Ok(Some(query_data)) => {
                // After migration logic in update_query_data_in_project, analysis_chat_history should be empty
                // if the file was modified. If it's a freshly loaded old file, it will still contain the vec.
                // We prioritize the new structure if it's populated.
                if !query_data.chat_nodes.is_empty() {
                    self.get_active_chat_branch(&query_data.chat_nodes, query_data.current_node_id)
                } else {
                    // Fallback for files that were *just loaded* and haven't triggered a save/migration yet.
                    // This scenario should be rare after the first modification/save.
                    query_data.analysis_chat_history
                }
            },
            _ => Vec::new(),
        }
    }

    /// Adds a chat message to the graph and sets it as the new current_node_id.
    /// Optionally accepts a parent_id to create a branch.
    pub fn add_chat_message(
        &self,
        query_manager: &QueryManager, // Dependency injection
        project_dir: &Path,
        mut message: ChatMessage, // Make message mutable to set its ID and parent_id
        query_filename: &str,
        parent_id_override: Option<Uuid>, // <--- NEW PARAMETER
    ) -> Result<Uuid, String> { // <--- RETURN Uuid
        query_manager.update_query_data_in_project(project_dir, query_filename, |qd| {
            // New messages always get a new ID, even if from an existing ChatMessage clone
            message.id = Uuid::new_v4();
            // Parent is either the override or the current head of the conversation
            message.parent_id = parent_id_override.or(qd.current_node_id);

            qd.chat_nodes.insert(message.id, message.clone()); // Insert the new message

            // When adding a new message, it becomes the new current_node_id
            // No need to find furthest descendant here, as it's by definition a leaf.
            qd.current_node_id = Some(message.id);
        })?;
        Ok(message.id) // Return the ID of the newly added message
    }

    // Update to use message_id (Uuid) instead of index
    pub fn update_message_in_history(
        &self,
        query_manager: &QueryManager, // Dependency injection
        project_dir: &Path,
        message_id: Uuid, // <--- CHANGED FROM usize
        updated_message_content: String, // Only content is typically updated for existing message
        query_filename: &str,
        // The original message fields other than content are usually retained upon update.
        // If other fields can be updated, they should be passed explicitly.
    ) -> Result<(), String> {
        query_manager.update_query_data_in_project(project_dir, query_filename, |qd| {
            if let Some(msg) = qd.chat_nodes.get_mut(&message_id) {
                // Only update content. Other fields like role, commit_hash etc. remain from original.
                // If you intend to update all fields, pass a full ChatMessage and replace.
                msg.content = updated_message_content;
                // You might want to update the timestamp here too, if the edit counts as a 'modification'
                msg.timestamp = Some(Utc::now());
            } else {
                eprintln!("Attempted to update message with ID {} but not found.", message_id);
            }
        })
    }

    // Update to use message_id (Uuid) instead of index
    pub fn update_message_visibility(
        &self,
        query_manager: &QueryManager, // Dependency injection
        project_dir: &Path,
        message_id: Uuid, // <--- CHANGED FROM usize
        hidden: bool,
        query_filename: &str,
    ) -> Result<(), String> {
        query_manager.update_query_data_in_project(project_dir, query_filename, |qd| {
            if let Some(msg) = qd.chat_nodes.get_mut(&message_id) {
                msg.hidden = hidden;
                msg.timestamp = Some(Utc::now()); // Update timestamp on visibility change
            } else {
                eprintln!("Attempted to update message visibility for ID {} but not found.", message_id);
            }
        })
    }

    // NEW: Helper function to find all direct children of a given node
    fn find_direct_children(&self, nodes: &HashMap<Uuid, ChatMessage>, parent_id: Uuid) -> Vec<Uuid> {
        nodes.values()
             .filter(|msg| msg.parent_id == Some(parent_id))
             .map(|msg| msg.id)
             .collect()
    }

    /// Finds the furthest linear descendant of a given node ID.
    /// A linear descendant is a child that is the only child of its parent.
    /// The function returns the ID of the leaf node in that linear path,
    /// or the original node ID if it has no children or branches immediately.
    pub fn find_furthest_linear_descendant(
        &self,
        nodes: &HashMap<Uuid, ChatMessage>,
        start_node_id: Uuid,
    ) -> Uuid {
        let mut current_id = start_node_id;
        loop {
            let children = self.find_direct_children(nodes, current_id);
            if children.len() == 1 {
                current_id = children[0]; // Continue down the linear path
            } else {
                // No children, or more than one child (branching point), so this is the furthest linear descendant
                return current_id;
            }
        }
    }

    // NEW: Set the current node to a specific message ID, effectively jumping to a branch
    pub fn set_current_node(
        &self,
        query_manager: &QueryManager,
        project_dir: &Path,
        query_filename: &str,
        requested_node_id: Uuid, // Renamed for clarity
    ) -> Result<(), String> {
        query_manager.update_query_data_in_project(project_dir, query_filename, |qd| {
            if qd.chat_nodes.contains_key(&requested_node_id) {
                // Find the furthest linear descendant of the requested node
                let new_current_leaf_id = self.find_furthest_linear_descendant(&qd.chat_nodes, requested_node_id);
                qd.current_node_id = Some(new_current_leaf_id);
            } else {
                eprintln!("Attempted to set current node to non-existent message ID: {}. Ignoring.", requested_node_id);
            }
        })
    }

    // Will handle logic from Project::reset_chat_history
    pub fn reset_chat_history(
        &self,
        query_manager: &QueryManager, // Dependency injection
        project_dir: &Path,
        query_filename: &str,
    ) -> Result<(), String> {
        query_manager.update_query_data_in_project(project_dir, query_filename, |qd| {
            qd.chat_nodes = HashMap::new(); // Clear all nodes
            qd.current_node_id = None; // Reset current pointer
        })
    }
}