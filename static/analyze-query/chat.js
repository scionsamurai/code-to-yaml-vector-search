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

        fetch('/chat-analysis', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                project: projectName,
                message: message
            })
        })
        .then(response => response.text())
        .then(async responseText => {
            const messageDiv = addMessageToChat('model', responseText, chatContainer);
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

    fetch('/reset-analysis-chat', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            project: projectName
        })
    })
    .catch(error => {
        console.error('Error resetting chat:', error);
    });
}

export function addMessageToChat(role, content, chatContainer) {
    const messageDiv = document.createElement('div');
    messageDiv.className = `chat-message ${role}-message`;
    
    // Create message content div
    const messageContent = document.createElement('div');
    messageContent.className = 'message-content';
    messageContent.innerHTML = formatMessage(content);
    messageDiv.dataset.originalContent = content;
    
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
    
    messageDiv.appendChild(messageContent);
    messageDiv.appendChild(messageControls);
    chatContainer.appendChild(messageDiv);
    
    return messageDiv;
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
            index: messageIndex 
        })
    })
    .catch(error => {
        console.error('Error saving edited message:', error);
    });
}