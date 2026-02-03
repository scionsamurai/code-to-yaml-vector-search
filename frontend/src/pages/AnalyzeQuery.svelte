<!-- frontend/src/pages/AnalyzeQuery.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import FileContextControl from '../components/FileContextControl.svelte';
  import ChatInterface from '../components/ChatInterface.svelte';
  import GitActions from '../components/GitActions.svelte';
  import SearchFilesModal from '../components/SearchFilesModal.svelte';
  import OptimizePromptModal from '../components/OptimizePromptModal.svelte';
  import QueryManagement from '../components/QueryManagement.svelte';
  import { setProjectSourceDirectory } from "../lib/analyze-query/utils.js";
  import { 
    updateContext, fetchChatHistory, fetchOtherProjectFiles, fetchBranchingData
  } from '../lib/analyze-query/api.js';


  interface ChatMessage {
    role: string;
    content: string;
    id?: string;
    hidden?: string;
  }

  let { extraData } = $props();

  let { 
    project_name, query_id, query_text, project_source_dir, relevant_files, saved_context_files, 
    llm_suggested_files, available_queries, include_file_descriptions, auto_commit, current_repo_branch_name, 
    all_branches, git_enabled, file_yaml_override, default_use_yaml,
    grounding_with_search,
    project_provider,
    agentic_mode_enabled: initialAgenticModeEnabled
  } = $derived(extraData);

  // --- Local UI State ---
  let messageInput = $state('');
  let isSearchModalOpen = $state(false);
  let isOptimizePromptModalOpen = $state(false);
  let includeDescriptions = $state(include_file_descriptions);
  let currentAgenticModeEnabled = $state(initialAgenticModeEnabled);
  // This local state will now be updated by ChatInterface and then trigger the effect
  let currentGroundingWithSearch = $state(grounding_with_search); // MODIFIED: Use `currentGroundingWithSearch` to track UI state.
  let branch_display_data = $state<Record<string, any>>({}); // Initialize as an empty object with proper type

  
  let selectedFiles = $state([...saved_context_files]);
  let otherProjectFiles = $state<string[]>([]);
  let chatHistory = $state<ChatMessage[]>([]);

  // --- Lifecycle Hooks ---
  onMount(async () => {
    setProjectSourceDirectory(project_source_dir);
    chatHistory = await fetchChatHistory(project_name, query_id);
    otherProjectFiles = await fetchOtherProjectFiles(project_name, llm_suggested_files, relevant_files);
    branch_display_data = await fetchBranchingData(project_name, query_id);
  });

  // --- Derived States and Functions ---
  // Update context whenever selectedFiles, includeDescriptions, or currentGroundingWithSearch changes
  $effect(() => {
    const filesChanged = JSON.stringify(selectedFiles) !== JSON.stringify(saved_context_files);
    const descriptionsChanged = includeDescriptions !== include_file_descriptions;
    const groundingChanged = currentGroundingWithSearch !== grounding_with_search; // MODIFIED: Compare with `currentGroundingWithSearch`

    if (filesChanged || descriptionsChanged || groundingChanged) {
        updateContext(project_name, query_id, selectedFiles, includeDescriptions, currentGroundingWithSearch); // MODIFIED: Pass `currentGroundingWithSearch`
    }
  });

    $effect(() => {
      if (currentAgenticModeEnabled !== initialAgenticModeEnabled) {
        // Call API to update agentic_mode_enabled
        updateAgenticMode(project_name, query_id, currentAgenticModeEnabled);
      }
  });

  async function updateAgenticMode(projectName: string, queryId: string, enabled: boolean) {
    try {
      const response = await fetch('/llm/chat_analysis/update_agentic_mode', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ project: projectName, query_id: queryId, enabled }),
      });
      if (!response.ok) throw new Error('Failed to update agentic mode');
      initialAgenticModeEnabled = enabled;
      // Optionally refresh or show success message
    } catch (error) {
      console.error('Error updating agentic mode:', error);
      currentAgenticModeEnabled = !enabled; // Revert UI state on error
    }
  }

  function switchQuery(e: Event) {
    const target = e.target as HTMLSelectElement;
    window.location.href = `/${project_name}/${target.value}`;
  }

  // --- Event Handlers ---
  const handleFileSelectionChange = (newSelectedFiles: string[]) => {
    selectedFiles = newSelectedFiles;
  };

  const handleChatFileCheckboxChange = (value: { filePath: string; isChecked: boolean }) => {
    const { filePath, isChecked } = value;
    let newSelectedFiles = [...selectedFiles];

    if (isChecked) {
      if (!newSelectedFiles.includes(filePath)) {
        newSelectedFiles.push(filePath);
      }
    } else {
      newSelectedFiles = newSelectedFiles.filter(file => file !== filePath);
    }
    selectedFiles = newSelectedFiles; // This will trigger the $effect to update context
  };


  async function handleSendMessage(message: string) {
    messageInput = ''; // Clear input immediately
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
        if (typeof error === 'object' && error !== null && 'message' in error) alert(`Error sending message: ${error.message}`);
    }
  }

  async function handleResetChat() {
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

  const handleIncludeDescriptionsToggle = async (newValue: boolean) => {
    includeDescriptions = newValue;
    // The $effect will call updateContext with the new value
  };

  // NEW: Handler for grounding toggle, now receiving from ChatInterface
  const handleGroundingToggle = async (newValue: boolean) => {
    currentGroundingWithSearch = newValue;
    // The $effect will call updateContext with the new value
  };

  const handleOtherFilesFetch = async () => {
    otherProjectFiles = await fetchOtherProjectFiles(project_name, llm_suggested_files, relevant_files);
  };
  // Modal Toggle Functions
  function toggleSearchModal() {
    isSearchModalOpen = !isSearchModalOpen;
  }

  function toggleOptimizePromptModal() {
    isOptimizePromptModalOpen = !isOptimizePromptModalOpen;
  }
</script>

<svelte:head>
  <title>Analyze Query - {project_name}</title>
  <link rel="stylesheet" href="/static/analysis.css">
  <link rel="stylesheet" href="/static/global.css">
</svelte:head>

<div class="analysis-container">
  <aside class="editable-query">
    <p><strong>Project:</strong> {project_name}</p>

    <QueryManagement
      project_name={project_name}
      query_id={query_id}
      initialQuery={query_text}
      available_queries={available_queries}
    />

    <div class="agentic-control">
      <label>
        <input type="checkbox" bind:checked={currentAgenticModeEnabled} />
        Enable Agentic Control
      </label>
    </div>
    {#if !currentAgenticModeEnabled}
      <FileContextControl
        {project_name}
        {query_id}
        {llm_suggested_files}
        {relevant_files}
        {otherProjectFiles}
        {selectedFiles}
        {file_yaml_override}
        {default_use_yaml}
        {include_file_descriptions}
        updatefilesSelected={(e: any) => handleFileSelectionChange(e)}
        fetchOtherProjectFiles={handleOtherFilesFetch}
        includeDescriptionsToggled={(e: any) => handleIncludeDescriptionsToggle(e)}
      />
    {/if}

    <!-- REMOVED: Grounding with Search Toggle from here -->
  </aside>

  <main class="chat-interface">
    <ChatInterface
      {project_name}
      {query_id}
      {chatHistory}
      {branch_display_data}
      {selectedFiles}
      {project_provider}
      initialGroundingWithSearch={currentGroundingWithSearch}
      sendMessage={(e: CustomEvent<string>) => handleSendMessage(e.detail)}
      resetChat={handleResetChat}
      toggleSearchModal={toggleSearchModal}
      toggleOptimizePromptModal={toggleOptimizePromptModal}
      onFileCheckboxChange={handleChatFileCheckboxChange}
      onGroundingToggle={handleGroundingToggle}
    >
      {#snippet git_stuff()}
        {#if git_enabled}
            <GitActions
                {project_name}
                {query_id}
                initialAutoCommit={auto_commit}
                initialBranch={current_repo_branch_name}
                {all_branches}
            />
        {/if}
      {/snippet}
    </ChatInterface>
  </main>
</div>


{#if isSearchModalOpen}
  <SearchFilesModal {project_name} onClose={() => (isSearchModalOpen = false)} />
{/if}

{#if isOptimizePromptModalOpen}
  <OptimizePromptModal {project_name} queryId={query_id} initialPrompt={query_text} onClose={() => (isOptimizePromptModalOpen = false)} />
{/if}

<div class="actions" style="position: absolute;bottom: 0;">
  <a href="/projects/{project_name}" class="button">Back to Project</a>
</div>