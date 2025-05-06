// static/analyze-query-chat.js

import { formatMessage } from './utils.js';

export function sendMessage(chatContainer) {
    const messageInput = document.getElementById('analysis-message-input');
    const message = messageInput.value.trim();
    if (message) {
        addMessageToChat('user', message, chatContainer);
        messageInput.value = '';

        const projectName = document.getElementById('project-name').value;
        const queryText = document.getElementById('query-text').value;

        const chatHistory = getChatHistory();

        fetch('/chat-analysis', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                project: projectName,
                query: queryText,
                message: message,
                history: chatHistory
            })
        })
        .then(response => response.text())
        .then(responseText => {
            addMessageToChat('model', responseText, chatContainer);
            chatContainer.scrollTop = chatContainer.scrollHeight;
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

function getChatHistory() {
    const chatMessages = document.querySelectorAll('#analysis-chat-container .chat-message');
    let history = [];

    chatMessages.forEach(message => {
        if (message.classList.contains('system-message')) return; // Skip system messages

        const role = message.classList.contains('user-message') ? 'user' : 'model';
        const content = message.querySelector('.message-content').textContent;
        history.push({ role, content });
    });

    return history;
}

function addMessageToChat(role, content, chatContainer) {
    const messageDiv = document.createElement('div');
    messageDiv.className = `chat-message ${role}-message`;

    const messageContent = document.createElement('div');
    messageContent.className = 'message-content';
    messageContent.innerHTML = formatMessage(content);

    messageDiv.appendChild(messageContent);
    chatContainer.appendChild(messageDiv);
    chatContainer.scrollTop = chatContainer.scrollHeight;
}