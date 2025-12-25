<!-- frontend/src/components/ChatInterface.svelte -->
<script lang="ts">
  import BranchNavigation from './BranchNavigation.svelte';
  import { formatMessage, linkFilePathsInElement } from "../lib/analyze-query/utils.js";
  import {
    highlightAction
  } from "../lib/analyze-query/syntax-highlighting.js";

  let { project_name, query_id, chatHistory, branch_display_data, git_stuff, sendMessage, resetChat, toggleSearchModal, toggleOptimizePromptModal } = $props();
  let messageInput = $state('');
  let chatContainer: HTMLElement;

  function handleSend() {
    if (messageInput.trim()) {
      sendMessage(messageInput);
    }
  }
  let prev_scroll_height = 0;
  $effect(() => {
      if (chatContainer && chatHistory) {
        if (prev_scroll_height) {
          chatContainer.scrollTop = chatContainer.scrollHeight;
          prev_scroll_height = chatContainer.scrollHeight;
        } else {
          chatContainer.scrollTo({
            top: prev_scroll_height + 100,
            behavior: 'smooth'
          });
        }
        // Link file paths in the new messages
        const messageElements = chatContainer.querySelectorAll('.message-content');
        messageElements.forEach(el => linkFilePathsInElement(el as HTMLElement));
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
      <div class="message-content" use:highlightAction>
        {@html formatMessage(msg.content)}
      </div>

      <div class="message-controls">
        <button class="edit-message-btn" title="Edit message">Edit</button>
        <button class="hide-message-btn" title="{msg.hidden ? 'Unhide' : 'Hide'} message" data-hidden={msg.hidden}>{msg.hidden ? 'unhide' : 'hide'}</button>
        <button class="regenerate-message-btn" title="Regenerate response">Regenerate</button>
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
  <button class="secondary" onclick={resetChat}>Reset Chat</button>
  <button onclick={toggleSearchModal}>Search Files</button>
  <button onclick={toggleOptimizePromptModal}>Optimize Prompt</button>
</div>