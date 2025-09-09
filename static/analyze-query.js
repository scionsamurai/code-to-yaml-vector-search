// static/analyze-query.js
import { initializeElements } from "./analyze-query/elements.js";
import { updateContext } from "./analyze-query/context.js";
import { setupQueryEditor, setupTitleEditor } from "./analyze-query/query.js";
import {
  sendMessage,
  resetChat,
  toggleEditMode,
  toggleHideMessage,
  regenerateLastMessage,
  addMessageToChat
} from "./analyze-query/chat.js";
import {
  applySyntaxHighlighting,
} from "./analyze-query/syntax-highlighting.js";
import { initCodeBlockActions } from "./analyze-query/code-block-actions.js";
import { formatMessage, setProjectSourceDirectory, linkFilePathsInElement } from "./analyze-query/utils.js";

async function initAnalysisChat() {
  const projectName = document.getElementById("project-name").value;
  const queryText = document.getElementById("query-text").value;
  const projectSourceDir = document.getElementById("project-source-dir").value;

  setProjectSourceDirectory(projectSourceDir);

  setupQueryEditor(projectName);
  setupTitleEditor(projectName);

  const { chatContainer } = initializeElements(
    () => sendMessage(chatContainer),
    () => resetChat(chatContainer),
    () => updateContext(projectName, queryText)
  );

  const querySelector = document.getElementById("query-selector");
  if (querySelector) {
    querySelector.addEventListener("change", function () {
      const selectedQueryId = this.value;

      const chatContainer = document.getElementById("analysis-chat-container");
      if (chatContainer) {
        chatContainer.innerHTML = "";
      }

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
  for (const messageDiv of chatMessages) {
    const messageContent = messageDiv.querySelector(".message-content");
    const rawContent = messageContent.innerHTML;
    messageDiv.dataset.originalContent = rawContent;
    messageContent.innerHTML = formatMessage(rawContent);
  }

  await applySyntaxHighlighting();

  chatMessages.forEach(messageDiv => {
    linkFilePathsInElement(messageDiv.querySelector('.message-content'));
  });

  initCodeBlockActions();

  const searchButton = document.createElement("button");
  searchButton.id = "analysis-search-button";
  searchButton.textContent = "Search Files";
  document.querySelector(".chat-input").appendChild(searchButton);

  const modal = document.createElement("div");
  modal.id = "search-results-analysis-modal";
  modal.classList.add("analysis-search-modal");
  modal.style.display = "none";
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
    searchResultsContent.innerHTML = "";

    if (data.success) {
      searchResultsContent.innerHTML = data.html;
      // Optional: If you want to link paths in search results as well, uncomment:
      // linkFilePathsInElement(searchResultsContent);
    } else {
      searchResultsContent.textContent = "Error: " + data.error;
    }

    modal.style.display = "block";
  });

  modal.querySelector(".close-search-modal").addEventListener("click", () => {
    modal.style.display = "none";
  });

  window.addEventListener("click", (event) => {
    if (event.target === modal) {
      modal.style.display = "none";
    }
  });

    document.body.addEventListener('click', (event) => {
    const link = event.target.closest('a.file-path-link');
    if (link) {
      event.preventDefault(); // Prevent default browser navigation

      // Create an invisible iframe
      const iframe = document.createElement('iframe');
      iframe.style.display = 'none'; // Keep it hidden
      document.body.appendChild(iframe);

      // Set the iframe's source to the VS Code URI
      // This will attempt to trigger the VS Code application via the iframe's isolated context.
      iframe.src = link.href;

      // Clean up the iframe after a short delay
      // This is not strictly necessary for functionality, but keeps the DOM tidy.
      setTimeout(() => {
        try {
          document.body.removeChild(iframe);
        } catch (e) {
          console.warn("Could not remove temporary iframe:", e);
        }
      }, 500); // Give the browser half a second to process the URI scheme
    }
  });
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initAnalysisChat);
} else {
  initAnalysisChat();
}