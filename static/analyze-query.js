// static/analyze-query.js
import { initializeElements } from "./analyze-query/elements.js";
import { updateContext } from "./analyze-query/context.js";
import { setupQueryEditor, setupTitleEditor } from "./analyze-query/query.js";
import {
  sendMessage,
  resetChat,
  toggleEditMode,
  toggleHideMessage,
  regenerateMessage,
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

      // Clear the chat container to prepare for new query's history (optional, as page will reload)
      if (chatContainer) {
        chatContainer.innerHTML = "";
      }

      // --- MODIFIED START ---
      // Keeping this as a full page reload for now due to complexity of re-rendering entire page context
      window.location.href = `/analyze-query/${projectName}/${selectedQueryId}`;
      // --- MODIFIED END ---
    });
  }

  // --- Initializing existing chat messages (from server-rendered HTML) ---
  const chatMessages = chatContainer.querySelectorAll(".chat-message");
  for (const messageDiv of chatMessages) {
    const messageContent = messageDiv.querySelector(".message-content");
    const rawContent = messageDiv.dataset.originalContent || messageContent.innerHTML; // Get original content from dataset or innerHTML
    messageDiv.dataset.originalContent = rawContent; // Ensure it's stored

    messageContent.innerHTML = formatMessage(rawContent); // Apply formatting on load

    // Setup event listeners for existing buttons
    // These listeners now pass the messageDiv directly, and the functions will extract data-message-id
    const editButton = messageDiv.querySelector('.edit-message-btn');
    if (editButton) {
        editButton.addEventListener('click', () => toggleEditMode(messageDiv));
    }
    const hideButton = messageDiv.querySelector('.hide-message-btn');
    if (hideButton) {
        hideButton.addEventListener('click', () => toggleHideMessage(messageDiv));
    }
    // Check if a regenerate button already exists, if not, add it for model messages
    // This is to ensure pre-rendered model messages also have the button
    let regenerateButton = messageDiv.querySelector('.regenerate-message-btn');
    if (!regenerateButton && messageDiv.classList.contains('model-message')) {
        // Find message controls and append the button if it's a model message
        const messageControls = messageDiv.querySelector('.message-controls');
        if (messageControls) {
            regenerateButton = document.createElement('button'); // createRegenerateButton is not directly exposed to analyze-query.js
            regenerateButton.className = 'regenerate-message-btn';
            regenerateButton.textContent = 'Regenerate';
            regenerateButton.title = 'Regenerate response';
            messageControls.appendChild(regenerateButton);
        }
    }
    if (regenerateButton) {
        regenerateButton.addEventListener('click', () => regenerateMessage(messageDiv));
    }
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

  const optimizePromptButton = document.createElement("button");
  optimizePromptButton.id = "optimize-prompt-button";
  optimizePromptButton.textContent = "Optimize Prompt";
  chatInputContainer.appendChild(optimizePromptButton);

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
                            <h3>Optimize Prompt</h3>
                            <span class="close-optimize-prompt-modal">&times;</span>
                        </div>
                        <div class="modal-body">
                            <p><strong>Original Prompt:</strong></p>
                            <textarea id="original-prompt-display" class="text-area-fmt" rows="3" readonly></textarea>
                            
                            <p><strong>Optimization Direction (Optional):</strong></p>
                            <textarea id="optimization-direction-input" class="text-area-fmt" rows="4" placeholder="e.g., Make it more concise, focus on code structure, simplify technical jargon, include specific keywords..."></textarea>

                            <div class="checkbox-group">
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
  document.body.appendChild(optimizePromptModal);

  const defaultOptimizationDirections = `
The goal is to make the queries more effective, precise, and clear.

Consider the following aspects when optimizing:
- **Clarity and Specificity:** Make the query unambiguous.
- **Keywords:** Suggest relevant programming terms, API names, design patterns, or function types.
- **Context:** If a direction is provided, incorporate it to focus the query.
- **Conciseness:** Remove unnecessary words without losing meaning.
- **Searchability:** Think about what terms would best match code files.
  `

  // Event listener for Search Files button
  searchButton.addEventListener("click", async () => {
    const projectName = document.getElementById("project-name").value;
    const promptText = document.getElementById("analysis-message-input").value;

    const response = await fetch("/search-related-files", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        project: projectName,
        query: promptText,
      }),
    });

    const data = await response.json();

    let searchResultsContent = document.getElementById(
      "search-results-content-files-modal"
    );
    searchResultsContent.innerHTML = "";

    if (data.success) {
      searchResultsContent.innerHTML = data.html;
    } else {
      searchResultsContent.textContent = "Error: " + data.error;
    }

    searchModal.style.display = "block";
  });

  // Event listener for Optimize Prompt button
  optimizePromptButton.addEventListener("click", () => {
    const originalPromptInput = document.getElementById("analysis-message-input");
    document.getElementById("original-prompt-display").value = originalPromptInput.value;
    document.getElementById("optimization-direction-input").value = defaultOptimizationDirections;
    document.getElementById("optimized-prompt-output").value = "";
    document.getElementById("optimized-prompt-loading").style.display = "none";
    document.getElementById("optimized-prompt-error").style.display = "none";
    document.getElementById("use-optimized-prompt-btn").style.display = "none";
    optimizePromptModal.style.display = "block";
  });

  // Event listener for Generate Optimized Prompt button inside the modal
  document.getElementById("generate-optimized-prompt-btn").addEventListener("click", async () => {
    const projectName = document.getElementById("project-name").value;
    const queryId = document.getElementById("query-id").value;
    const originalPrompt = document.getElementById("original-prompt-display").value;
    const optimizationDirection = document.getElementById("optimization-direction-input").value;
    const includeChatHistory = document.getElementById("include-chat-history-checkbox").checked;
    const includeContextFiles = document.getElementById("include-context-files-checkbox").checked;
    const optimizedPromptOutput = document.getElementById("optimized-prompt-output");
    const loadingDiv = document.getElementById("optimized-prompt-loading");
    const errorDiv = document.getElementById("optimized-prompt-error");
    const useOptimizedButton = document.getElementById("use-optimized-prompt-btn");

    optimizedPromptOutput.value = "";
    loadingDiv.style.display = "block";
    errorDiv.style.display = "none";
    useOptimizedButton.style.display = "none";

    try {
      const response = await fetch("/optimize-prompt", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          project: projectName,
          query_id: queryId,
          original_prompt: originalPrompt,
          optimization_direction: optimizationDirection,
          include_chat_history: includeChatHistory,
          include_context_files: includeContextFiles,
        }),
      });

      const data = await response.json();

      if (data.success) {
        optimizedPromptOutput.value = data.optimized_prompt;
        useOptimizedButton.style.display = "block";
      } else {
        errorDiv.textContent = "Error: " + data.error;
        errorDiv.style.display = "block";
      }
    } catch (error) {
      console.error("Error optimizing prompt:", error);
      errorDiv.textContent = "Network error or unexpected response.";
      errorDiv.style.display = "block";
    } finally {
      loadingDiv.style.display = "none";
    }
  });

  // Event listener for Use Optimized Prompt button inside the modal
  document.getElementById("use-optimized-prompt-btn").addEventListener("click", () => {
    const optimizedPrompt = document.getElementById("optimized-prompt-output").value;
    document.getElementById("analysis-message-input").value = optimizedPrompt;
    optimizePromptModal.style.display = "none";
  });

  // Event listener for closing Search modal
  searchModal.querySelector(".close-search-modal").addEventListener("click", () => {
    searchModal.style.display = "none";
  });

  // Event listener for closing Optimize Prompt modal
  optimizePromptModal.querySelector(".close-optimize-prompt-modal").addEventListener("click", () => {
    optimizePromptModal.style.display = "none";
  });
  document.getElementById("close-optimize-prompt-modal").addEventListener("click", () => {
    optimizePromptModal.style.display = "none";
  });


  // Close modals if clicked outside
  window.addEventListener("click", (event) => {
    if (event.target === searchModal) {
      searchModal.style.display = "none";
    }
    if (event.target === optimizePromptModal) {
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

    // Handle branch navigation clicks
    chatContainer.addEventListener('click', async (event) => {
        const branchNavButton = event.target.closest('.branch-nav-btn');
        if (branchNavButton && !branchNavButton.disabled) {
            const newCurrentNodeId = branchNavButton.dataset.navTargetId;
            const projectName = document.getElementById('project-name').value;
            const queryId = document.getElementById('query-id').value;

            try {
                const response = await fetch('/set-current-chat-node', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        project_name: projectName,
                        query_id: queryId,
                        new_current_node_id: newCurrentNodeId
                    })
                });

                if (response.ok) {
                    console.log(`Current chat node set to: ${newCurrentNodeId}. Reloading chat.`);
                    // Trigger a full page reload to reflect the new chat branch
                    location.reload(); // Keeping reload here for full branch context update
                } else {
                    const errorText = await response.text();
                    console.error('Error setting current chat node:', errorText);
                    alert(`Failed to switch branch: ${errorText}`);
                }
            } catch (error) {
                console.error('Network error switching branch:', error);
                alert(`A network error occurred: ${error.message}`);
            }
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