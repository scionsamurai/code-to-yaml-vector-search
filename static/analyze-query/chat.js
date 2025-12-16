// static/analyze-query/chat.js
import { formatMessage, linkFilePathsInElement } from './utils.js';
import { applySyntaxHighlighting } from './syntax-highlighting.js';
import { initCodeBlockActions } from './code-block-actions.js';

export function sendMessage(chatContainer) {
    const messageInput = document.getElementById('analysis-message-input');
    const message = messageInput.value.trim();
    if (message) {
        // Add user message to chat. We don't have an ID yet, so messageId is null.
        const userMessageDiv = addMessageToChat('user', message, chatContainer); // Capture the created div

        messageInput.value = '';

        if (chatContainer) {
            chatContainer.scrollTop = chatContainer.scrollHeight;
        }

        const projectName = document.getElementById('project-name').value;
        const queryId = document.getElementById('query-id').value;

        fetch('/chat-analysis', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                project: projectName,
                message: message,
                query_id: queryId
            })
        })
        .then(response => {
            if (!response.ok) {
                return response.text().then(text => { throw new Error(text); });
            }
            // Expect JSON object with user_message_id, model_message_id, and model_content
            return response.json();
        })
        .then(async data => {
            // Update the user message div with its actual ID from the backend
            if (data.user_message_id) {
                userMessageDiv.dataset.messageId = data.user_message_id;
            }

            // --- No longer dynamically adding/removing regenerate buttons or branch navigators here ---
            // --- The entire chat will reload to reflect the new state, including branch navigators. ---
            location.reload(); // Full reload to ensure correct branch state and button placement
        })
        .catch(error => {
            console.error('Error:', error);
            addMessageToChat('model', `Error: Could not get a response. ${error.message}`, chatContainer);
            if (chatContainer) {
                chatContainer.scrollTop = chatContainer.scrollHeight;
            }
        });
    }
}

export function resetChat(chatContainer) {
    // Clear chat display immediately for UX
    while (chatContainer.firstChild) {
        chatContainer.removeChild(chatContainer.firstChild);
    }

    const projectName = document.getElementById('project-name').value;
    const queryIdInput = document.getElementById('query-id');

    fetch('/reset-analysis-chat', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            project: projectName,
            query_id: queryIdInput.value
        })
    })
    .then(response => response.text()) // Expect the new query_id (filename) as text
    .then(async responseText => {
        console.log('Chat reset successfully:', responseText);
        const querySelector = document.getElementById('query-selector');
        const newQueryId = responseText;

        // Clear existing options in the query selector
        querySelector.innerHTML = '';
        // Add the new query_id as an option
        const option = document.createElement('option');
        option.value = newQueryId;
        option.text = newQueryId.replace(".json", ""); // Display without .json
        querySelector.appendChild(option);
        querySelector.value = newQueryId; // Select the new query

        queryIdInput.value = newQueryId; // Update the hidden input with the new query ID
        location.reload(); // Reload to reflect the reset chat and new query_id
    })
    .catch(error => {
        console.error('Error resetting chat:', error);
        addMessageToChat('model', `Error resetting chat: ${error.message}`, chatContainer);
    });
}

// Updated to accept an optional messageId
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
    // Event listener now directly on button, retrieves ID from parent
    editButton.addEventListener('click', (event) => toggleEditMode(event.target.closest('.chat-message')));

    messageControls.appendChild(editButton);

    const hideButton = document.createElement('button');
    hideButton.className = 'hide-message-btn';
    hideButton.textContent = hidden ? 'Unhide' : 'Hide';
    hideButton.title = hidden ? 'Unhide message' : 'Hide message';
    hideButton.dataset.hidden = hidden; // Store hidden state
    // Event listener now directly on button, retrieves ID from parent
    hideButton.addEventListener('click', (event) => toggleHideMessage(event.target.closest('.chat-message')));

    messageControls.appendChild(hideButton);

    messageDiv.appendChild(messageContent);
    messageDiv.appendChild(messageControls);
    chatContainer.appendChild(messageDiv);

    // The actual linking will happen AFTER applySyntaxHighlighting in the .then() block of sendMessage
    // or for initial messages, directly in initAnalysisChat. This ensures order of operations.

    return messageDiv; // <-- Ensure this returns the messageDiv
}

