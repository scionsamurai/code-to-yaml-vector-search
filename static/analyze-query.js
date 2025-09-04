// static/analyze-query.js
import { initializeElements } from "./analyze-query/elements.js";
import { updateContext } from "./analyze-query/context.js";
import { setupQueryEditor, setupTitleEditor } from "./analyze-query/query.js";
import {
  sendMessage,
  resetChat,
  toggleEditMode,
  toggleHideMessage,
  regenerateLastMessage, // Import regenerateLastMessage
  addMessageToChat // Import addMessageToChat (not strictly needed here but good for consistency)
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

  // Event listeners for existing messages (loaded from HTML)
  document.querySelectorAll(".edit-message-btn").forEach((button) => {
    button.addEventListener("click", function () {
      const messageDiv = button.closest(".chat-message");
      toggleEditMode(messageDiv);
    });
  });
  document.querySelectorAll(".hide-message-btn").forEach((button) => {
    button.addEventListener("click", function () {
      const messageDiv = button.closest(".chat-message");
      toggleHideMessage(messageDiv);
    });
  });
  document.querySelectorAll(".regenerate-message-btn").forEach((button) => {
    button.addEventListener("click", function () {
      const messageDiv = button.closest(".chat-message");
      regenerateLastMessage(messageDiv);
    });
  });

  // Format existing messages from Markdown to HTML and store original content
  const chatMessages = chatContainer.querySelectorAll(".chat-message");
  chatMessages.forEach((messageDiv) => {
    const messageContent = messageDiv.querySelector(".message-content");
    // Ensure original content is correctly stored if not already from data-original-content attribute
    // We already added `msg.content` to the div innerHTML in render_analyze_query_page, which is the raw content.
    // So we can use that for originalContent, and then format it.
    const rawContent = messageContent.innerHTML;
    messageDiv.dataset.originalContent = rawContent;
    messageContent.innerHTML = formatMessage(rawContent);
  });

  // Apply syntax highlighting to existing code blocks
  await applySyntaxHighlighting();

  updateCopyLinks();

  const searchButton = document.createElement("button");
  searchButton.id = "analysis-search-button";
  searchButton.textContent = "Search Files";
  document.querySelector(".chat-input").appendChild(searchButton);

  const modal = document.createElement("div");
  modal.id = "search-results-analysis-modal";
  modal.classList.add("analysis-search-modal"); // Use classList
  modal.style.display = "none"; // Initially hidden
  modal.innerHTML = `<div class="analysis-search-modal-content">
                        <div class="modal-header">
                            <h3>Search Results</h3>
                            <span class="close-search-modal">&times;</span>
                        </div>
                        <div id="search-results-content-files-modal"></div>
                    </div>`;
  document.body.appendChild(modal);

  searchButton.addEventListener("click", async () => {
    const projectName = document.getElementById("project-name").value;
    const queryText = document.getElementById("analysis-message-input").value;

    const response = await fetch("/search-related-files", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        project: projectName,
        query: queryText,
      }),
    });

    const data = await response.json();

    let searchResultsContent = document.getElementById(
      "search-results-content-files-modal"
    );
    searchResultsContent.innerHTML = ""; // Clear previous results

    if (data.success) {
      searchResultsContent.innerHTML = data.html; // Insert the HTML
    } else {
      searchResultsContent.textContent = "Error: " + data.error;
    }

    modal.style.display = "block"; // Show the modal
  });

  // Close the modal when the close button is clicked
  modal.querySelector(".close-search-modal").addEventListener("click", () => {
    modal.style.display = "none";
  });

  // Close the modal if the user clicks outside of it
  window.addEventListener("click", (event) => {
    if (event.target === modal) {
      modal.style.display = "none";
    }
  });
}

// Initialize the chat when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initAnalysisChat);
} else {
  initAnalysisChat();
}