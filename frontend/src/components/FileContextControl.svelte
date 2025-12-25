<!-- frontend/src/components/FileContextControl.svelte -->
<script lang="ts">
  import FileList from './FileList.svelte';

  let {
    project_name,
    query_id,
    llm_suggested_files,
    relevant_files,
    otherProjectFiles,
    selectedFiles,
    file_yaml_override,
    default_use_yaml,
    include_file_descriptions, filesSelected, fetchOtherProjectFiles, includeDescriptionsToggled
  } = $props();

  // --- Event Handlers ---
  function handleFileCheckboxChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const filePath = target.value;
    const isChecked = target.checked;

    let newSelectedFiles = [...selectedFiles]; // Copy the array

    if (isChecked) {
      newSelectedFiles = [...newSelectedFiles, filePath];
    } else {
      newSelectedFiles = newSelectedFiles.filter(file => file !== filePath);
    }

    selectedFiles = newSelectedFiles; // Update local state
    filesSelected(newSelectedFiles); // Dispatch the event
  }

  function selectAllFiles(fileList: string[]) {
    const newSelectedFiles = [...fileList];
    selectedFiles = newSelectedFiles;
    filesSelected(newSelectedFiles);
  }

  function deselectAllFiles() {
    selectedFiles = [];
    filesSelected([]);
  }

  // Include Description toggle
  async function toggleIncludeDescriptions() {
    includeDescriptionsToggled(!include_file_descriptions);
  }
</script>

<h2>File context control</h2>
<div class="file-snippets">
  <label>
    <input type="checkbox" checked={include_file_descriptions} onchange={toggleIncludeDescriptions} />
    Include descriptions in prompt
  </label>

  <h3>LLM Suggested Files</h3>
  <FileList
    files={llm_suggested_files}
    selectedFiles={selectedFiles}
    project={{ file_yaml_override: file_yaml_override, default_use_yaml: default_use_yaml }}
    fileChange={handleFileCheckboxChange}
  />

  <h3>Semantic Matches - LLM suggestions</h3>
  <FileList
    files={relevant_files}
    selectedFiles={selectedFiles}
    project={{ file_yaml_override: file_yaml_override, default_use_yaml: default_use_yaml }}
    fileChange={handleFileCheckboxChange}
  />
  {#if otherProjectFiles.length > 0}
    <h3>Other Project Files</h3>
    <FileList
      files={otherProjectFiles}
      selectedFiles={selectedFiles}
      project={{ file_yaml_override: file_yaml_override, default_use_yaml: default_use_yaml }}
      fileChange={handleFileCheckboxChange}
    />
  {/if}
</div>

<style>
  .file-snippets {
    margin-bottom: 20px;
  }
</style>