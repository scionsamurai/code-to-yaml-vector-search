// static/analyze-query/chat.js
import { formatMessage, linkFilePathsInElement } from './utils.js';
import { applySyntaxHighlighting } from './syntax-highlighting.js';
import { initCodeBlockActions } from './code-block-actions.js';

/**
 * Scrolls the given chat container to the bottom.
 * @param {HTMLElement} chatContainer
 */
function scrollToBottom(chatContainer) {
    if (chatContainer) {
        chatContainer.scrollTop = chatContainer.scrollHeight;
    }
}

/**
 * Appends a message div to the chat, applies formatting and event listeners.
 * @param {string} role 'user' or 'model'
 * @param {string} content Raw markdown content
 * @param {HTMLElement} chatContainer
 * @param {boolean} hidden Whether the message is initially hidden
 * @param {string} messageId UUID of the message
 * @returns {HTMLElement} The created message div
 */
export function addMessageToChat(role, content, chatContainer, hidden = false, messageId = null) {
    const messageDiv = document.createElement('div');
    messageDiv.className = `chat-message ${role}-message`;
    if (messageId) {
        messageDiv.dataset.messageId = messageId; // Store the UUID
    }

    const messageContent = document.createElement('div');
    messageContent.className = 'message-content';
    messageContent.innerHTML = formatMessage(content); // Format Markdown here
    messageDiv.dataset.originalContent = content; // Always store original content for editing

    const messageControls = document.createElement('div');
    messageControls.className = 'message-controls';

    const editButton = document.createElement('button');
    editButton.className = 'edit-message-btn';
    editButton.textContent = 'Edit';
    editButton.title = 'Edit message';
    editButton.addEventListener('click', (event) => toggleEditMode(event.target.closest('.chat-message')));
    messageControls.appendChild(editButton);

    const hideButton = document.createElement('button');
    hideButton.className = 'hide-message-btn';
    hideButton.textContent = hidden ? 'Unhide' : 'Hide';
    hideButton.title = hidden ? 'Unhide message' : 'Hide message';
    hideButton.dataset.hidden = hidden; // Store hidden state
    hideButton.addEventListener('click', (event) => toggleHideMessage(event.target.closest('.chat-message')));
    messageControls.appendChild(hideButton);

    // Only add regenerate button for model messages
    if (role === 'model') {
        const regenerateButton = createRegenerateButton();
        messageControls.appendChild(regenerateButton);
    }

    messageDiv.appendChild(messageContent);
    messageDiv.appendChild(messageControls);
    chatContainer.appendChild(messageDiv);

    // Apply post-processing
    applySyntaxHighlighting(messageDiv);
    linkFilePathsInElement(messageContent);
    initCodeBlockActions(messageDiv); // Init actions for new code blocks

    return messageDiv;
}

/**
 * Updates an existing message div's content and re-applies formatting/actions.
 * @param {HTMLElement} messageDiv The target message div
 * @param {string} newContent Raw markdown content
 */
function updateMessageInChat(messageDiv, newContent) {
    const messageContent = messageDiv.querySelector('.message-content');
    messageContent.innerHTML = formatMessage(newContent);
    messageDiv.dataset.originalContent = newContent; // Update original content for future edits

    applySyntaxHighlighting(messageDiv);
    linkFilePathsInElement(messageContent);
    initCodeBlockActions(messageDiv);
}

// NOTE: The `replaceMessageWithNew` function has been removed as it was not used and the new
// dynamic update logic for regeneration and branching on edit directly handles message insertion.


