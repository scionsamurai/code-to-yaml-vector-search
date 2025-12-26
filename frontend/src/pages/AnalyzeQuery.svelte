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
    updateContext, fetchChatHistory, fetchOtherProjectFiles, fetchBranchingData, toggleAutoCommitBackend
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
  } = $derived(extraData);

  // --- Local UI State ---
  let messageInput = $state('');
  let isSearchModalOpen = $state(false);
  let isOptimizePromptModalOpen = $state(false);
  let autoCommit = $state(auto_commit); // Local copy of the auto-commit state
  let includeDescriptions = $state(include_file_descriptions);
  let currentBranch = $state(current_repo_branch_name);
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
  // Update context whenever selectedFiles changes
  $effect(() => {
    const filesChanged = JSON.stringify(selectedFiles) !== JSON.stringify(saved_context_files);
    if (filesChanged || includeDescriptions !== include_file_descriptions) updateContext(project_name, query_id, selectedFiles, includeDescriptions);
  });

  function switchQuery(e: Event) {
    const target = e.target as HTMLSelectElement;
    window.location.href = `/${project_name}/${target.value}`;
  }

  // --- Event Handlers ---
  const handleFileSelectionChange = (newSelectedFiles: string[]) => {
    selectedFiles = newSelectedFiles;
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
  const handleAutoCommitToggle = async (newValue: boolean) => {
        autoCommit = newValue;
        try {
          await toggleAutoCommitBackend(project_name, query_id, newValue);
        } catch (error) {
          console.error('Error updating auto-commit:', error);
          autoCommit = !newValue;
          if (typeof error === 'object' && error !== null && 'message' in error) alert(`Error updating auto-commit: ${error.message}`);
        }
    };

  const handleIncludeDescriptionsToggle = async (newValue: boolean) => {
    includeDescriptions = newValue;
    await updateContext(project_name, query_id, selectedFiles, newValue);
  };
  const handleBranchChange = (newBranch: string) => {
      currentBranch = newBranch;
      console.log("Parent switching to " + newBranch);
  }

  const handleCommitChanges = () => {
    console.log('Commit changes clicked');
  };

  const handlePushChanges = () => {
    console.log('Push changes clicked');
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

    <hr />
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
  </aside>

  <main class="chat-interface">
    <ChatInterface
      {project_name}
      {query_id}
      {chatHistory}
      {branch_display_data}
      sendMessage={(e: CustomEvent<string>) => handleSendMessage(e.detail)}
      resetChat={handleResetChat}
      toggleSearchModal={toggleSearchModal}
      toggleOptimizePromptModal={toggleOptimizePromptModal}
    >
      {#snippet git_stuff()}
        {#if git_enabled}
            <GitActions
                {project_name}
                {query_id}
                {auto_commit}
                {currentBranch}
                {all_branches}
                autoCommitToggled={(e: CustomEvent<boolean>) => handleAutoCommitToggle(e.detail)}
                branchChanged={(e: CustomEvent<string>) => handleBranchChange(e.detail)}
                commitChanges={handleCommitChanges}
                pushChanges={handlePushChanges}
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

<div class="actions">
  <a href="/projects/{project_name}" class="button">Back to Project</a>
</div>
