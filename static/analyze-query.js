// static/analyze-query.js
import { initializeElements } from "./analyze-query/elements.js";
import { updateContext } from "./analyze-query/context.js";
import { setupQueryEditor, setupTitleEditor } from "./analyze-query/query.js";
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

  setupQueryEditor(projectName);
  setupTitleEditor(projectName);

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

    // Add event listeners to yaml checkboxes
    document.querySelectorAll('.yaml-checkbox').forEach(checkbox => {
        checkbox.addEventListener('change', function() {
            const filePath = this.value;
            const useYaml = this.checked;
            updateFileYamlOverride(projectName, filePath, useYaml);
        });
    });
}

// Initialize the chat when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initAnalysisChat);
} else {
  initAnalysisChat();
}


async function updateFileYamlOverride(projectName, filePath, useYaml) {
  console.log('Updating YAML override for:', projectName, filePath, useYaml);
  try {
    const response = await fetch('/update-file-yaml-override', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        project: projectName,
        file_path: filePath,
        use_yaml: useYaml
      })
    });
    console.log('response', response);

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const result = await response.json();
    console.log('File YAML override updated:', result);

    // Optionally, update the UI to reflect the change
    const contextStatus = document.getElementById('context-status');
    contextStatus.textContent = `YAML setting for ${filePath} updated.`;
    contextStatus.style.display = 'block';
    setTimeout(() => {
      contextStatus.style.opacity = 0;
      setTimeout(() => {
        contextStatus.style.display = 'none';
        contextStatus.style.opacity = 1;
      }, 500);
    }, 3000);

  } catch (error) {
    console.error('Error updating file YAML override:', error);
    alert('Failed to update YAML setting. See console for details.');
  }
}