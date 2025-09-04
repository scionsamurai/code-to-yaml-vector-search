// static/analyze-query/chat.js
import { formatMessage } from './utils.js';
import { applySyntaxHighlighting, updateCopyLinks } from './syntax-highlighting.js';

export function sendMessage(chatContainer) {
    const messageInput = document.getElementById('analysis-message-input');
    const message = messageInput.value.trim();
    if (message) {
        addMessageToChat('user', message, chatContainer);
        messageInput.value = '';

        const projectName = document.getElementById('project-name').value;
        const queryId = document.getElementById('query-id').value; // Get query_id

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
        .then(response => response.text())
        .then(async responseText => {
            // --- FIX START: Ensure only the latest model message has a regenerate button ---
            // Remove ALL regenerate buttons from ALL model messages first
            chatContainer.querySelectorAll('.chat-message.model-message .regenerate-message-btn').forEach(btn => {
                btn.remove();
            });
            // --- FIX END ---

            // Get the current number of messages to set the data-message-index
            const messageIndex = chatContainer.children.length;

            const messageDiv = addMessageToChat('model', responseText, chatContainer, false, messageIndex);
            // Add regenerate button to the newly added model message
            const messageControls = messageDiv.querySelector('.message-controls');
            const regenerateButton = createRegenerateButton();
            messageControls.appendChild(regenerateButton);

            await applySyntaxHighlighting(messageDiv);
            updateCopyLinks(messageDiv);
        })
        .catch(error => {
            console.error('Error:', error);
            addMessageToChat('model', 'Error: Could not get a response.', chatContainer);
        });
    }
}

export function resetChat(chatContainer) {
    while (chatContainer.firstChild) {
        chatContainer.removeChild(chatContainer.firstChild);
    }

    const projectName = document.getElementById('project-name').value;
    const queryId = document.getElementById('query-id');

    fetch('/reset-analysis-chat', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            project: projectName,
            query_id: queryId.value
        })
    })
    .then(response => response.text())
    .then(async responseText => {
        console.log('Chat reset successfully:', responseText);
        const query_selct = document.getElementById('query-selector');
        // add option to select html tag
        const option = document.createElement('option');
        option.value = responseText;
        option.text = responseText.replace(".json", "");
        query_selct.appendChild(option);
        // set the query-selectors to equal new option
        query_selct.value = responseText;
        queryId.value = responseText;
        
    })
    .catch(error => {
        console.error('Error resetting chat:', error);
    });
}

export function addMessageToChat(role, content, chatContainer, hidden = false, messageIndex = -1) {
    const messageDiv = document.createElement('div');
    messageDiv.className = `chat-message ${role}-message`;
    if (messageIndex !== -1) {
        messageDiv.dataset.messageIndex = messageIndex;
    }
    
    // Create message content div
    const messageContent = document.createElement('div');
    messageContent.className = 'message-content';
    messageContent.innerHTML = formatMessage(content);
    messageDiv.dataset.originalContent = content; // Store original raw content
    
    // Create message controls
    const messageControls = document.createElement('div');
    messageControls.className = 'message-controls';
    
    // Add edit button
    const editButton = document.createElement('button');
    editButton.className = 'edit-message-btn';
    editButton.textContent = 'Edit';
    editButton.title = 'Edit message';
    editButton.addEventListener('click', () => toggleEditMode(messageDiv));
    
    messageControls.appendChild(editButton);

    // Add hide button
    const hideButton = document.createElement('button');
    hideButton.className = 'hide-message-btn';
    hideButton.textContent = hidden ? 'Unhide' : 'Hide';
    hideButton.title = hidden ? 'Unhide message' : 'Hide message';
    hideButton.dataset.hidden = hidden; // Store the hidden state on the button
    hideButton.addEventListener('click', () => toggleHideMessage(messageDiv));
    
    messageControls.appendChild(hideButton);
    
    messageDiv.appendChild(messageContent);
    messageDiv.appendChild(messageControls);
    chatContainer.appendChild(messageDiv);
    
    return messageDiv;
}

function createRegenerateButton() {
    const regenerateButton = document.createElement('button');
    regenerateButton.className = 'regenerate-message-btn';
    regenerateButton.textContent = 'Regenerate';
    regenerateButton.title = 'Regenerate response';
    regenerateButton.addEventListener('click', (event) => {
        const messageDiv = event.target.closest('.chat-message');
        regenerateLastMessage(messageDiv);
    });
    return regenerateButton;
}


