// static/analyze-query/git-branch-actions.js

document.addEventListener('DOMContentLoaded', () => {
    const autoCommitCheckbox = document.getElementById('auto-commit-checkbox');
    const startNewBranchButton = document.getElementById('start-new-branch-button');
    const pushChangesButton = document.getElementById('push-changes-button');
    const mergeToMainButton = document.getElementById('merge-to-main-button');
    const commitButton = document.getElementById('commit-button');
    const gitBranchSelector = document.getElementById('git-branch-selector');
    const gitActionMessageDiv = document.getElementById('git-action-message');

    const projectName = document.getElementById('project-name').value;
    const queryId = document.getElementById('query-id').value;
    let projectGitBranchName = document.getElementById('git-branch-selector').value;

    function displayGitMessage(message, isError = false) {
        gitActionMessageDiv.textContent = message;
        gitActionMessageDiv.style.backgroundColor = isError ? '#f8d7da' : '#d4edda'; // Bootstrap alert colors
        gitActionMessageDiv.style.color = isError ? '#721c24' : '#155724';
        gitActionMessageDiv.style.display = 'block';
        setTimeout(() => { gitActionMessageDiv.style.display = 'none'; }, 5000);
    }

    if (pushChangesButton) {
        pushChangesButton.addEventListener('click', async () => {
            if (!confirm(`Are you sure you want to push changes for branch '${gitBranchSelector.dataset.currentBranch}' to remote?`)) {
                displayGitMessage('Push cancelled.', true);
                return;
            }

            try {
                const response = await fetch('/push-git-changes', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ project_name: projectName }),
                });

                const data = await response.json();
                if (data.success) {
                    displayGitMessage(data.message);
                } else {
                    displayGitMessage('Push failed: ' + data.message, true);
                }
            } catch (error) {
                console.error('Error pushing changes:', error);
                displayGitMessage('Error pushing changes.', true);
            }
        });
    }

    if (autoCommitCheckbox) {
        autoCommitCheckbox.addEventListener('change', async () => {
            const autoCommit = autoCommitCheckbox.checked;
            try {
                const response = await fetch('/update-query-auto-commit', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        project_name: projectName,
                        query_id: queryId,
                        auto_commit: autoCommit,
                    }),
                });

                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }

                const data = await response.json();
                if (!data.success) {
                    console.error('Failed to update auto-commit:', data.error);
                    displayGitMessage('Failed to update auto-commit setting: ' + data.error, true);
                    autoCommitCheckbox.checked = !autoCommit; // Revert checkbox state
                } else {
                    displayGitMessage('Auto-commit setting updated.');
                    // Toggle commit button visibility based on auto-commit state
                    if (commitButton) {
                        commitButton.style.display = autoCommit ? 'none' : 'inline';
                    }
                }
            } catch (error) {
                console.error('Error updating auto-commit:', error);
                displayGitMessage('Error updating auto-commit setting.', true);
                autoCommitCheckbox.checked = !autoCommit; // Revert checkbox state
            }
        });
        
        if (commitButton) {
            commitButton.style.display = autoCommitCheckbox.checked ? 'none' : 'inline';
        }
    }
    if (commitButton) {
        commitButton.addEventListener('click', async () => {
            const commitMessage = prompt('Enter commit message:');
            if (!commitMessage) {
                displayGitMessage('Commit cancelled.', true);
                return;
            }

            try {
                const response = await fetch('/commit-changes', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ project_name: projectName, message: commitMessage }),
                });

                const data = await response.json();
                if (data.success) {
                    displayGitMessage(data.message);
                } else {
                    displayGitMessage('Commit failed: ' + data.message, true);
                }
            } catch (error) {
                console.error('Error committing changes:', error);
                displayGitMessage('Error committing changes.', true);
            }
        });
     }

    if (startNewBranchButton) {
        startNewBranchButton.addEventListener('click', async () => {
            if (startNewBranchButton.disabled) {
                return; // Do nothing if disabled
            }
            try {
                const response = await fetch('/suggest-branch-name', { // Corrected URL
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        project_name: projectName,
                        query_id: queryId,
                    }),
                });

                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }

                const data = await response.json();
                const suggestedBranchName = data.branch_name;

                const branchName = prompt('Enter branch name:', suggestedBranchName);

                if (branchName) {
                    const createBranchResponse = await fetch('/create-git-branch', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ project_name: projectName, branch_name: branchName }),
                    });

                    const createBranchData = await createBranchResponse.json();
                    if (createBranchData.success) {
                        displayGitMessage(createBranchData.message);
                        projectGitBranchName = branchName; // Update local state
                        startNewBranchButton.disabled = true; // Disable "Start New Branch"
                        if (mergeToMainButton) mergeToMainButton.style.display = 'inline-block'; // Show "Merge to Main"
                        await updateBranchSelector(projectName, branchName); // Update and select new branch
                    } else {
                        displayGitMessage('Failed to create branch: ' + createBranchData.message, true);
                    }
                }
            } catch (error) {
                console.error('Error suggesting branch name:', error);
                displayGitMessage('Error suggesting or creating branch.', true);
            }
        });
    }

    if (mergeToMainButton) {
        // Initial visibility based on projectGitBranchName
        if (projectGitBranchName && projectGitBranchName !== 'main' && projectGitBranchName !== 'master') {
            mergeToMainButton.style.display = 'inline-block';
        } else {
            mergeToMainButton.style.display = 'none';
        }

        mergeToMainButton.addEventListener('click', async () => {
            if (!confirm(`Are you sure you want to merge branch '${projectGitBranchName}' into the default branch and delete it?`)) {
                displayGitMessage('Merge cancelled.', true);
                return;
            }

            try {
                const response = await fetch('/merge-git-branch', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ project_name: projectName }),
                });

                const data = await response.json();
                if (response.status === 409) { // Conflict
                    displayGitMessage(data.message, true);
                } else if (data.success) {
                    displayGitMessage(data.message);
                    projectGitBranchName = ''; // No active feature branch anymore
                    startNewBranchButton.disabled = false; // Enable "Start New Branch"
                    mergeToMainButton.style.display = 'none'; // Hide "Merge to Main"
                    await updateBranchSelector(projectName, 'main'); // Update and select default branch (assuming 'main')
                } else {
                    displayGitMessage('Merge failed: ' + data.message, true);
                }
            } catch (error) {
                console.error('Error merging branch:', error);
                displayGitMessage('Error merging branch.', true);
            }
        });
    }

    if (gitBranchSelector) {
        gitBranchSelector.addEventListener('change', async (event) => {
            const selectedBranch = event.target.value;
            if (!confirm(`Are you sure you want to checkout to branch '${selectedBranch}'? Uncommitted changes will be carried over.`)) {
                // Revert selection if user cancels
                event.target.value = gitBranchSelector.dataset.currentBranch; 
                displayGitMessage('Checkout cancelled.', true);
                return;
            }

            try {
                const response = await fetch('/checkout-git-branch', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ project_name: projectName, branch_name: selectedBranch }),
                });

                const data = await response.json();
                if (data.success) {
                    displayGitMessage(data.message);
                    // Update the dataset property to reflect the new current branch
                    gitBranchSelector.dataset.currentBranch = selectedBranch; 
                    // Update projectGitBranchName if the selected branch is a feature branch
                    if (selectedBranch !== 'main' && selectedBranch !== 'master') { // Assuming main/master are default
                        projectGitBranchName = selectedBranch;
                        startNewBranchButton.disabled = true;
                        if (mergeToMainButton) mergeToMainButton.style.display = 'inline-block';
                    } else {
                        projectGitBranchName = ''; // Default branch, so no feature branch active
                        startNewBranchButton.disabled = false;
                        if (mergeToMainButton) mergeToMainButton.style.display = 'none';
                    }
                } else {
                    displayGitMessage('Failed to checkout branch: ' + data.message, true);
                    // Revert dropdown selection on failure
                    event.target.value = gitBranchSelector.dataset.currentBranch;
                }
            } catch (error) {
                console.error('Error checking out branch:', error);
                displayGitMessage('Error checking out branch.', true);
                event.target.value = gitBranchSelector.dataset.currentBranch;
            }
        });
    }

    // Function to dynamically update branch selector
    async function updateBranchSelector(projectName, newSelectedBranch = null) {
        try {
            const response = await fetch(`/git-branches?project_name=${projectName}`);
            const data = await response.json();

            if (data.success && data.branches) {
                gitBranchSelector.innerHTML = ''; // Clear existing options
                data.branches.forEach(branch => {
                    const option = document.createElement('option');
                    option.value = branch;
                    option.textContent = branch;
                    if (newSelectedBranch === branch || (newSelectedBranch === null && data.current_repo_branch === branch)) {
                        option.selected = true;
                    }
                    gitBranchSelector.appendChild(option);
                });
                // Set data-current-branch for reversion in case of checkout failure
                gitBranchSelector.dataset.currentBranch = newSelectedBranch || data.current_repo_branch || '';
            } else {
                console.error('Failed to fetch branches:', data.message);
                displayGitMessage('Failed to refresh branch list.', true);
            }
        } catch (error) {
            console.error('Error fetching branches:', error);
            displayGitMessage('Error refreshing branch list.', true);
        }
    }

    // Initial update of the branch selector to ensure its dataset attribute is correctly set
    // and to handle any dynamic changes that might have occurred.
    // It also ensures the correct visibility of buttons on page load.
    updateBranchSelector(projectName, gitBranchSelector.value); // Use current value as initial selected

    // Set the data-current-branch attribute on initial load
    if (gitBranchSelector && gitBranchSelector.value) {
        gitBranchSelector.dataset.currentBranch = gitBranchSelector.value;
    }

    // Initial update of button states based on projectGitBranchName from hidden input
    if (projectGitBranchName && projectGitBranchName !== 'main' && projectGitBranchName !== 'master') {
        if (startNewBranchButton) startNewBranchButton.disabled = true;
        if (mergeToMainButton) mergeToMainButton.style.display = 'inline-block';
    } else {
        if (startNewBranchButton) startNewBranchButton.disabled = false;
        if (mergeToMainButton) mergeToMainButton.style.display = 'none';
    }  
});