function createRegenerateButton() {
    const regenerateButton = document.createElement('button');
    regenerateButton.className = 'regenerate-message-btn';
    regenerateButton.textContent = 'Regenerate';
    regenerateButton.title = 'Regenerate response';
    regenerateButton.addEventListener('click', (event) => {
        const messageDiv = event.target.closest('.chat-message');
        regenerateMessage(messageDiv); // Changed name to be more general if we allow regenerating any message
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

        messageDiv.dataset.originalContent = editedContent;

        // Temporarily clear content to show "Saving..." or similar if desired,
        // but for now we'll just re-render after saving

        messageDiv.classList.remove('editing');
        if (editor) editor.remove();
        if (messageDiv.querySelector('.edit-controls')) messageDiv.querySelector('.edit-controls').remove();

        saveEditedMessage(messageId, editedContent, createNewBranch, messageDiv); // Pass messageDiv to handle updates
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
            messageContent.innerHTML = formatMessage(originalContent);
            applySyntaxHighlighting(messageDiv);
            linkFilePathsInElement(messageContent);
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

        editControls.appendChild(saveButton);
        editControls.appendChild(cancelButton);
        editControls.appendChild(createBranchContainer); // Add the checkbox

        messageDiv.insertBefore(editor, messageContent.nextSibling);
        messageDiv.insertBefore(editControls, editor.nextSibling);
    }
}

// Updated to accept messageId, content, createNewBranch flag, and messageDiv for UI update
function saveEditedMessage(messageId, content, createNewBranch, originalMessageDiv) {
    const projectName = document.getElementById('project-name').value;
    const chatContainer = originalMessageDiv.parentElement;

    fetch('/update-chat-message', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            project: projectName,
            content: content,
            message_id: messageId,
            query_id: document.getElementById('query-id').value,
            create_new_branch: createNewBranch // Pass the new flag
        })
    })
    .then(response => {
        if (!response.ok) {
            return response.text().then(text => { throw new Error(text); });
        }
        return response.json(); // Expect JSON with message_id and content
    })
    .then(async data => {
        // --- Full reload to reflect the new state, including branch navigators. ---
        location.reload();
    })
    .catch(error => {
        console.error('Error saving edited message:', error);
        alert(`Failed to save edited message: ${error.message}.`);
        // Re-enable edit mode or revert UI if an error occurred
        originalMessageDiv.classList.add('editing');
        const messageContent = originalMessageDiv.querySelector('.message-content');
        messageContent.innerHTML = formatMessage(originalMessageDiv.dataset.originalContent); // Revert to previous content
        applySyntaxHighlighting(originalMessageDiv);
        linkFilePathsInElement(messageContent);
    });
}


export function toggleHideMessage(messageDiv) {
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
function saveHiddenMessage(messageId, hidden) {
    const projectName = document.getElementById('project-name').value;

    fetch('/update-message-visibility', {
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
    })
    .catch(error => {
        console.error('Error saving hidden message:', error);
    });
}

// Renamed from regenerateLastMessage to regenerateMessage to be more general
export async function regenerateMessage(messageDiv) {
    const chatContainer = messageDiv.parentElement;
    const projectName = document.getElementById('project-name').value;
    const queryId = document.getElementById('query-id').value;
    const messageId = messageDiv.dataset.messageId; // Retrieve messageId of the MODEL message

    if (!messageId) {
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
    messageContent.innerHTML = '<em>Regenerating response...</em>';

    try {
        const response = await fetch('/regenerate-chat-message', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                project: projectName,
                query_id: queryId,
                message_id: messageId // Pass the ID of the MODEL message to regenerate from
            })
        });

        if (response.ok) {
            const data = await response.json(); // Expect JSON with message_id and content
            // --- Full reload to reflect the new state, including branch navigators. ---
            location.reload();
        } else {
            const errorText = await response.text();
            console.error('Error regenerating message:', errorText);
            messageContent.innerHTML = formatMessage(originalContent);
            alert(`Failed to regenerate response: ${errorText}. Please check server logs.`);
        }
    } catch (error) {
        console.error('Network error during regeneration:', error);
        messageContent.innerHTML = formatMessage(originalContent);
        alert(`A network error occurred during regeneration: ${error.message}.`);
    } finally {
        if (regenerateButton) {
            regenerateButton.disabled = false;
            regenerateButton.textContent = 'Regenerate';
        }
    }
}