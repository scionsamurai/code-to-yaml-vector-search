// frontend/src/lib/analyze-query/api.js
export async function updateContext(project_name, query_id, files, include_descriptions, grounding_with_search) { // MODIFIED: Added grounding_with_search
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
                grounding_with_search: grounding_with_search, // ADDED: Pass grounding_with_search
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

// --- NEW API FUNCTIONS FOR GIT STATUS AND BRANCHES ---

/**
 * Fetches the current Git status (uncommitted changes, unpushed commits) from the backend.
 * @param {string} project_name The name of the project.
 * @returns {Promise<{has_uncommitted_changes: boolean, has_unpushed_commits: boolean}>}
 */
export async function fetchGitStatus(project_name) {
    try {
        const response = await fetch(`/git-status?project_name=${project_name}`);
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        const data = await response.json();
        if (data.success) {
            return {
                has_uncommitted_changes: data.has_uncommitted_changes,
                has_unpushed_commits: data.has_unpushed_commits
            };
        } else {
            console.error('Failed to fetch Git status:', data.message);
            // Optionally display a user-facing alert here, or let the caller handle it.
            return { has_uncommitted_changes: false, has_unpushed_commits: false };
        }
    } catch (error) {
        console.error('Error fetching Git status:', error);
        // Optionally display a user-facing alert here.
        return { has_uncommitted_changes: false, has_unpushed_commits: false };
    }
}

/**
 * Fetches all local Git branches and the current repository branch name from the backend.
 * @param {string} project_name The name of the project.
 * @returns {Promise<{all_branches: string[], current_repo_branch: string | null}>}
 */
export async function fetchRepoBranches(project_name) {
    try {
        const response = await fetch(`/git-branches?project_name=${project_name}`);
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        const data = await response.json();
        if (data.success && data.branches) {
            return {
                all_branches: data.branches,
                current_repo_branch: data.current_repo_branch // This is the actual repo HEAD branch
            };
        } else {
            console.error('Failed to fetch Git branches:', data.message);
            return { all_branches: [], current_repo_branch: null };
        }
    } catch (error) {
        console.error('Error fetching Git branches:', error);
        return { all_branches: [], current_repo_branch: null };
    }
}

export async function createGitBranch(project_name, branch_name) {
    try {
        const response = await fetch('/create-git-branch', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ project_name, branch_name }),
        });
        const data = await response.json();
        if (!response.ok) {
            throw new Error(data.message || `HTTP error! status: ${response.status}`);
        }
        return data;
    } catch (error) {
        console.error('Error creating branch:', error);
        throw error;
    }
}

export async function checkoutGitBranch(project_name, branch_name) {
    try {
        const response = await fetch('/checkout-git-branch', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ project_name, branch_name }),
        });
        const data = await response.json();
        if (!response.ok) {
            throw new Error(data.message || `HTTP error! status: ${response.status}`);
        }
        return data;
    } catch (error) {
        console.error('Error checking out branch:', error);
        throw error;
    }
}

export async function mergeGitBranch(project_name) {
    try {
        const response = await fetch('/merge-git-branch', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ project_name }),
        });
        const data = await response.json();
        if (!response.ok) {
            // Handle specific conflict status
            if (response.status === 409) {
                throw new Error(data.message || 'Merge conflicts detected.');
            }
            throw new Error(data.message || `HTTP error! status: ${response.status}`);
        }
        return data;
    } catch (error) {
        console.error('Error merging branch:', error);
        throw error;
    }
}

export async function suggestBranchName(project_name, query_id) {
    try {
        const response = await fetch('/suggest-branch-name', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ project_name, query_id }),
        });
        const data = await response.json();
        if (!response.ok) {
            throw new Error(data.message || `HTTP error! status: ${response.status}`);
        }
        return data.branch_name;
    } catch (error) {
        console.error('Error suggesting branch name:', error);
        throw error;
    }
}