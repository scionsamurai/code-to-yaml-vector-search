// static/analyze-query.js
function initAnalysisChat() {
    const chatContainer = document.getElementById('analysis-chat-container');
    const messageInput = document.getElementById('analysis-message-input');
    const sendButton = document.getElementById('analysis-send-button');
    const resetButton = document.getElementById('analysis-reset-button');
    
    // Add event listeners
    if (sendButton) sendButton.addEventListener('click', sendMessage);
    if (resetButton) resetButton.addEventListener('click', resetChat);
    
    // Add event listener for Enter key in the input
    if (messageInput) {
        messageInput.addEventListener('keypress', function(e) {
            if (e.key === 'Enter') {
                sendMessage();
            }
        });
    }
    
    // Add toggle-all functionality for both sections
    const toggleRelevantButton = document.getElementById('toggle-relevant-files');
    if (toggleRelevantButton) {
        toggleRelevantButton.addEventListener('click', function() {
            toggleAllCheckboxes('relevant-files-list');
            updateContext(); // Update after toggling
        });
    }
    
    const toggleOtherButton = document.getElementById('toggle-other-files');
    if (toggleOtherButton) {
        toggleOtherButton.addEventListener('click', function() {
            toggleAllCheckboxes('other-files-list');
            updateContext(); // Update after toggling
        });
    }
    
    // Add event listeners to all file checkboxes for automatic updates
    document.querySelectorAll('.file-checkbox').forEach(checkbox => {
        checkbox.addEventListener('change', updateContext);
    });
    
    function toggleAllCheckboxes(containerId) {
        const container = document.getElementById(containerId);
        const checkboxes = container.querySelectorAll('input[type="checkbox"]');
        const allChecked = Array.from(checkboxes).every(cb => cb.checked);
        
        checkboxes.forEach(checkbox => {
            checkbox.checked = !allChecked;
        });
    }
    
    function updateContext() {
        // Show loading indicator
        const statusMessage = document.getElementById('context-status');
        if (statusMessage) {
            statusMessage.textContent = 'Updating context...';
            statusMessage.style.display = 'block';
        }
        
        // Get all selected files
        const selectedFiles = [];
        document.querySelectorAll('.file-checkbox:checked').forEach(checkbox => {
            selectedFiles.push(checkbox.value);
        });
        
        // Get project name
        const projectName = document.getElementById('project-name').value;
        const queryText = document.getElementById('query-text').value;
        
        // Send request to update context
        fetch('/update-analysis-context', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                project: projectName,
                query: queryText,
                files: selectedFiles
            })
        })
        .then(response => response.json())
        .then(data => {
            if (data.success) {
                if (statusMessage) {
                    statusMessage.textContent = `Context updated: ${selectedFiles.length} files selected`;
                    // Make the message fade out after 2 seconds
                    setTimeout(() => {
                        statusMessage.style.opacity = '0';
                        setTimeout(() => {
                            statusMessage.style.display = 'none';
                            statusMessage.style.opacity = '1';
                        }, 500);
                    }, 2000);
                }
                

            } else {
                if (statusMessage) {
                    statusMessage.textContent = 'Error: Failed to update context.';
                }
            }
        })
        .catch(error => {
            console.error('Error:', error);
            if (statusMessage) {
                statusMessage.textContent = 'Error: Could not update context.';
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
                addMessageToChat('model', responseText);
                chatContainer.scrollTop = chatContainer.scrollHeight;
            })
            .catch(error => {
                console.error('Error:', error);
                addMessageToChat('model', 'Error: Could not get a response.');
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
        
        // Add all visible messages
        chatMessages.forEach(message => {
            if (message.classList.contains('system-message')) return; // Skip system messages
            
            const role = message.classList.contains('user-message') ? 'user' : 'model';
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