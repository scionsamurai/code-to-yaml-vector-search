// static/analyze-query.js
import { initializeElements } from "./analyze-query/elements.js";
import { updateContext } from "./analyze-query/context.js";
import {
  sendMessage,
  resetChat,
  toggleEditMode,
} from "./analyze-query/chat.js";
import { applySyntaxHighlighting } from "./analyze-query/syntax-highlighting.js";
import { formatMessage } from "./analyze-query/utils.js";

async function initAnalysisChat() {
  const projectName = document.getElementById("project-name").value;
  const queryText = document.getElementById("query-text").value;

  const { chatContainer } = initializeElements(
    () => sendMessage(chatContainer),
    () => resetChat(chatContainer),
    () => updateContext(projectName, queryText)
  );

  document.querySelectorAll(".edit-message-btn").forEach((button) => {
    button.addEventListener("click", function () {
      const messageDiv = button.closest(".chat-message");
      toggleEditMode(messageDiv);
    });
  });

  // Format existing messages from Markdown to HTML
  const chatMessages = chatContainer.querySelectorAll(".chat-message");
  chatMessages.forEach((messageDiv) => {
    const messageContent = messageDiv.querySelector(".message-content");
    const originalContent =
      messageDiv.dataset.originalContent || messageContent.textContent;
    messageContent.innerHTML = formatMessage(originalContent);
  });

  // Apply syntax highlighting to existing code blocks
  await applySyntaxHighlighting();
}

// Initialize the chat when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initAnalysisChat);
} else {
  initAnalysisChat();
}