export async function sendMessage(chatContainer) {
    const messageInput = document.getElementById('analysis-message-input');
    const message = messageInput.value.trim();
    if (message) {
        // Add user message to chat immediately for UX. messageId is temporary for now.
        const tempUserMessageDiv = addMessageToChat('user', message, chatContainer);
        // Clear input and scroll before API call
        messageInput.value = '';
        scrollToBottom(chatContainer);

        const projectName = document.getElementById('project-name').value;
        const queryId = document.getElementById('query-id').value;

        try {
            const response = await fetch('/chat-analysis', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    project: projectName,
                    message: message,
                    query_id: queryId
                })
            });

            if (!response.ok) {
                const errorText = await response.text();
                throw new Error(errorText);
            }

            const data = await response.json();

            if (data.success) {
                // Update the temporary user message with its actual ID from backend
                tempUserMessageDiv.dataset.messageId = data.user_message.id;
                // Update its content in case backend processed it or for consistency
                updateMessageInChat(tempUserMessageDiv, data.user_message.content);

                // Add model message
                const modelMessageDiv = addMessageToChat(
                    data.model_message.role,
                    data.model_message.content,
                    chatContainer,
                    data.model_message.hidden,
                    data.model_message.id
                );
                scrollToBottom(chatContainer);

                // Optionally, update the branch UI here if a specific message's children count changed
                // This would involve finding the parent of the user message (which is `current_node_id` before this interaction)
                // and re-rendering its branch navigation controls. This is complex, will defer for now.

            } else {
                addMessageToChat('model', `Error: ${data.message || 'Unknown error during chat analysis.'}`, chatContainer);
                scrollToBottom(chatContainer);
            }
        } catch (error) {
            console.error('Error:', error);
            addMessageToChat('model', `Error: Could not get a response. ${error.message}`, chatContainer);
            scrollToBottom(chatContainer);
        } finally {
            // Re-enable input if needed or other final UI touches
            // If the temp message wasn't replaced, clean it up or keep for error.
            // Current approach keeps it and just adds a model error below it.
        }
    }
}

export async function resetChat(chatContainer) {
    // Clear chat display immediately for UX
    while (chatContainer.firstChild) {
        chatContainer.removeChild(chatContainer.firstChild);
    }

    const projectName = document.getElementById('project-name').value;
    const queryId = document.getElementById('query-id').value;

    try {
        const response = await fetch('/reset-analysis-chat', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                project: projectName,
                query_id: queryId
            })
        });

        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText);
        }

        const data = await response.json();

        if (data.success) {
            // Populate chat with new history
            data.initial_chat_history.forEach(msg => {
                addMessageToChat(msg.role, msg.content, chatContainer, msg.hidden, msg.id);
            });
            scrollToBottom(chatContainer);
            // Re-render query data / page for the new current_node_id and potential branch changes
            // For full UI context update, a reload is simpler for now, similar to query selection.
            // This is a trade-off. If only chat needs update, it's fine.
            // If other UI elements depend on `current_node_id` this would need a partial reload or JS to update everything.
            // Keeping reload here for full context reset due to complexity of re-rendering full page state (branch UI, etc.).
            location.reload();
        } else {
            addMessageToChat('model', `Error resetting chat: ${data.message || 'Unknown error.'}`, chatContainer);
        }
    } catch (error) {
        console.error('Error resetting chat:', error);
        addMessageToChat('model', `Error resetting chat: ${error.message}`, chatContainer);
    } finally {
        scrollToBottom(chatContainer);
    }
}

function createRegenerateButton() {
    const regenerateButton = document.createElement('button');
    regenerateButton.className = 'regenerate-message-btn';
    regenerateButton.textContent = 'Regenerate';
    regenerateButton.title = 'Regenerate response';
    regenerateButton.addEventListener('click', (event) => {
        const messageDiv = event.target.closest('.chat-message');
        regenerateMessage(messageDiv);
    });
    return regenerateButton;
}


