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

  const chatInputContainer = document.querySelector(".chat-input");

  const searchButton = document.createElement("button");
  searchButton.id = "analysis-search-button";
  searchButton.textContent = "Search Files";
  chatInputContainer.appendChild(searchButton);

  const optimizePromptButton = document.createElement("button"); // Renamed button variable
  optimizePromptButton.id = "optimize-prompt-button"; // Renamed ID
  optimizePromptButton.textContent = "Optimize Prompt"; // Renamed text
  chatInputContainer.appendChild(optimizePromptButton); // Add next to search button

  const searchModal = document.createElement("div");
  searchModal.id = "search-results-analysis-modal";
  searchModal.classList.add("analysis-search-modal");
  searchModal.style.display = "none";
  searchModal.innerHTML = `<div class="analysis-search-modal-content">
                        <div class="modal-header">
                            <h3>Search Results</h3>
                            <span class="close-search-modal">&times;</span>
                        </div>
                        <div id="search-results-content-files-modal"></div>
                    </div>`;
  document.body.appendChild(searchModal);

  const optimizePromptModal = document.createElement("div");
  optimizePromptModal.id = "optimize-prompt-modal";
  optimizePromptModal.classList.add("analysis-search-modal");
  optimizePromptModal.style.display = "none";
  optimizePromptModal.innerHTML = `<div class="analysis-search-modal-content">
                        <div class="modal-header">
                            <h3>Optimize Prompt</h3> <!-- Renamed header text -->
                            <span class="close-optimize-prompt-modal">&times;</span> <!-- Renamed close button class -->
                        </div>
                        <div class="modal-body">
                            <p><strong>Original Prompt:</strong></p>
                            <textarea id="original-prompt-display" class="text-area-fmt" rows="3" readonly></textarea>
                            
                            <p><strong>Optimization Direction (Optional):</strong></p>
                            <textarea id="optimization-direction-input" class="text-area-fmt" rows="4" placeholder="e.g., Make it more concise, focus on code structure, simplify technical jargon, include specific keywords..."></textarea>

                            <div class="checkbox-group"> <!-- New container for checkboxes -->
                                <label>
                                    <input type="checkbox" id="include-chat-history-checkbox">
                                    Include Chat Conversation History
                                </label>
                                <label>
                                    <input type="checkbox" id="include-context-files-checkbox">
                                    Include Selected Context Files
                                </label>
                            </div>

                            <button id="generate-optimized-prompt-btn" class="primary">Generate Optimized Prompt</button>
                            <div id="optimized-prompt-loading" style="display:none; color: gray;">Generating...</div>
                            <div id="optimized-prompt-error" style="color: red; display:none;"></div>

                            <p><strong>Optimized Prompt:</strong></p>
                            <textarea id="optimized-prompt-output" class="text-area-fmt" rows="5" readonly></textarea>
                        </div>
                        <div class="modal-footer">
                            <button id="use-optimized-prompt-btn" class="primary" style="display:none;">Use Optimized Prompt</button>
                            <button id="close-optimize-prompt-modal" class="secondary">Close</button>
                        </div>
                    </div>`;
  document.body.appendChild(optimizePromptModal); // Appending new modal variable

  // Event listener for Search Files button
  searchButton.addEventListener("click", async () => {
    const projectName = document.getElementById("project-name").value;
    const promptText = document.getElementById("analysis-message-input").value; // Variable rename for clarity

    const response = await fetch("/search-related-files", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        project: projectName,
        query: promptText, // Backend still expects 'query' for search, which is fine
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

    searchModal.style.display = "block";
  });

  const defaultOptimizationDirections = `
The goal is to make the queries more effective, precise, and clear.

Consider the following aspects when optimizing:
- **Clarity and Specificity:** Make the query unambiguous.
- **Keywords:** Suggest relevant programming terms, API names, design patterns, or function types.
- **Context:** If a direction is provided, incorporate it to focus the query.
- **Conciseness:** Remove unnecessary words without losing meaning.
- **Searchability:** Think about what terms would best match code files.
  `

  // Event listener for Optimize Prompt button
  optimizePromptButton.addEventListener("click", () => { // Changed variable name
    const originalPromptInput = document.getElementById("analysis-message-input");
    document.getElementById("original-prompt-display").value = originalPromptInput.value; // Changed ID
    document.getElementById("optimization-direction-input").value = defaultOptimizationDirections; // Clear previous direction
    document.getElementById("optimized-prompt-output").value = ""; // Changed ID
    document.getElementById("optimized-prompt-loading").style.display = "none"; // Changed ID
    document.getElementById("optimized-prompt-error").style.display = "none"; // Changed ID
    document.getElementById("use-optimized-prompt-btn").style.display = "none"; // Changed ID
    optimizePromptModal.style.display = "block"; // Changed modal variable
  });

  // Event listener for Generate Optimized Prompt button inside the modal
  document.getElementById("generate-optimized-prompt-btn").addEventListener("click", async () => { // Changed ID
    const projectName = document.getElementById("project-name").value;
    const queryId = document.getElementById("query-id").value; // Get query_id for context
    const originalPrompt = document.getElementById("original-prompt-display").value; // Changed ID and variable name
    const optimizationDirection = document.getElementById("optimization-direction-input").value;
    const includeChatHistory = document.getElementById("include-chat-history-checkbox").checked;
    const includeContextFiles = document.getElementById("include-context-files-checkbox").checked;
    const optimizedPromptOutput = document.getElementById("optimized-prompt-output"); // Changed ID and variable name
    const loadingDiv = document.getElementById("optimized-prompt-loading"); // Changed ID
    const errorDiv = document.getElementById("optimized-prompt-error"); // Changed ID
    const useOptimizedButton = document.getElementById("use-optimized-prompt-btn"); // Changed ID

    optimizedPromptOutput.value = "";
    loadingDiv.style.display = "block";
    errorDiv.style.display = "none";
    useOptimizedButton.style.display = "none";

    try {
      const response = await fetch("/optimize-prompt", { // Changed endpoint
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          project: projectName,
          query_id: queryId,
          original_prompt: originalPrompt, // Changed field name
          optimization_direction: optimizationDirection,
          include_chat_history: includeChatHistory,
          include_context_files: includeContextFiles,
        }),
      });

      const data = await response.json();

      if (data.success) {
        optimizedPromptOutput.value = data.optimized_prompt; // Changed field name
        useOptimizedButton.style.display = "block";
      } else {
        errorDiv.textContent = "Error: " + data.error;
        errorDiv.style.display = "block";
      }
    } catch (error) {
      console.error("Error optimizing prompt:", error); // Changed log message
      errorDiv.textContent = "Network error or unexpected response.";
      errorDiv.style.display = "block";
    } finally {
      loadingDiv.style.display = "none";
    }
  });

  // Event listener for Use Optimized Prompt button inside the modal
  document.getElementById("use-optimized-prompt-btn").addEventListener("click", () => { // Changed ID
    const optimizedPrompt = document.getElementById("optimized-prompt-output").value; // Changed ID and variable name
    document.getElementById("analysis-message-input").value = optimizedPrompt;
    optimizePromptModal.style.display = "none"; // Close the modal after using the prompt
  });

  // Event listener for closing Search modal
  searchModal.querySelector(".close-search-modal").addEventListener("click", () => {
    searchModal.style.display = "none";
  });

  // Event listener for closing Optimize Prompt modal
  optimizePromptModal.querySelector(".close-optimize-prompt-modal").addEventListener("click", () => { // Changed class
    optimizePromptModal.style.display = "none";
  });
  document.getElementById("close-optimize-prompt-modal").addEventListener("click", () => { // Changed ID
    optimizePromptModal.style.display = "none";
  });


  // Close modals if clicked outside
  window.addEventListener("click", (event) => {
    if (event.target === searchModal) {
      searchModal.style.display = "none";
    }
    if (event.target === optimizePromptModal) { // Changed modal variable
      optimizePromptModal.style.display = "none";
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

  // Add event listener for the file path checkboxes
  document.body.addEventListener('change', function(event) {
      if (event.target.classList.contains('file-path-checkbox')) {
          const filePath = event.target.value;
          const isChecked = event.target.checked;

          // Find the corresponding checkbox in the file list
          const fileListCheckbox = document.querySelector(`.file-list input[type="checkbox"][value="${filePath}"]`);

          if (fileListCheckbox) {
              fileListCheckbox.checked = isChecked;
          }

          // Trigger updateContext to reflect the change
          updateContext(projectName, queryText);
      }
  });

    // Scroll to bottom of chat
    if (chatContainer) {
        chatContainer.scrollTop = chatContainer.scrollHeight;
    }
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initAnalysisChat);
} else {
  initAnalysisChat();
}