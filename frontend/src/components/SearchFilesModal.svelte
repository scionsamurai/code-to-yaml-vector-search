<!-- frontend/src/components/SearchFilesModal.svelte -->
<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte';

    export let project_name: string;
    export let onClose: () => void;

    let searchResults: string = "";
    let searchQuery: string = "";
    let isLoading: boolean = false;

    const dispatch = createEventDispatcher();

    async function searchFiles() {
        isLoading = true;
        searchResults = "";
        try {
            const response = await fetch("/search-related-files", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    project: project_name,
                    query: searchQuery,
                }),
            });

            const data = await response.json();

            if (data.success) {
                searchResults = data.html;
            } else {
                searchResults = `<p>Error: ${data.error}</p>`;
            }
        } catch (error) {
            if (typeof error === 'object' && error !== null && 'message' in error) {
                searchResults = `<p>Error: ${error.message}</p>`;
            } else {
                searchResults = `<p>Error: An unknown error occurred.</p>`;
            }
        } finally {
            isLoading = false;
        }
    }

    onMount(() => {
        // Focus on the search input when the modal opens
        const searchInput = document.getElementById("search-query") as HTMLInputElement;
        if (searchInput) {
            searchInput.focus();
        }
    });

    function closeModal() {
        dispatch('close');
        onClose();
    }
</script>

<style>
    /* Basic Modal Styles - you can customize these */
    .modal-overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 1000;
    }

    .modal {
        background: white;
        padding: 20px;
        border-radius: 8px;
        width: 80%;
        max-width: 800px;
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
    }

    .modal-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 10px;
    }

    .modal-header h2 {
        margin: 0;
    }

    .close-button {
        background: none;
        border: none;
        font-size: 20px;
        cursor: pointer;
    }

    .search-input {
        width: 100%;
        padding: 8px;
        margin-bottom: 10px;
        border: 1px solid #ccc;
        border-radius: 4px;
    }

    .search-button {
        background-color: #007bff;
        color: white;
        border: none;
        padding: 8px 12px;
        border-radius: 4px;
        cursor: pointer;
    }

    .results {
        margin-top: 20px;
    }

    .loading {
        text-align: center;
        font-style: italic;
        color: gray;
    }
</style>

<div class="modal-overlay" on:click={closeModal} role="presentation">
    <div class="modal">
        <div class="modal-header">
            <h2>Search Files</h2>
            <button class="close-button" on:click={closeModal}>Ã—</button>
        </div>

        <input
            type="text"
            id="search-query"
            class="search-input"
            placeholder="Enter search query"
            bind:value={searchQuery}
        />
        <button class="search-button" on:click={searchFiles}>Search</button>

        <div class="results">
            {#if isLoading}
                <p class="loading">Loading...</p>
            {:else}
                {@html searchResults}
            {/if}
        </div>
    </div>
</div>