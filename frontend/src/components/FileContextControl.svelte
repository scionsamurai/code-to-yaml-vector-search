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
    include_file_descriptions, updatefilesSelected, fetchOtherProjectFiles, includeDescriptionsToggled
  } = $props();

  // --- Event Handlers ---
  function handleFileCheckboxChange(value: { filePath: string; isChecked: boolean }) {
    const filePath = value.filePath;
    const isChecked = value.isChecked;

    let newSelectedFiles = [...selectedFiles]; // Copy the array

    if (isChecked) {
      newSelectedFiles = [...newSelectedFiles, filePath];
    } else {
      newSelectedFiles = newSelectedFiles.filter(file => file !== filePath);
    }

    selectedFiles = newSelectedFiles; // Update local state
    updatefilesSelected(newSelectedFiles); // Dispatch the event
  }

  // New function to toggle selection for a given list of files
  function toggleFilesSelection(filesToToggle: string[]) {
    // Check if ALL files in filesToToggle are currently present in selectedFiles
    const allCurrentFilesSelected = filesToToggle.every(file => selectedFiles.includes(file));
    let newSelectedFiles = [...selectedFiles];

    if (allCurrentFilesSelected) {
      // If all are currently selected, deselect them all from newSelectedFiles
      newSelectedFiles = newSelectedFiles.filter(file => !filesToToggle.includes(file));
    } else {
      // If not all are selected (or none), add all filesToToggle to newSelectedFiles
      for (const file of filesToToggle) {
        if (!newSelectedFiles.includes(file)) { // Avoid duplicates
          newSelectedFiles.push(file);
        }
      }
    }
    
    selectedFiles = newSelectedFiles; // Update local state
    updatefilesSelected(newSelectedFiles); // Dispatch the event to parent
  }

  // Include Description toggle
  async function toggleIncludeDescriptions() {
    includeDescriptionsToggled(!include_file_descriptions);
  }
</script>

<h2 style="margin: 0; text-align: center;">File context control</h2>
<div class="file-snippets">
  <label>
    <input type="checkbox" checked={include_file_descriptions} onchange={toggleIncludeDescriptions} />
    Include descriptions in prompt
  </label>

  <h3>
    LLM Suggested Files
    <button class="small-button" onclick={() => toggleFilesSelection(llm_suggested_files)}>
      Toggle All
    </button>
  </h3>
  <FileList
    files={llm_suggested_files}
    selectedFiles={selectedFiles}
    project={{ file_yaml_override: file_yaml_override, default_use_yaml: default_use_yaml }}
    fileChange={handleFileCheckboxChange}
  />

  <h3>
    Semantic Matches - LLM suggestions
    <button class="small-button" onclick={() => toggleFilesSelection(relevant_files)}>
      Toggle All
    </button>
  </h3>
  <FileList
    files={relevant_files}
    selectedFiles={selectedFiles}
    project={{ file_yaml_override: file_yaml_override, default_use_yaml: default_use_yaml }}
    fileChange={handleFileCheckboxChange}
  />
  {#if otherProjectFiles.length > 0}
    <h3>
      Other Project Files
    </h3>
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
  h3 {
    display: flex;
    align-items: center;
    gap: 10px; /* Space between title and button */
  }
</style>