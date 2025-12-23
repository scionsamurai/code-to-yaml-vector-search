<!-- frontend/src/pages/AnalyzeQuery.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import SearchFilesModal from '../components/SearchFilesModal.svelte';
  import OptimizePromptModal from '../components/OptimizePromptModal.svelte';

  let { extraData } = $props();

  // Destructure props from Rust
  let {
    project_name,
    query_id,
    query_text,
    project_source_dir,
    relevant_files,
    saved_context_files,
    llm_suggested_files,
    existing_chat_history,
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

  let chatContainer: HTMLElement;
  let selectedFiles = $state([...saved_context_files]); // Convert to state variable

  // -- Derived States and Functions --
  // Update context whenever selectedFiles changes
  $effect(() => {
    updateContext(selectedFiles, includeDescriptions);
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
          project_name: project_name,
          query_id: query_id,
          message: messageInput,
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const newMessage = await response.json();
      existing_chat_history = [...existing_chat_history, newMessage]; // Update chat history
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

  onMount(() => {
    if (chatContainer) chatContainer.scrollTop = chatContainer.scrollHeight;
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

<style>
  /* You can import your existing CSS or move it here */
  @import '/static/analysis.css';
  @import '/static/global.css';

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
  <aside class="sidebar">
    <p><strong>Project:</strong> {project_name}</p>

    <div class="query-selector">
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
      <h2>Chat</h2>
      {#if git_enabled}
        <div class="git-status">
          Branch:
          <select bind:value={currentBranch} onchange={onBranchChange}>
            {#each all_branches as branch}
              <option value={branch}>{branch}</option>
            {/each}
          </select>
        </div>
      {/if}
        <button onclick={toggleSearchModal}>Search Files</button>
        <button onclick={toggleOptimizePromptModal}>Optimize Prompt</button>
    </header>

    <div class="messages" bind:this={chatContainer}>
      {#each existing_chat_history as msg}
        <div class="message {msg.role}-message">
          <strong>{msg.role}:</strong>
          <div class="content">
            <!-- You would use a markdown/syntax highlighter component here -->
            {msg.content}
          </div>
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
      <button onclick={resetChat}>Reset Chat</button>

        {#if git_enabled}
            <div>
                <label>
                    Auto-Commit:
                    <input type="checkbox" bind:checked={autoCommit} onchange={toggleAutoCommit} />
                </label>
                <button onclick={commitChanges}>Commit</button>
                <button onclick={pushChanges}>Push</button>
            </div>
        {/if}
    </div>
  </main>
</div>

{#if isSearchModalOpen}
  <SearchFilesModal {project_name} onClose={() => (isSearchModalOpen = false)} />
{/if}

{#if isOptimizePromptModalOpen}
  <OptimizePromptModal {project_name} queryId={query_id} initialPrompt={query_text} onClose={() => (isOptimizePromptModalOpen = false)} />
{/if}