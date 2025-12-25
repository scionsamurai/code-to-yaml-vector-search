<!-- frontend/src/components/FileList.svelte -->
<script lang="ts">
    let { files, selectedFiles, project, fileChange } = $props();

    function handleFileCheckboxChange(event: Event) {
        const target = event.target as HTMLInputElement;
        const filePath = target.value;
        const isChecked = target.checked;

        // Dispatch an event to notify the parent component
        const detail = { filePath, isChecked };
        fileChange(detail);
    }
</script>

<style>
    /*  CSS rules to match the original look and feel */
    /*  You'll need to inspect the static/analysis.css file
        to copy the relevant styles here */
    .file-item {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 5px;
        border-bottom: 1px solid #eee;
    }

    .left {
        display: flex;
        align-items: center;
    }

    .right {
        margin-left: auto;
    }
</style>

<div class="file-list">
{#each files as file}
    <div class="file-item">
        <span class="left">
            <input
                type="checkbox"
                class="file-checkbox"
                value={file}
                checked={selectedFiles.includes(file)}
                onchange={handleFileCheckboxChange}
            />
            <span>{file}</span>
        </span>
        {#if !file.endsWith(".md")}
            <span class="right">
                <input
                    type="checkbox"
                    class="yaml-checkbox"
                    value={file}
                    checked={project.file_yaml_override[file] ?? project.default_use_yaml}
                />
                YAML
            </span>
        {/if}
    </div>
{/each}
</div>