<!-- frontend/src/components/ChatInterface.svelte -->
<script lang="ts">
  import { onMount } from 'svelte'; // Import onMount
  import BranchNavigation from './BranchNavigation.svelte';
  import { formatMessage } from "../lib/analyze-query/utils.js";
  import { highlightAction } from "../lib/analyze-query/syntax-highlighting.js";

  let { 
    project_name, 
    query_id, 
    chatHistory, 
    branch_display_data, 
    git_stuff, 
    selectedFiles,
    project_provider,             // NEW: Receive project_provider
    initialGroundingWithSearch,   // NEW: Receive initial grounding state
    sendMessage: propsSendMessage, 
    resetChat: propsResetChat, 
    toggleSearchModal, 
    toggleOptimizePromptModal,
    onFileCheckboxChange,
    onGroundingToggle,            // NEW: Receive callback for grounding changes
  } = $props();
  
  let messageInput = $state('');
  let chatContainer: HTMLElement;

  // --- Local State ---
  let scroll_height = $state(0);
  let editingMessageId: string | null = $state(null); // Track which message is being edited
  let enableGroundingWithSearch = $state(initialGroundingWithSearch); // NEW: Local state for grounding

  // --- Effects ---
  // NEW: Update local grounding state if initial prop changes (e.g., query switch)
  $effect(() => {
    enableGroundingWithSearch = initialGroundingWithSearch;
  });

  function scrollToBottom() {
    if (chatContainer) {
      chatContainer.scrollTop = chatContainer.scrollHeight;
    }
  }

  // Existing handleSend function
  async function handleSend() {
    if (messageInput.trim()) {
      await sendMessage(messageInput.trim()); // Use the passed down sendMessage
      messageInput = '';
      
      chatContainer.scrollTo({
        top: scroll_height + 100,
        behavior: 'smooth'
      });
    }
  }
  async function sendMessage(message: string) {
    try {
        const response = await fetch('/chat-analysis', { // Corrected path
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                project: project_name,
                query_id: query_id,
                message: message,
            }),
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const newMessage = await response.json();
        chatHistory = [...chatHistory, newMessage.user_message, newMessage.model_message]; // Update chat history
    } catch (error) {
        console.error('Error sending message:', error);
        console.log('Project:', project_name, 'Query ID:', query_id, 'Message:', message);
        if (typeof error === 'object' && error !== null && 'message' in error) alert(`Error sending message: ${error.message}`);
    }
  }
  async function resetChat() {
      if (confirm('Are you sure you want to reset the chat?')) {
        try {
          
          const response = await fetch('/reset-analysis-chat', {
              method: 'POST',
              headers: {
                  'Content-Type': 'application/json',
              },
              body: JSON.stringify({
                  project: project_name,
                  query_id: query_id
              })
          });
          
          if (response.ok) {
            window.location.reload(); // Simplest way to reset the chat
          } else {
            throw new Error(`Failed to reset chat: ${response.statusText}`);
          }
        } catch (error) {
          console.error('Error resetting chat:', error);
          if (typeof error === 'object' && error !== null && 'message' in error) alert(`Error resetting chat: ${error.message}`);
        }
      }
    }
    async function saveEditedMessage(messageId: string, content: string, createNewBranch: boolean) {
      try {
        const response = await fetch('/update-chat-message', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                project: project_name,
                content: content,
                message_id: messageId,
                query_id: query_id,
                create_new_branch: createNewBranch
            })
        });

        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText);
        }

        const data = await response.json();

        if (data.success) {
            if (createNewBranch) {
                // If branching, remove the old message from history and add the new one
                chatHistory = chatHistory.filter((msg: any) => msg.id !== messageId);
                chatHistory = [...chatHistory, data.message];
            } else {
                // Find and update the message in chatHistory
                chatHistory = chatHistory.map((msg: any) => {
                    if (msg.id === messageId) {
                        return data.message;
                    }
                    return msg;
                });
            }
        } else {
            alert(`Failed to save edited message: ${data.message || 'Unknown error.'}`);
        }
    } catch (error) {
        console.error('Error saving edited message:', error);
        if (typeof error === 'object' && error !== null && 'message' in error) alert(`Failed to save edited message: ${error.message}.`);
    } finally {
        editingMessageId = null; // Ensure edit mode is off
    }
}

  // Edit mode toggle
  function toggleEditMode(messageId: string) {
    if (editingMessageId === messageId) {
      editingMessageId = null; // Exit edit mode
    } else {
      editingMessageId = messageId; // Enter edit mode
    }
  }
  async function toggleHideMessage(messageId: string, hidden: boolean) {
      try {
        const response = await fetch('/update-message-visibility', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                project: project_name,
                message_id: messageId,
                query_id: query_id,
                hidden: hidden
            })
        });

        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText);
        }

        chatHistory = chatHistory.map((msg: any) => {
            if (msg.id === messageId) {
                return { ...msg, hidden: hidden };
            }
            return msg;
        });

        console.log(`Message ${messageId} visibility updated to hidden=${hidden}.`);
    } catch (error) {
        console.error('Error saving hidden message:', error);
        if (typeof error === 'object' && error !== null && 'message' in error) alert(`Failed to update message visibility: ${error.message}.`);
    }
}

