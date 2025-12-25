<!-- frontend/src/pages/AnalyzeQuery.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import BranchNavigation from '../components/BranchNavigation.svelte';
  import SearchFilesModal from '../components/SearchFilesModal.svelte';
  import OptimizePromptModal from '../components/OptimizePromptModal.svelte';
  import { formatMessage, setProjectSourceDirectory, linkFilePathsInElement } from "../lib/analyze-query/utils.js";
import {
  highlightAction
} from "../lib/analyze-query/syntax-highlighting.js";

  let { extraData } = $props();

  // Destructure props from Rust (REMOVED existing_chat_history)
  let {
    project_name,
    query_id,
    query_text,
    project_source_dir,
    relevant_files,
    saved_context_files,
    llm_suggested_files,
    available_queries,
    include_file_descriptions,
    auto_commit,
    current_repo_branch_name,
    all_branches,
    git_enabled,
  } = $derived(extraData);

  // Local UI State
  let messageInput = $state('');
  let isSearchModalOpen = $state(false);
  let isOptimizePromptModalOpen = $state(false);
  let autoCommit = $state(auto_commit); // Local copy of the auto-commit state
  let includeDescriptions = $state(include_file_descriptions);
  let currentBranch = $state(current_repo_branch_name);

  let branch_display_data = $state<Record<string, any>>({}); // Initialize as an empty object with proper type

  let chatContainer: HTMLElement;
  let selectedFiles = $state([...saved_context_files]); // Convert to state variable

  // --- NEW: Add a state variable for the chat history ---
  interface ChatMessage {
    role: string;
    content: string;
    id?: string;
    hidden?: string;
  }
  let chatHistory = $state<ChatMessage[]>([]);

  // -- Derived States and Functions --
  // Update context whenever selectedFiles changes
  $effect(() => {
    const filesChanged = JSON.stringify(selectedFiles) !== JSON.stringify(saved_context_files);
    if (filesChanged || includeDescriptions !== include_file_descriptions) updateContext(selectedFiles, includeDescriptions);
  });

  async function updateContext(files: string[], includeDescriptions: boolean) {
    try {
      const response = await fetch('/update-analysis-context', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          project_name: project_name,
          query_id: query_id,
          files: files,
          include_descriptions: includeDescriptions,
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      console.log('Context updated:', data);
    } catch (error) {
      console.error('Error updating context:', error);
      if (typeof error === 'object' && error !== null && 'message' in error) alert(`Error updating context: ${error.message}`);
    }
  }

  // Chat Interface Functions
  async function handleSend() {
    if (!messageInput.trim()) return;

    try {
      const response = await fetch('/llm/chat_analysis/chat_analysis', { // Corrected path
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          project: project_name,
          query_id: query_id,
          message: messageInput,
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const newMessage = await response.json();
      chatHistory = [...chatHistory, newMessage.user_message, newMessage.model_message]; // Update chat history
      messageInput = ''; // Clear input
    } catch (error) {
      console.error('Error sending message:', error);
      if (typeof error === 'object' && error !== null && 'message' in error) alert(`Error sending message: ${error.message}`);
    }
  }

  async function resetChat() {
    if (confirm('Are you sure you want to reset the chat?')) {
      try {
        const response = await fetch(`/llm/chat_analysis/reset_analysis_chat?project_name=${project_name}&query_id=${query_id}`, {
          method: 'POST',
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

  // Modal Toggle Functions
  function toggleSearchModal() {
    isSearchModalOpen = !isSearchModalOpen;
  }

  function toggleOptimizePromptModal() {
    isOptimizePromptModalOpen = !isOptimizePromptModalOpen;
  }

  // Auto-Commit Toggle Function
  async function toggleAutoCommit() {
    autoCommit = !autoCommit; // Flip the local state immediately

    try {
      const response = await fetch(`/query/update_auto_commit?project_name=${project_name}&query_id=${query_id}&auto_commit=${autoCommit}`, {
        method: 'POST',
      });

      if (!response.ok) {
        autoCommit = !autoCommit; // Revert on error
        throw new Error(`Failed to update auto-commit: ${response.statusText}`);
      }
    } catch (error) {
      autoCommit = !autoCommit; // Revert on error
      console.error('Error updating auto-commit:', error);
      if (typeof error === 'object' && error !== null && 'message' in error) alert(`Error updating auto-commit: ${error.message}`);
    }
  }

    // Include Description toggle
    async function toggleIncludeDescriptions() {
        includeDescriptions = !includeDescriptions;
        updateContext(selectedFiles, includeDescriptions); // Directly trigger context update
    }

  // Git actions
    async function commitChanges() {
        // Implement your commit logic here, using fetch to call the backend.
        console.log('Commit changes clicked');
    }

    async function pushChanges() {
        // Implement your push logic here, using fetch to call the backend.
        console.log('Push changes clicked');
    }

  // Branch selecting
    async function onBranchChange(event: Event) {
        if (!(event.target instanceof HTMLSelectElement)) return;
        const newBranch = event.target.value;
        console.log("switching to " + newBranch);
    }

  // --- NEW: Fetch chat history on component mount ---
  onMount(async () => {
    try {
      const response = await fetch(`/${project_name}/${query_id}/chat_history`);
      if (!response.ok) {
        throw new Error(`Failed to fetch chat history: ${response.statusText}`);
      }
      const data = await response.json();
      chatHistory = data.history;
    } catch (error) {
      console.error('Error fetching chat history:', error);
      if (typeof error === 'object' && error !== null && 'message' in error) alert(`Error fetching chat history: ${error.message}`);
    }
    if (chatContainer) chatContainer.scrollTop = chatContainer.scrollHeight;
    try {
      const response = await fetch(`/get-branching-data?project_name=${project_name}&query_id=${query_id}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch branching data: ${response.statusText}`);
      }
      branch_display_data = await response.json();
    } catch (error) {
      console.error('Error fetching branching data:', error);
      if (typeof error === 'object' && error !== null && 'message' in error) alert(`Error fetching branching data: ${error.message}`);
    }
  });

  function switchQuery(e: Event) {
    const target = e.target as HTMLSelectElement;
    window.location.href = `/analyze-query/${project_name}/${target.value}`;
  }

  function handleFileCheckboxChange(event: Event) {
        const target = event.target as HTMLInputElement;
        const filePath = target.value;
        const isChecked = target.checked;

        if (isChecked) {
            selectedFiles = [...selectedFiles, filePath];
        } else {
            selectedFiles = selectedFiles.filter(file => file !== filePath);
        }
    }

    function selectAllFiles(fileList: string[]) {
        selectedFiles = [...fileList]; // Select all files
    }

    function deselectAllFiles() {
        selectedFiles = []; // Deselect all files
    }

</script>

<svelte:head>
  <title>Analyze Query - {project_name}</title>
  <link rel="stylesheet" href="/static/analysis.css">
  <link rel="stylesheet" href="/static/global.css">
</svelte:head>

<style>
  /* You can import your existing CSS or move it here */
  /* @import '/static/analysis.css';
  @import '/static/global.css'; */

  .analysis-layout {
    display: grid;
    grid-template-columns: 350px 1fr;
    gap: 20px;
    height: 90vh;
  }
  .sidebar {
    overflow-y: auto;
    padding: 10px;
    background: #f9f9f9;
  }
  .chat-area {
    display: flex;
    flex-direction: column;
  }
  .messages {
    flex-grow: 1;
    overflow-y: auto;
    border: 1px solid #ccc;
    padding: 10px;
    margin-bottom: 10px;
  }
</style>

<div class="analysis-layout">
  <!-- Sidebar: Files and Settings -->
  <aside class="editable-query">
    <p><strong>Project:</strong> {project_name}</p>

    <div class="query-selector">
      <label for="query-select"><strong>Select Query:</strong></label>
      <select onchange={switchQuery} value={query_id}>
        {#each available_queries as [id, title]}
          <option value={id}>{title}</option>
        {/each}
      </select>
    </div>

    <hr />
    <label>
      <input type="checkbox" checked={includeDescriptions} onchange={toggleIncludeDescriptions} />
      Include descriptions in prompt
    </label>

    {#if llm_suggested_files.length > 0}
        <h4>LLM Suggested</h4>
        <ul>
            {#each llm_suggested_files as file}
                <li>
                    <label>
                        <input
                            type="checkbox"
                            value={file}
                            checked={selectedFiles.includes(file)}
                            onchange={handleFileCheckboxChange}
                        />
                        {file.split('/').pop()}
                    </label>
                </li>
            {/each}
        </ul>
    {/if}


    <h4>Relevant Files</h4>
    <ul>
      {#each relevant_files as file}
        <li>
          <label>
            <input
              type="checkbox"
              value={file}
              checked={selectedFiles.includes(file)}
              onchange={handleFileCheckboxChange}
            />
            {file.split('/').pop()}
          </label>
        </li>
      {/each}
    </ul>

    {#if relevant_files.length > 0}
      <button onclick={() => selectAllFiles(relevant_files)}>Select All Relevant</button>
      <button onclick={deselectAllFiles}>Deselect All</button>
    {/if}
  </aside>

  <!-- Main: Chat Interface -->
  <main class="chat-area">
    <header class="chat-header">
      <h2>Chat about your code</h2>
      {#if git_enabled}
        <div class="git-status">
          Branch:
          <select bind:value={currentBranch} onchange={onBranchChange}>
            {#each all_branches as branch}
              <option value={branch}>{branch}</option>
            {/each}
          </select>
        </div>

        <div>
            <label>
                Auto-Commit:
                <input type="checkbox" bind:checked={autoCommit} onchange={toggleAutoCommit} />
            </label>
            <button onclick={commitChanges}>Commit</button>
            <button onclick={pushChanges}>Push</button>
        </div>
      {/if}

    </header>

    <div class="chat-container" bind:this={chatContainer}>
      {#each chatHistory as msg (msg.id)}
        <div class="chat-message {msg.role}-message">
          <!-- <strong>{msg.role}:</strong> -->
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
  </main>
</div>

{#if isSearchModalOpen}
  <SearchFilesModal {project_name} onClose={() => (isSearchModalOpen = false)} />
{/if}

{#if isOptimizePromptModalOpen}
  <OptimizePromptModal {project_name} queryId={query_id} initialPrompt={query_text} onClose={() => (isOptimizePromptModalOpen = false)} />
{/if}