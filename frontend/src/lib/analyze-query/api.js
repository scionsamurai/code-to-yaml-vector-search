// frontend/src/lib/analyze-query/api.js
export async function updateContext(project_name, query_id, files, include_descriptions) {
    try {
        const response = await fetch('/update-analysis-context', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                project: project_name,
                query_id: query_id,
                files: files,
                include_file_descriptions: include_descriptions,
            }),
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        const data = await response.json();
        return data;
    } catch (error) {
        console.error('Error updating context:', error);
        if (typeof error === 'object' && error !== null && 'message' in error) alert(`Error updating context: ${error.message}`);
        throw error;
    }
}

export async function fetchChatHistory(project_name, query_id) {
  try {
    const response = await fetch(`/${project_name}/${query_id}/chat_history`);
    if (!response.ok) {
      throw new Error(`Failed to fetch chat history: ${response.statusText}`);
    }
    const data = await response.json();
    return data.history;
  } catch (error) {
    console.error('Error fetching chat history:', error);
    if (typeof error === 'object' && error !== null && 'message' in error) alert(`Error fetching chat history: ${error.message}`);
    return [];
  }
}

export async function fetchOtherProjectFiles(project_name, llm_suggested_files, relevant_files) {
  try {
    const excludedFiles = [...llm_suggested_files, ...relevant_files];

    const response = await fetch('/get-other-project-files', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        project_name: project_name,
        excluded_files: excludedFiles,
      }),
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data = await response.json();
    if (data.success && Array.isArray(data.files)) {
      return data.files;
    } else {
      console.error('API response for other files was not successful:', data);
      return [];
    }
  } catch (error) {
    console.error('Error fetching other project files:', error);
    if (typeof error === 'object' && error !== null && 'message' in error) alert(`Error fetching other project files: ${error.message}`);
    return [];
  }
}
export async function fetchBranchingData(project_name, query_id) {
    try {
        const response = await fetch(`/get-branching-data?project_name=${project_name}&query_id=${query_id}`);
        if (!response.ok) {
          throw new Error(`Failed to fetch branching data: ${response.statusText}`);
        }
        const branch_display_data = await response.json();
        return branch_display_data;
      } catch (error) {
        console.error('Error fetching branching data:', error);
        if (typeof error === 'object' && error !== null && 'message' in error) alert(`Error fetching branching data: ${error.message}`);
        return {};
      }
}
export async function toggleAutoCommitBackend(project_name, query_id, autoCommit) {
        try {
          const response = await fetch(`/query/update_auto_commit?project_name=${project_name}&query_id=${query_id}&auto_commit=${autoCommit}`, {
            method: 'POST',
          });

          if (!response.ok) {
            throw new Error(`Failed to update auto-commit: ${response.statusText}`);
          }
        } catch (error) {
          throw error;
        }
}