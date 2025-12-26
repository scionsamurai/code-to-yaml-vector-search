<!-- frontend/src/components/QueryManagement.svelte -->
<script lang="ts">
  import EditTitleModal from './EditTitleModal.svelte';
  import EditQueryModal from './EditQueryModal.svelte';
  import Notification from './Notification.svelte';

  let { project_name, query_id, initialQuery, available_queries } = $props();

  let isTitleModalOpen = $state(false);
  let isQueryModalOpen = $state(false);
  let notificationMessage = $state('');
  let notificationType = $state(''); // "success" or "error"
  let queryText = $state(initialQuery);

  function openTitleModal() {
    isTitleModalOpen = true;
  }

  function closeTitleModal() {
    isTitleModalOpen = false;
  }

  function openQueryModal() {
    isQueryModalOpen = true;
  }

  function closeQueryModal() {
    isQueryModalOpen = false;
  }

  async function updateTitle(newTitle: string) {
    try {
      const response = await fetch('/update-analysis-title', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          project: project_name,
          title: newTitle,
          query_id: query_id,
        }),
      });

      if (response.ok) {
        // Update UI (dropdown) - access the select element directly or use a reactive statement
        const selectedOption = document.querySelector('#query-select option:checked');
        if (selectedOption) {
          selectedOption.textContent = newTitle;
        }
        closeTitleModal();
        showNotification('Title updated successfully!', 'success');
      } else {
        // Handle error
        const errorData = await response.text();
        console.error('Failed to update title:', errorData);
        showNotification(`Failed to update title: ${errorData}`, 'error');
      }
    } catch (error) {
      console.error('Error updating title:', error);
      showNotification('An error occurred.', 'error');
    }
  }

  async function updateQuery(newQuery: string) {
    try {
      const response = await fetch('/update-analysis-query', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          project: project_name,
          query: newQuery,
          query_id: query_id,
        }),
      });

      if (response.ok) {
        queryText = newQuery; // Update local query text
        closeQueryModal();
        showNotification('Query updated successfully!', 'success');
      } else {
        const errorData = await response.text();
        console.error('Failed to update query:', errorData);
        showNotification(`Failed to update query: ${errorData}`, 'error');
      }
    } catch (error) {
      console.error('Error updating query:', error);
      showNotification('An error occurred.', 'error');
    }
  }

  function showNotification(message: string, type: string) {
    notificationMessage = message;
    notificationType = type;
    setTimeout(() => {
      notificationMessage = ''; // Clear after a delay
    }, 3000);
  }
    function switchQuery(e: Event) {
        const target = e.target as HTMLSelectElement;
        window.location.href = `/${project_name}/${target.value}`;
    }
</script>

<div class="query-management-container">
  <div class="query-selector">
      <label for="query-select"><strong>Select Query:</strong></label>
      <select onchange={switchQuery} id="query-select" value={query_id}>
        {#each available_queries as [id, title]}
          <option value={id}>{title}</option>
        {/each}
      </select>
    <button id="edit-title-btn" class="secondary" onclick={openTitleModal}>Edit Title</button>
  </div>

  <div class="query-display-container">
    <p id="query-display">{queryText}</p>
    <button id="edit-query-btn" class="secondary" onclick={openQueryModal}>Edit Query</button>
  </div>

  <EditTitleModal
    isOpen={isTitleModalOpen}
    title={available_queries.find(([id, title]: [any, any]) => id === query_id)?.[1] || ''}
    onClose={closeTitleModal}
    onUpdate={updateTitle}
    query_id={query_id}
  />

  <EditQueryModal
    isOpen={isQueryModalOpen}
    query={queryText}
    onClose={closeQueryModal}
    onUpdate={updateQuery}
    project_name={project_name}
    query_id={query_id}
  />

  {#if notificationMessage}
    <Notification message={notificationMessage} type={notificationType} />
  {/if}
</div>

<style>
  /* Add styles for the container and any other elements */
</style>