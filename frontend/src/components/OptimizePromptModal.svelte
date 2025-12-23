<!-- frontend/src/pages/OptimizePromptModal.svelte -->
<script lang="ts">
    import { createEventDispatcher } from 'svelte';

    const { project_name, queryId, initialPrompt, onClose }: { project_name: string; queryId: string; initialPrompt: string; onClose: () => void } = $props();

    let optimizationDirection = $state('');
    let includeChatHistory = $state(false);
    let includeContextFiles = $state(false);
    let optimizedPrompt = $state('');
    let isLoading = $state(false);
    let error = $state('');

    const dispatch = createEventDispatcher();

    const defaultOptimizationDirections = `
The goal is to make the queries more effective, precise, and clear.

Consider the following aspects when optimizing:
- **Clarity and Specificity:** Make the query unambiguous.
- **Keywords:** Suggest relevant programming terms, API names, design patterns, or function types.
- **Context:** If a direction is provided, incorporate it to focus the query.
- **Conciseness:** Remove unnecessary words without losing meaning.
- **Searchability:** Think about what terms would best match code files.
  `;

    async function generateOptimizedPrompt() {
        optimizedPrompt = '';
        isLoading = true;
        error = '';

        try {
            const response = await fetch('/optimize-prompt', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    project: project_name,
                    query_id: queryId,
                    original_prompt: initialPrompt,
                    optimization_direction: optimizationDirection || defaultOptimizationDirections,
                    include_chat_history: includeChatHistory,
                    include_context_files: includeContextFiles,
                }),
            });

            const data = await response.json();

            if (data.success) {
                optimizedPrompt = data.optimized_prompt;
            } else {
                error = data.error;
            }
        } catch (e) {
            console.error('Error optimizing prompt:', e);
            error = 'Network error or unexpected response.';
        } finally {
            isLoading = false;
        }
    }

    function useOptimizedPrompt() {
        // Dispatch an event to the parent component (AnalyzeQuery.svelte)
        dispatch('useOptimizedPrompt', optimizedPrompt);
        onClose(); // Close the modal
    }
</script>

<style>
    /* Copy relevant styles from static/analysis.css and static/global.css */
    .analysis-search-modal {
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

    .analysis-search-modal-content {
        background-color: #fff;
        padding: 20px;
        border-radius: 5px;
        width: 80%;
        max-width: 800px;
    }

    .modal-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 10px;
    }

    .close-search-modal {
        cursor: pointer;
        font-size: 20px;
    }

    .text-area-fmt {
        width: 100%;
        padding: 8px;
        margin-bottom: 10px;
        border: 1px solid #ccc;
        border-radius: 4px;
        resize: vertical;
    }

    .checkbox-group {
        display: flex;
        flex-direction: column;
        margin-bottom: 10px;
    }

    .checkbox-group label {
        margin-bottom: 5px;
    }

    .primary {
        background-color: #007bff;
        color: white;
        border: none;
        padding: 10px 20px;
        border-radius: 5px;
        cursor: pointer;
    }

    .primary:disabled {
        background-color: #ccc;
        cursor: not-allowed;
    }

    .secondary {
        background-color: #6c757d;
        color: white;
        border: none;
        padding: 10px 20px;
        border-radius: 5px;
        cursor: pointer;
    }

    #optimized-prompt-error {
        color: red;
        margin-bottom: 10px;
    }

    #optimized-prompt-loading {
        color: gray;
        margin-bottom: 10px;
    }
</style>

<div class="analysis-search-modal">
    <div class="analysis-search-modal-content">
        <div class="modal-header">
            <h3>Optimize Prompt</h3>
            <span role="presentation" class="close-search-modal" onclick={onClose}>&amp;times;</span>
        </div>
        
        <div class="modal-body">
            <p><strong>Original Prompt:</strong></p>
            <textarea class="text-area-fmt" rows="3" readonly value={initialPrompt}></textarea>

            <p><strong>Optimization Direction (Optional):</strong></p>
            <textarea
                class="text-area-fmt"
                rows="4"
                placeholder="e.g., Make it more concise, focus on code structure, simplify technical jargon, include specific keywords..."
                bind:value={optimizationDirection}
            ></textarea>

            <div class="checkbox-group">
                <label>
                    <input type="checkbox" bind:checked={includeChatHistory} />
                    Include Chat Conversation History
                </label>
                <label>
                    <input type="checkbox" bind:checked={includeContextFiles} />
                    Include Selected Context Files
                </label>
            </div>

            <button class="primary" onclick={generateOptimizedPrompt} disabled={isLoading}>
                {#if isLoading}
                    Generating...
                {:else}
                    Generate Optimized Prompt
                {/if}
            </button>

            {#if error}
                <div id="optimized-prompt-error">Error: {error}</div>
            {/if}

            <p><strong>Optimized Prompt:</strong></p>
            <textarea class="text-area-fmt" rows="5" readonly value={optimizedPrompt}></textarea>
        </div>

        <div class="modal-footer">
            <button class="primary" disabled={!optimizedPrompt} onclick={useOptimizedPrompt}>
                Use Optimized Prompt
            </button>
            <button class="secondary" onclick={onClose}>Close</button>
        </div>
    </div>
</div>