// static/analyze-query/elements.js

export function initializeElements(sendMessage, resetChat, updateContext) {
    const chatContainer = document.getElementById('analysis-chat-container');
    const messageInput = document.getElementById('analysis-message-input');
    const sendButton = document.getElementById('analysis-send-button');
    const resetButton = document.getElementById('analysis-reset-button');
    const toggleRelevantButton = document.getElementById('toggle-relevant-files');
    const toggleOtherButton = document.getElementById('toggle-other-files');
    
    if (sendButton) sendButton.addEventListener('click', sendMessage);
    if (resetButton) resetButton.addEventListener('click', resetChat);
    
    if (messageInput) {
        messageInput.addEventListener('keypress', function(e) {
            if (e.key === 'Enter') {
                sendMessage();
            }
        });
    }
    
    if (toggleRelevantButton) {
        toggleRelevantButton.addEventListener('click', function() {
            toggleAllCheckboxes('relevant-files-list');
            updateContext(); // Update after toggling
        });
    }
    
    if (toggleOtherButton) {
        toggleOtherButton.addEventListener('click', function() {
            toggleAllCheckboxes('other-files-list');
            updateContext(); // Update after toggling
        });
    }
    
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

    return { chatContainer };
}