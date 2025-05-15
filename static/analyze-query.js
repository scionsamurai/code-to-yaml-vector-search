// static/analyze-query.js
import { initializeElements } from "./analyze-query/elements.js";
import { updateContext } from "./analyze-query/context.js";
import { setupQueryEditor } from "./analyze-query/query.js";
import {
  sendMessage,
  resetChat,
  toggleEditMode,
} from "./analyze-query/chat.js";
import {
  applySyntaxHighlighting,
  updateCopyLinks,
} from "./analyze-query/syntax-highlighting.js";
import { formatMessage } from "./analyze-query/utils.js";


async function initAnalysisChat() {
  const projectName = document.getElementById("project-name").value;
  const queryText = document.getElementById("query-text").value;

  // Setup query editor functionality
  setupQueryEditor(projectName);

  const { chatContainer } = initializeElements(
    () => sendMessage(chatContainer),
    () => resetChat(chatContainer),
    () => updateContext(projectName, queryText)
  );

  // Add event listener to the query selector
  const querySelector = document.getElementById("query-selector");
  if (querySelector) {
    querySelector.addEventListener("change", function () {
      const selectedQueryId = this.value;

      // **Clear the chat history container before submitting**
      const chatContainer = document.getElementById("analysis-chat-container"); // Get the chat container element
      if (chatContainer) {
        chatContainer.innerHTML = ""; // Clear the chat container's content
      }

      // Submit the form to load the selected query
      const form = document.createElement("form");
      form.method = "post";
      form.action = "/analyze-query";

      // Add the project
      const projectInput = document.createElement("input");
      projectInput.type = "hidden";
      projectInput.name = "project";
      projectInput.value = projectName;
      form.appendChild(projectInput);

      const queryIdInput = document.createElement("input");
      queryIdInput.type = "hidden";
      queryIdInput.name = "query_id";
      queryIdInput.value = selectedQueryId;
      form.appendChild(queryIdInput);

      document.body.appendChild(form);
      form.submit();
    });
  }

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

  updateCopyLinks();
}

// Initialize the chat when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initAnalysisChat);
} else {
  initAnalysisChat();
}
