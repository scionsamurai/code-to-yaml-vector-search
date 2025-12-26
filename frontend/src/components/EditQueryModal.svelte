<!-- frontend/src/components/EditQueryModal.svelte -->
<script lang="ts">
    let { isOpen, query, onClose, onUpdate, project_name, query_id } = $props();
    let newQuery = $state(query);
</script>

{#if isOpen}
    <div class="modal">
        <div class="modal-content">
            <span role="presentation" class="close-modal" onclick={onClose}>&times;</span>
            <h2>Edit Query</h2>
            <textarea bind:value={newQuery}></textarea>
            <button onclick={() => onUpdate(newQuery)}>Update Query</button>
        </div>
    </div>
{/if}

<style>
    /* Basic modal styles */
    .modal {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
    }

    .modal-content {
        background-color: white;
        padding: 20px;
        border-radius: 5px;
        position: relative; /* For close button positioning */
    }

    .close-modal {
        position: absolute;
        top: 10px;
        right: 10px;
        cursor: pointer;
    }
    textarea {
        width: 400px;
        height: 200px;
    }
</style>