export function toggleEditMode(messageDiv) {
    const messageContent = messageDiv.querySelector('.message-content');
    const role = messageDiv.classList.contains('user-message') ? 'user' : 'model';
    
    // If already in edit mode, exit it
    if (messageDiv.classList.contains('editing')) {
        const editor = messageDiv.querySelector('.message-editor');
        const editedContent = editor.value;
        
        // Store the updated content
        messageDiv.dataset.originalContent = editedContent;
        
        // Update message content
        messageContent.innerHTML = formatMessage(editedContent);
        
        // Exit edit mode
        messageDiv.classList.remove('editing');
        editor.remove();
        
        applySyntaxHighlighting(messageDiv);
        
        // Save the edited message to the server
        saveEditedMessage(messageDiv, role, editedContent);
    } else {
        // Enter edit mode
        messageDiv.classList.add('editing');
        
        // Get original content from data attribute instead of HTML
        const originalContent = messageDiv.dataset.originalContent || messageContent.textContent;
        const editor = document.createElement('textarea');
        editor.className = 'message-editor';
        editor.value = originalContent;
        
        // Add controls for saving/canceling
        const editControls = document.createElement('div');
        editControls.className = 'edit-controls';
        
        const saveButton = document.createElement('button');
        saveButton.className = 'save-edit-btn';
        saveButton.textContent = 'Save';
        saveButton.addEventListener('click', () => toggleEditMode(messageDiv));
        
        const cancelButton = document.createElement('button');
        cancelButton.className = 'cancel-edit-btn';
        cancelButton.textContent = 'Cancel';
        cancelButton.addEventListener('click', () => {
            messageDiv.classList.remove('editing');
            editor.remove();
            editControls.remove();
        });
        
        editControls.appendChild(saveButton);
        editControls.appendChild(cancelButton);
        
        // Insert editor and controls
        messageDiv.insertBefore(editor, messageContent.nextSibling);
        messageDiv.insertBefore(editControls, editor.nextSibling);
    }
}

function saveEditedMessage(messageDiv, role, content) {
    const projectName = document.getElementById('project-name').value;
    const chatContainer = messageDiv.parentElement;
    const messageIndex = Array.from(chatContainer.children).indexOf(messageDiv);
    
    fetch('/update-chat-message', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            project: projectName,
            role: role,
            content: content,
            index: messageIndex,
            query_id: document.getElementById('query-id').value
        })
    })
    .catch(error => {
        console.error('Error saving edited message:', error);
    });
}

export function toggleHideMessage(messageDiv) {
    const hideButton = messageDiv.querySelector('.hide-message-btn');
    const hidden = hideButton.dataset.hidden === 'true';
    const newHiddenState = !hidden;

    hideButton.textContent = newHiddenState ? 'Unhide' : 'Hide';
    hideButton.title = newHiddenState ? 'Unhide message' : 'Hide message';
    hideButton.dataset.hidden = newHiddenState;

     // Save the hidden state to the server
     saveHiddenMessage(messageDiv, newHiddenState);
}

function saveHiddenMessage(messageDiv, hidden) {
    const projectName = document.getElementById('project-name').value;
    const chatContainer = messageDiv.parentElement;
    const messageIndex = Array.from(chatContainer.children).indexOf(messageDiv);
    const role = messageDiv.classList.contains('user-message') ? 'user' : 'model';
    // No need to send content for visibility update, but originalContent is stored in dataset
    // const content = messageDiv.dataset.originalContent; 

    fetch('/update-message-visibility', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            project: projectName,
            index: messageIndex,
            query_id: document.getElementById('query-id').value,
            hidden: hidden
        })
    })
    .catch(error => {
        console.error('Error saving hidden message:', error);
    });
}

export async function regenerateLastMessage(messageDiv) {
    const chatContainer = messageDiv.parentElement;
    const projectName = document.getElementById('project-name').value;
    const queryId = document.getElementById('query-id').value;
    const messageIndex = parseInt(messageDiv.dataset.messageIndex, 10);

    // Disable buttons and show loading indicator
    const regenerateButton = messageDiv.querySelector('.regenerate-message-btn');
    if (regenerateButton) {
        regenerateButton.disabled = true;
        regenerateButton.textContent = 'Regenerating...';
    }
    const messageContent = messageDiv.querySelector('.message-content');
    const originalContent = messageDiv.dataset.originalContent; // Store original content from dataset
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
                index: messageIndex // Index of the message to regenerate
            })
        });

        if (response.ok) {
            const newContent = await response.text();
            messageDiv.dataset.originalContent = newContent; // Update original raw content
            messageContent.innerHTML = formatMessage(newContent); // Format and update display
            await applySyntaxHighlighting(messageDiv); // Re-apply highlighting
            updateCopyLinks(messageDiv); // Re-apply copy links
        } else {
            const errorText = await response.text();
            console.error('Error regenerating message:', errorText);
            messageContent.innerHTML = formatMessage(originalContent); // Revert on error, reformat original
            alert('Failed to regenerate response. Please check server logs.');
        }
    } catch (error) {
        console.error('Network error during regeneration:', error);
        messageContent.innerHTML = formatMessage(originalContent); // Revert on error, reformat original
        alert('A network error occurred during regeneration.');
    } finally {
        if (regenerateButton) {
            regenerateButton.disabled = false;
            regenerateButton.textContent = 'Regenerate';
        }
    }
}