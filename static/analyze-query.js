// static/analyze-query.js
function initAnalysisChat() {
    const chatContainer = document.getElementById('analysis-chat-container');
    const messageInput = document.getElementById('analysis-message-input');
    const sendButton = document.getElementById('analysis-send-button');
    const resetButton = document.getElementById('analysis-reset-button');
    
    // Add event listener for the send button
    if (sendButton) {
        sendButton.addEventListener('click', sendMessage);
    }
    
    // Add event listener for the reset button
    if (resetButton) {
        resetButton.addEventListener('click', resetChat);
    }
    
    // Add event listener for Enter key in the input
    if (messageInput) {
        messageInput.addEventListener('keypress', function(e) {
            if (e.key === 'Enter') {
                sendMessage();
            }
        });
    }
    
    function sendMessage() {
        const message = messageInput.value.trim();
        if (message) {
            // Add user message to chat
            addMessageToChat('user', message);
            messageInput.value = '';
            
            // Get project and query from hidden inputs
            const projectName = document.getElementById('project-name').value;
            const queryText = document.getElementById('query-text').value;
            
            // Get conversation history
            const chatHistory = getChatHistory();
            
            // Send message to server with full history
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
                addMessageToChat('assistant', responseText);
                chatContainer.scrollTop = chatContainer.scrollHeight;
            })
            .catch(error => {
                console.error('Error:', error);
                addMessageToChat('system', 'Error: Could not get a response.');
            });
        }
    }
    
    function resetChat() {
        // Clear the chat container
        while (chatContainer.firstChild) {
            chatContainer.removeChild(chatContainer.firstChild);
        }
        
        // Get project name
        const projectName = document.getElementById('project-name').value;
        
        // Reset the chat history on the server
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
        
        // Include the initial prompt if available
        const initialPrompt = document.getElementById('analysis-initial-prompt');
        if (initialPrompt) {
            history.push({
                role: 'system',
                content: initialPrompt.value
            });
        }
        
        // Add all visible messages
        chatMessages.forEach(message => {
            const role = message.classList.contains('user-message') ? 'user' : 'assistant';
            const content = message.querySelector('.message-content').textContent;
            history.push({ role, content });
        });
        
        return history;
    }
    
    function addMessageToChat(role, content) {
        const messageDiv = document.createElement('div');
        messageDiv.className = `chat-message ${role}-message`;
        
        const messageContent = document.createElement('div');
        messageContent.className = 'message-content';
        messageContent.innerHTML = formatMessage(content);
        
        messageDiv.appendChild(messageContent);
        chatContainer.appendChild(messageDiv);
        chatContainer.scrollTop = chatContainer.scrollHeight;
    }
    
    function formatMessage(content) {
        // Convert markdown code blocks to HTML
        return content.replace(/```(\w*)([\s\S]*?)```/g, function(match, language, code) {
            return `<pre><code class="${language}">${code}</code></pre>`;
        }).replace(/\n/g, '<br>');
    }
}