export function toggleEditMode(messageDiv) {
    const messageContent = messageDiv.querySelector('.message-content');
    const messageId = messageDiv.dataset.messageId;

    if (!messageId) {
        console.error("Attempted to edit message without a message ID.");
        return;
    }

    if (messageDiv.classList.contains('editing')) {
        const editor = messageDiv.querySelector('.message-editor');
        const editedContent = editor.value;
        const createNewBranch = messageDiv.querySelector('.create-branch-on-edit-checkbox')?.checked || false;

        // Temporarily show loading state
        messageContent.innerHTML = '<em>Saving changes...</em>';

        messageDiv.classList.remove('editing');
        if (editor) editor.remove();
        if (messageDiv.querySelector('.edit-controls')) messageDiv.querySelector('.edit-controls').remove();

        saveEditedMessage(messageId, editedContent, createNewBranch, messageDiv);
    } else {
        messageDiv.classList.add('editing');

        const originalContent = messageDiv.dataset.originalContent || messageContent.textContent;
        const editor = document.createElement('textarea');
        editor.className = 'message-editor text-area-fmt';
        editor.value = originalContent;

        const editControls = document.createElement('div');
        editControls.className = 'edit-controls';

        const saveButton = document.createElement('button');
        saveButton.className = 'save-edit-btn primary';
        saveButton.textContent = 'Save';
        saveButton.addEventListener('click', () => toggleEditMode(messageDiv));

        const cancelButton = document.createElement('button');
        cancelButton.className = 'cancel-edit-btn secondary';
        cancelButton.textContent = 'Cancel';
        cancelButton.addEventListener('click', () => {
            messageDiv.classList.remove('editing');
            editor.remove();
            editControls.remove();
            // Re-render original content if editing was cancelled after changes but before save
            updateMessageInChat(messageDiv, originalContent);
        });

        const createBranchCheckbox = document.createElement('input');
        createBranchCheckbox.type = 'checkbox';
        createBranchCheckbox.id = `create-branch-on-edit-${messageId}`;
        createBranchCheckbox.className = 'create-branch-on-edit-checkbox';
        const createBranchLabel = document.createElement('label');
        createBranchLabel.htmlFor = `create-branch-on-edit-${messageId}`;
        createBranchLabel.textContent = ' Create new branch on edit';
        const createBranchContainer = document.createElement('div');
        createBranchContainer.className = 'create-branch-checkbox-container';
        createBranchContainer.appendChild(createBranchCheckbox);
        createBranchContainer.appendChild(createBranchLabel);

        messageDiv.insertBefore(editor, messageContent.nextSibling);
        messageDiv.insertBefore(editControls, editor.nextSibling);
        editControls.appendChild(saveButton);
        editControls.appendChild(cancelButton);
        editControls.appendChild(createBranchContainer); // Add the checkbox
    }
}

async function saveEditedMessage(messageId, content, createNewBranch, originalMessageDiv) {
    const projectName = document.getElementById('project-name').value;
    const chatContainer = originalMessageDiv.parentElement;

    try {
        const response = await fetch('/update-chat-message', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                project: projectName,
                content: content,
                message_id: messageId,
                query_id: document.getElementById('query-id').value,
                create_new_branch: createNewBranch
            })
        });

        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText);
        }

        const data = await response.json();

        if (data.success) {
            if (createNewBranch) {
                // Remove the original message from the DOM
                originalMessageDiv.remove();

                // Find the parent message div for insertion
                // The backend sends back `parent_message_id`. If null, it's a root message.
                const parentMessageDiv = data.parent_message_id
                    ? chatContainer.querySelector(`.chat-message[data-message-id="${data.parent_message_id}"]`)
                    : null;

                const newMessageDiv = addMessageToChat(
                    data.message.role,
                    data.message.content,
                    chatContainer, // Will append to end initially
                    data.message.hidden,
                    data.message.id
                );

                if (parentMessageDiv) {
                    parentMessageDiv.insertAdjacentElement('afterend', newMessageDiv);
                }
                // If no parent, it was a root message or appended to end by addMessageToChat, which is fine.

                alert('Message edited and new branch created successfully. Note: Branch navigation controls may require a page reload to update visually.');
            } else {
                // In-place update
                updateMessageInChat(originalMessageDiv, data.message.content);
            }
            scrollToBottom(chatContainer);
        } else {
            alert(`Failed to save edited message: ${data.message || 'Unknown error.'}`);
            // Revert UI if an error occurred
            updateMessageInChat(originalMessageDiv, originalMessageDiv.dataset.originalContent); // Revert to previous content
        }
    } catch (error) {
        console.error('Error saving edited message:', error);
        alert(`Failed to save edited message: ${error.message}.`);
        // Revert UI if a network error occurred
        updateMessageInChat(originalMessageDiv, originalMessageDiv.dataset.originalContent); // Revert to previous content
    } finally {
        originalMessageDiv.classList.remove('editing'); // Ensure edit mode is off
    }
}


export async function toggleHideMessage(messageDiv) {
    const hideButton = messageDiv.querySelector('.hide-message-btn');
    const messageId = messageDiv.dataset.messageId;

    if (!messageId) {
        console.error("Attempted to toggle visibility for message without a message ID.");
        return;
    }

    const hidden = hideButton.dataset.hidden === 'true';
    const newHiddenState = !hidden;

    hideButton.textContent = newHiddenState ? 'Unhide' : 'Hide';
    hideButton.title = newHiddenState ? 'Unhide message' : 'Hide message';
    hideButton.dataset.hidden = newHiddenState;

    saveHiddenMessage(messageId, newHiddenState);
}

