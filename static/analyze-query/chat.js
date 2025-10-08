// static/analyze-query/chat.js
import { formatMessage, linkFilePathsInElement } from './utils.js'; // Import linkFilePathsInElement
import { applySyntaxHighlighting } from './syntax-highlighting.js';
import { initCodeBlockActions } from './code-block-actions.js';

export function sendMessage(chatContainer) {
    const messageInput = document.getElementById('analysis-message-input');
    const message = messageInput.value.trim();
    if (message) {
        addMessageToChat('user', message, chatContainer);
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
        .then(response => response.text())
        .then(async responseText => {
            chatContainer.querySelectorAll('.chat-message.model-message .regenerate-message-btn').forEach(btn => {
                btn.remove();
            });

            const messageIndex = chatContainer.children.length;

            const messageDiv = addMessageToChat('model', responseText, chatContainer, false, messageIndex);
            const messageControls = messageDiv.querySelector('.message-controls');
            const regenerateButton = createRegenerateButton();
            messageControls.appendChild(regenerateButton);

            await applySyntaxHighlighting(messageDiv);
            linkFilePathsInElement(messageDiv.querySelector('.message-content'));
            initCodeBlockActions(messageDiv);

            if (chatContainer && messageDiv) {
                chatContainer.scrollTop = chatContainer.scrollTop + 400;
            }
        })
        .catch(error => {
            console.error('Error:', error);
            addMessageToChat('model', 'Error: Could not get a response.', chatContainer);
            if (chatContainer) {
                chatContainer.scrollTop = chatContainer.scrollHeight;
            }
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
        const option = document.createElement('option');
        option.value = responseText;
        option.text = responseText.replace(".json", "");
        query_selct.appendChild(option);
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
    
    const messageContent = document.createElement('div');
    messageContent.className = 'message-content';
    messageContent.innerHTML = formatMessage(content); // Format Markdown here
    messageDiv.dataset.originalContent = content;
    
    const messageControls = document.createElement('div');
    messageControls.className = 'message-controls';
    
    const editButton = document.createElement('button');
    editButton.className = 'edit-message-btn';
    editButton.textContent = 'Edit';
    editButton.title = 'Edit message';
    editButton.addEventListener('click', () => toggleEditMode(messageDiv));
    
    messageControls.appendChild(editButton);

    const hideButton = document.createElement('button');
    hideButton.className = 'hide-message-btn';
    hideButton.textContent = hidden ? 'Unhide' : 'Hide';
    hideButton.title = hidden ? 'Unhide message' : 'Hide message';
    hideButton.dataset.hidden = hidden;
    hideButton.addEventListener('click', () => toggleHideMessage(messageDiv));
    
    messageControls.appendChild(hideButton);
    
    messageDiv.appendChild(messageContent);
    messageDiv.appendChild(messageControls);
    chatContainer.appendChild(messageDiv);
    
    // The actual linking will happen AFTER applySyntaxHighlighting in the .then() block of sendMessage
    // or for initial messages, directly in initAnalysisChat. This ensures order of operations.
    
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
    
    if (messageDiv.classList.contains('editing')) {
        const editor = messageDiv.querySelector('.message-editor');
        const editedContent = editor.value;
        
        messageDiv.dataset.originalContent = editedContent;
        
        messageContent.innerHTML = formatMessage(editedContent);
        
        messageDiv.classList.remove('editing');
        editor.remove();
        
        applySyntaxHighlighting(messageDiv);
        linkFilePathsInElement(messageContent); // Re-apply linking after edit
        
        saveEditedMessage(messageDiv, role, editedContent);
    } else {
        messageDiv.classList.add('editing');
        
        const originalContent = messageDiv.dataset.originalContent || messageContent.textContent;
        const editor = document.createElement('textarea');
        editor.className = 'message-editor';
        editor.value = originalContent;
        
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
            query_id: document.getElementById('query-id').value,
            commit_hash: 'retain'
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

     saveHiddenMessage(messageDiv, newHiddenState);
}

function saveHiddenMessage(messageDiv, hidden) {
    const projectName = document.getElementById('project-name').value;
    const chatContainer = messageDiv.parentElement;
    const messageIndex = Array.from(chatContainer.children).indexOf(messageDiv);
    const role = messageDiv.classList.contains('user-message') ? 'user' : 'model';

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
                index: messageIndex
            })
        });

        if (response.ok) {
            const newContent = await response.text();
            messageDiv.dataset.originalContent = newContent;
            messageContent.innerHTML = formatMessage(newContent);
            await applySyntaxHighlighting(messageDiv);
            linkFilePathsInElement(messageDiv.querySelector('.message-content'));
            initCodeBlockActions(messageDiv);
            if (chatContainer && messageDiv) {
                // Scroll to the top of the regenerated message
                chatContainer.scrollTop = chatContainer.scrollTop + 400;
            }
        } else {
            const errorText = await response.text();
            console.error('Error regenerating message:', errorText);
            messageContent.innerHTML = formatMessage(originalContent);
            alert('Failed to regenerate response. Please check server logs.');
        }
    } catch (error) {
        console.error('Network error during regeneration:', error);
        messageContent.innerHTML = formatMessage(originalContent);
        alert('A network error occurred during regeneration.');
    } finally {
        if (regenerateButton) {
            regenerateButton.disabled = false;
            regenerateButton.textContent = 'Regenerate';
        }
    }
}