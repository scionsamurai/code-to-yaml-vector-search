<!-- BranchNavigation.svelte -->
<script>
  export let branchData; // BranchDisplayData for this branching point
  export let projectName;
  export let queryId;

  async function switchBranch(newCurrentNodeId) {
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
</script>

<div class="branch-navigation">
  {#if branchData.total_siblings > 1}
    {#if branchData.current_index > 0}
      <button
        class="branch-nav-btn"
        data-nav-target-id={branchData.sibling_ids[branchData.current_index - 1]}
        data-nav-direction="prev"
        title="Previous Branch"
        onclick={() => switchBranch(branchData.sibling_ids[branchData.current_index - 1])}
      >
        &larr;
      </button>
    {:else}
      <button class="branch-nav-btn disabled" disabled title="No Previous Branch">&larr;</button>
    {/if}

    <span>{branchData.current_index + 1} of {branchData.total_siblings}</span>

    {#if branchData.current_index < branchData.total_siblings - 1}
      <button
        class="branch-nav-btn"
        data-nav-target-id={branchData.sibling_ids[branchData.current_index + 1]}
        data-nav-direction="next"
        title="Next Branch"
        onclick={() => switchBranch(branchData.sibling_ids[branchData.current_index + 1])}
      >
        &rarr;
      </button>
    {:else}
      <button class="branch-nav-btn disabled" disabled title="No Next Branch">&rarr;</button>
    {/if}
  {/if}
</div>

<style>
  .branch-navigation {
    display: flex;
    align-items: center;
    gap: 0.5em;
  }

  .branch-nav-btn {
    /* Add your styling for the buttons here */
    padding: 0.25em 0.5em;
    border: 1px solid #ccc;
    background-color: #f0f0f0;
    cursor: pointer;
  }

  .branch-nav-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>