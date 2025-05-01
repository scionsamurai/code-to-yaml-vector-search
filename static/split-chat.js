// Updated static/split-chat.js
document.addEventListener('DOMContentLoaded', function() {
    const chatContainer = document.getElementById('chat-container');
    const messageInput = document.getElementById('message-input');
    const sendButton = document.getElementById('send-button');
    
    // Chat history to track all messages
    let chatHistory = [];
    
    // Initialize chat history with the initial prompt and response
    const initialPrompt = document.getElementById('initial-prompt').value;
    const initialResponse = document.getElementById('initial-response').value;
    
    // Add initial messages to history
    if (initialPrompt && initialResponse) {
        chatHistory.push({
            role: "system",
            content: initialPrompt
        });
        
        chatHistory.push({
            role: "assistant",
            content: initialResponse
        });
    }
    
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
            
            // Add to chat history
            chatHistory.push({
                role: "user",
                content: message
            });
            
            messageInput.value = '';
            
            // Get project and file path from hidden inputs
            const projectName = document.getElementById('project-name').value;
            const filePath = document.getElementById('file-path').value;
            
            // Send message to server with full chat history
            fetch('/chat-split', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    project: projectName,
                    file_path: filePath,
                    message: message,
                    history: chatHistory
                })
            })
            .then(response => response.text())
            .then(responseText => {
                // Add assistant response to chat
                addMessageToChat('assistant', responseText);
                
                // Add to chat history
                chatHistory.push({
                    role: "assistant",
                    content: responseText
                });
                
                chatContainer.scrollTop = chatContainer.scrollHeight;
            })
            .catch(error => {
                console.error('Error:', error);
                addMessageToChat('system', 'Error: Could not get a response.');
            });
        }
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
});