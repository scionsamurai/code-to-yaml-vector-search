<!-- frontend/src/components/GitActions.svelte -->
<script lang="ts">
  let { project_name, query_id, auto_commit, currentBranch, all_branches, autoCommitToggled, commitChanges, pushChanges, branchChanged } = $props();

    // Branch selecting
    async function onBranchChange(event: Event) {
        if (!(event.target instanceof HTMLSelectElement)) return;
        const newBranch = event.target.value;
        console.log("child switching to " + newBranch);
        branchChanged(newBranch);
    }

  async function toggleAutoCommit() {
    autoCommitToggled(!auto_commit);
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
    <label>
        <strong>Auto-Commit for this chat:</strong>
        <input type="checkbox" bind:checked={auto_commit} onchange={toggleAutoCommit} />
    </label>
    <button onclick={commitChanges}>Commit</button>
    <button onclick={pushChanges}>Push</button>
</div>