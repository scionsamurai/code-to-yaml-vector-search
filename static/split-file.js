//  static/split-file.js
function suggestSplit(project, filePath) {
    // Show a loading indicator
    const loadingDiv = document.createElement('div');
    loadingDiv.id = 'loading-overlay';
    loadingDiv.innerHTML = '<div class="loading-spinner"></div><div>Analyzing file structure...</div>';
    document.body.appendChild(loadingDiv);
    
    // Make the request
    fetch(`/suggest-split?project=${encodeURIComponent(project)}&file_path=${encodeURIComponent(filePath)}`, {
        method: 'POST',
    })
    .then(response => response.text())
    .then(data => {
        // Remove loading indicator
        document.body.removeChild(document.getElementById('loading-overlay'));
        
        // Show the result in a modal with chat interface
        const modal = document.createElement('div');
        modal.className = 'modal';
        modal.innerHTML = data; // The server now returns the complete HTML with chat interface
        document.body.appendChild(modal);
        
        // Initialize the chat functionality
        initSplitChat();
    })
    .catch(error => {
        // Remove loading indicator
        document.body.removeChild(document.getElementById('loading-overlay'));
        alert('Error: ' + error);
    });
}

// This will be called after the modal is created
function initSplitChat() {
    const chatContainer = document.getElementById('chat-container');
    const messageInput = document.getElementById('message-input');
    const sendButton = document.getElementById('send-button');
    
    // Add event listener for the send button
    if (sendButton) {
        sendButton.addEventListener('click', sendMessage);
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
            
            // Get project and file path from hidden inputs
            const projectName = document.getElementById('project-name').value;
            const filePath = document.getElementById('file-path').value;
            
            // Get conversation history
            const chatHistory = getChatHistory();
            
            // Send message to server with full history
            fetch('/chat-split', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },

                body: JSON.stringify({
                    project: projectName,
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
    
    function getChatHistory() {
        const chatMessages = document.querySelectorAll('.chat-message');
        let history = [];
        
        // Include the initial prompt if available
        const initialPrompt = document.getElementById('initial-prompt');
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