// Updated to accept messageId
async function saveHiddenMessage(messageId, hidden) {
    const projectName = document.getElementById('project-name').value;

    try {
        const response = await fetch('/update-message-visibility', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                project: projectName,
                message_id: messageId,
                query_id: document.getElementById('query-id').value,
                hidden: hidden
            })
        });

        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText);
        }
        // No UI update needed beyond button text, as visibility changes primarily affect LLM context,
        // which would require a regeneration or new message to become apparent.
        // If a message should visually be hidden/shown, that logic would go here.
        // For now, it's just updating the button and backend state.
        console.log(`Message ${messageId} visibility updated to hidden=${hidden}.`);
    } catch (error) {
        console.error('Error saving hidden message:', error);
        alert(`Failed to update message visibility: ${error.message}`);
        // Revert UI if error (e.g., button text)
        const messageDiv = document.querySelector(`.chat-message[data-message-id="${messageId}"]`);
        if (messageDiv) {
            const hideButton = messageDiv.querySelector('.hide-message-btn');
            hideButton.dataset.hidden = (!hidden).toString();
            hideButton.textContent = (!hidden) ? 'Unhide' : 'Hide';
            hideButton.title = (!hidden) ? 'Unhide message' : 'Hide message';
        }
    }
}

export async function regenerateMessage(messageDiv) {
    const chatContainer = messageDiv.parentElement;
    const projectName = document.getElementById('project-name').value;
    const queryId = document.getElementById('query-id').value; // This is the analysis query ID, not chat message ID
    const messageIdToRegenerate = messageDiv.dataset.messageId; // ID of the MODEL message being regenerated FROM

    if (!messageIdToRegenerate) {
        console.error("Attempted to regenerate message without a message ID.");
        return;
    }

    const regenerateButton = messageDiv.querySelector('.regenerate-message-btn');
    if (regenerateButton) {
        regenerateButton.disabled = true;
        regenerateButton.textContent = 'Regenerating...';
    }
    const messageContent = messageDiv.querySelector('.message-content');
    const originalContent = messageDiv.dataset.originalContent;
    messageContent.innerHTML = '<em>Regenerating response...</em>'; // Show loading state

    try {
        const response = await fetch('/regenerate-chat-message', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                project: projectName,
                query_id: queryId,
                message_id: messageIdToRegenerate // Send the ID of the model message to regenerate from
            })
        });

        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText);
        }

        const data = await response.json();

        if (data.success) {
            // Remove the old model message from the DOM
            messageDiv.remove();

            // Find the parent user message div for correct insertion point
            const userMessageDiv = chatContainer.querySelector(`.chat-message[data-message-id="${data.user_message_id}"]`);

            if (userMessageDiv) {
                // Insert the new model message immediately after its parent user message.
                const newModelMessageDiv = addMessageToChat(
                    data.new_model_message.role,
                    data.new_model_message.content,
                    chatContainer, // Temporarily append to chatContainer
                    data.new_model_message.hidden,
                    data.new_model_message.id
                );
                // Move the newly added message to the correct position
                userMessageDiv.insertAdjacentElement('afterend', newModelMessageDiv);

            } else {
                // Fallback if parent user message is not found (shouldn't happen if history is consistent)
                console.warn(`Parent user message with ID ${data.user_message_id} not found for regenerated message. Appending to end.`);
                // If it was already appended by `addMessageToChat`, it's at the end, which is an acceptable fallback.
            }


        } else {
            messageContent.innerHTML = formatMessage(originalContent); // Revert on error
            alert(`Failed to regenerate response: ${data.message || 'Unknown error.'}`);
        }
    } catch (error) {
        console.error('Network error during regeneration:', error);
        messageContent.innerHTML = formatMessage(originalContent); // Revert on network error
        alert(`A network error occurred during regeneration: ${error.message}.`);
    } finally {
        if (regenerateButton) {
            regenerateButton.disabled = false;
            regenerateButton.textContent = 'Regenerate';
        }
        scrollToBottom(chatContainer);
    }
}