async function regenerateMessage(messageId: string) {
    try {
        const response = await fetch('/regenerate-chat-message', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                project: project_name,
                query_id: query_id,
                message_id: messageId // Send the ID of the model message to regenerate from
            })
        });

        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText);
        }

        const data = await response.json();

        if (data.success) {
            chatHistory = chatHistory.map((msg: any) => {
                if (msg.id === messageId) {
                    return data.new_model_message;
                }
                return msg;
            });
        } else {
            alert(`Failed to regenerate response: ${data.message || 'Unknown error.'}`);
        }
    } catch (error) {
        console.error('Network error during regeneration', error);
        if (typeof error === 'object' && error !== null && 'message' in error) alert(`A network error occurred during regeneration ${error.message}.`);
    }
}

// NEW: Handler for local grounding toggle, emits to parent
const handleGroundingToggle = async (newValue: boolean) => {
  enableGroundingWithSearch = newValue;
  onGroundingToggle(newValue); // Call the parent's callback
};

  // --- Mount Effect ---

  onMount(() => {
    // VS Code iframe opening logic
    chatContainer.addEventListener('click', (event) => {
        const link = (event.target as HTMLElement).closest('a.file-path-link') as HTMLAnchorElement;
        if (link) {
            event.preventDefault(); // Prevent default browser navigation
            console.log('link clicked:', link, event);
            // Create an invisible iframe
            const iframe = document.createElement('iframe');
            iframe.style.display = 'none'; // Keep it hidden
            document.body.appendChild(iframe);

            // Set the iframe's source to the VS Code URI
            iframe.src = link.href;

            // Clean up the iframe after a short delay
            setTimeout(() => {
                try {
                    document.body.removeChild(iframe);
                } catch (e) {
                    console.warn("Could not remove temporary iframe:", e);
                }
            }, 500);
        }
    });

    // File path checkbox change logic
    chatContainer.addEventListener('change', (event) => {
        const target = event.target as HTMLInputElement;
        if (target.classList.contains('file-path-checkbox')) {
            const filePath = target.value;
            const isChecked = target.checked;
            onFileCheckboxChange({ filePath, isChecked }); // Call the prop function
        }
    });



  });

  // Scroll to bottom on chatHistory update
  $effect(() => {
    if (chatContainer && chatHistory) {
      if (scroll_height == 0) {
        scrollToBottom();
      } else {
        chatContainer.scrollTo({
          top: scroll_height + 100,
          behavior: 'smooth'
        });
      }
    };
  });
</script>

<header class="chat-header">
  <h2>Chat about your code</h2>
  {@render git_stuff?.()}
</header>

<div class="chat-container" bind:this={chatContainer}>
  {#each chatHistory as msg (msg.id)}
    <div class="chat-message {msg.role}-message">

      {#if msg.thoughts && msg.thoughts.length > 0}
        <details class="agent-thoughts">
          <summary>Agent Thoughts</summary>
          <ul>
            {#each msg.thoughts as thought}
              <li>{thought}</li>
            {/each}
          </ul>
        </details>
      {/if}
      
      <div class="message-content" use:highlightAction={selectedFiles}>
        {#if editingMessageId === msg.id}
            <textarea
                class="message-editor text-area-fmt"
                bind:value={msg.content}
            ></textarea>
            <div class="edit-controls">
                <button class="save-edit-btn primary" onclick={() => saveEditedMessage(msg.id, msg.content, false)}>Save</button>
                <button class="cancel-edit-btn secondary" onclick={() => toggleEditMode(msg.id)}>Cancel</button>
                <label>
                    Create new branch on edit
                    <input type="checkbox" />
                </label>
            </div>
        {:else}
            {@html formatMessage(msg.content)}
        {/if}
      </div>

      <div class="message-controls">
        {#if editingMessageId !== msg.id}
            <button class="edit-message-btn" title="Edit message" onclick={() => toggleEditMode(msg.id)}>Edit</button>
        {/if}
        <button class="hide-message-btn" title="{msg.hidden ? 'Unhide' : 'Hide'} message" data-hidden={msg.hidden} onclick={() => toggleHideMessage(msg.id, !msg.hidden)}>{msg.hidden ? 'unhide' : 'hide'}</button>
        <button class="regenerate-message-btn" title="Regenerate response" onclick={() => regenerateMessage(msg.id)}>Regenerate</button>
      </div>
      {#if branch_display_data[msg.id as string]}
        <BranchNavigation
          branchData={branch_display_data[msg.id as string]}
          projectName={project_name}
          queryId={query_id}
        />
      {/if}
    </div>
  {/each}
</div>

<div class="chat-input">
  <textarea
    bind:value={messageInput}
    placeholder="Ask a question..."
    onkeydown={(e) => e.key === 'Enter' && !e.shiftKey && handleSend()}
  ></textarea>
  <button onclick={handleSend}>Send</button>
  <div class="extra-options">
    <div class="extra-options__top">
      <button class="secondary" onclick={resetChat}>Reset Chat</button>
      <button onclick={toggleSearchModal}>Search Files</button>
      <button onclick={toggleOptimizePromptModal}>Optimize Prompt</button>
    </div>
    <div class="extra-options__bottom">
      <!-- NEW: Grounding with Search Toggle -->
      {#if project_provider === "gemini" || project_provider === "Gemini"}
          <div class="context-option">
              <label>
                  <input
                      type="checkbox"
                      bind:checked={enableGroundingWithSearch}
                      onchange={() => handleGroundingToggle(enableGroundingWithSearch)}
                  />
                  Grounding with Google Search
              </label>
          </div>
      {/if}
    </div>
  </div>
</div>