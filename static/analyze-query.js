// static/analyze-query.js
import { initializeElements } from './analyze-query/elements.js';
import { updateContext } from './analyze-query/context.js';
import { sendMessage, resetChat, toggleEditMode } from './analyze-query/chat.js';

function initAnalysisChat() {
    const projectName = document.getElementById('project-name').value;
    const queryText = document.getElementById('query-text').value;

    const { chatContainer } = initializeElements(
        () => sendMessage(chatContainer),
        () => resetChat(chatContainer),
        () => updateContext(projectName, queryText)
    );

    document.querySelectorAll('.edit-message-btn').forEach(button => {
        button.addEventListener('click', function() {
            const messageDiv = button.closest('.chat-message');
            toggleEditMode(messageDiv);
        });
    });

}

initAnalysisChat();