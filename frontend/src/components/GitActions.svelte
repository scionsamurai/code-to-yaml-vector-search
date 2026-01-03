<!-- frontend/src/components/GitActions.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { toggleAutoCommitBackend, fetchGitStatus } from '../lib/analyze-query/api.js';

  let { project_name, query_id, initialAutoCommit, initialBranch, all_branches } = $props();

  let autoCommit = $state(initialAutoCommit);
  let currentBranch = $state(initialBranch);
  let hasUnpushed = $state(false);
  let hasUncommittedChanges = $state(false);


  onMount(async () => {
    await updateGitStatus();
  });

  async function onBranchChange(event: Event) {
        if (!(event.target instanceof HTMLSelectElement)) return;
        const newBranch = event.target.value;
        currentBranch = newBranch;
        console.log("child switching to " + newBranch);
    }

  async function toggleAutoCommit() {
    autoCommit = !autoCommit;
    try {
      await toggleAutoCommitBackend(project_name, query_id, autoCommit);
    } catch (error) {
      console.error('Error updating auto-commit:', error);
      autoCommit = !autoCommit; // Revert on error
      if (typeof error === 'object' && error !== null && 'message' in error) alert(`Error updating auto-commit: ${error.message}`);
    }
  }

  async function handleCreateBranch() {
    const branchName = prompt('Enter branch name:');
    if (branchName) {
      try {
        const response = await fetch('/create-git-branch', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ project_name: project_name, branch_name: branchName }),
        });

        const createBranchData = await response.json();
        if (createBranchData.success) {
            alert(createBranchData.message);
            window.location.reload();
        } else {
          alert('Failed to create branch: ' + createBranchData.message);
        }
      } catch (error) {
        console.error('Error creating branch:', error);
        alert('Error creating branch.');
      }
    }
  }

  async function handleCommitChanges() {
    const commitMessage = prompt('Enter commit message:');
    if (!commitMessage) {
        alert('Commit cancelled.');
        return;
    }

    try {
        const response = await fetch('/commit-changes', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                project_name: project_name,
                message: commitMessage,
                query_id: query_id // <--- ADDED: Include queryId in the commit request
            }),
        });

        const data = await response.json();
        if (data.success) {
            alert(data.message);
            await updateGitStatus(); // ADDED updateGitStatus
        } else {
            alert('Commit failed: ' + data.message);
            await updateGitStatus(); // ADDED updateGitStatus
        }
    } catch (error) {
        console.error('Error committing changes:', error);
        alert('Error committing changes.');
        await updateGitStatus(); // ADDED updateGitStatus
    }
  };

  async function handlePushChanges() {
      if (!confirm(`Are you sure you want to push changes for branch '${currentBranch}' to remote?`)) {
          alert('Push cancelled.');
          return;
      }

      try {
          const response = await fetch('/push-git-changes', {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({ project_name: project_name }),
          });

          const data = await response.json();
          if (data.success) {
              alert(data.message);
              await updateGitStatus(); // ADDED updateGitStatus
          } else {
              alert('Push failed: ' + data.message);
              await updateGitStatus(); // ADDED updateGitStatus
          }
      } catch (error) {
          console.error('Error pushing changes:', error);
          alert('Error pushing changes.');
          await updateGitStatus(); // ADDED updateGitStatus
      }
  };
  async function updateGitStatus() {
    try {
      const status = await fetchGitStatus(project_name);
      hasUnpushed = status.has_unpushed_commits;
      hasUncommittedChanges = status.has_uncommitted_changes;
    } catch (error) {
      console.error('Error fetching git status', error);
    }
  }
</script>

<div class="git-status">
    <strong>Current Repo Branch:</strong>
    <select bind:value={currentBranch} onchange={onBranchChange}>
        {#each all_branches as branch}
            <option value={branch}>{branch}</option>
        {/each}
    </select>
</div>

<div>
    {#if hasUncommittedChanges}
      <button onclick={handleCommitChanges}>Commit</button>
    {/if}
    {#if hasUnpushed}
      <button onclick={handlePushChanges}>Push</button>
    {/if}
    <label>
        <strong>Auto-Commit for this chat:</strong>
        <input type="checkbox" bind:checked={autoCommit} onchange={toggleAutoCommit} />
    </label>
    <button onclick={handleCreateBranch}>Start New Branch</button>
</div>