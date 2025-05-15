// static/analyze-query/query.js
export function setupQueryEditor(projectName) {
    const editQueryBtn = document.getElementById('edit-query-btn');
    const queryEditModal = document.getElementById('query-edit-modal');
    const closeModalBtn = document.querySelector('.close-modal');
    const cancelQueryBtn = document.getElementById('cancel-query-btn');
    const updateQueryBtn = document.getElementById('update-query-btn');
    const editableQueryText = document.getElementById('editable-query-text');
    const queryDisplay = document.getElementById('query-display');
    
    // Open modal
    if (editQueryBtn && queryEditModal) {
        editQueryBtn.addEventListener('click', () => {
            queryEditModal.style.display = 'block';
            // Set focus to the textarea
            editableQueryText.focus();
        });
    }
    
    // Close modal functions
    function closeModal() {
        if (queryEditModal) {
            queryEditModal.style.display = 'none';
        }
    }
    
    if (closeModalBtn) {
        closeModalBtn.addEventListener('click', closeModal);
    }
    
    if (cancelQueryBtn) {
        cancelQueryBtn.addEventListener('click', closeModal);
    }
    
    // Close modal when clicking outside
    window.addEventListener('click', (event) => {
        if (event.target === queryEditModal) {
            closeModal();
        }
    });
    
    // Update query functionality
    if (updateQueryBtn && editableQueryText && queryDisplay) {
        updateQueryBtn.addEventListener('click', async () => {
            const newQuery = editableQueryText.value.trim();
            if (!newQuery) return;
            
            try {
                // Show loading state
                updateQueryBtn.textContent = 'Updating...';
                updateQueryBtn.disabled = true;
                
                const response = await fetch('/update-analysis-query', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        project: projectName,
                        query: newQuery,
                        query_id: document.getElementById('query-id').value, 
                    })
                });
                
                if (response.ok) {
                    // Update the query display and hidden input
                    queryDisplay.textContent = newQuery;
                    document.getElementById('query-text').value = newQuery;
                    
                    // Close the modal
                    closeModal();
                    
                    // Show notification
                    showNotification('Query updated successfully!');
                } else {
                    const errorData = await response.text();
                    console.error('Failed to update query:', errorData);
                    alert('Failed to update query. Please try again.');
                }
            } catch (error) {
                console.error('Error updating query:', error);
                alert('An error occurred while updating the query.');
            } finally {
                updateQueryBtn.textContent = 'Update Query';
                updateQueryBtn.disabled = false;
            }
        });
    }
}

function showNotification(message) {
    // Create notification element
    const notification = document.createElement('div');
    notification.className = 'notification';
    notification.textContent = message;
    document.body.appendChild(notification);
    
    // Show notification
    setTimeout(() => {
        notification.classList.add('show');
    }, 10);
    
    // Hide and remove notification after 3 seconds
    setTimeout(() => {
        notification.classList.remove('show');
        setTimeout(() => {
            document.body.removeChild(notification);
        }, 300);
    }, 3000);
}

// static/analyze-query/query.js

export function setupTitleEditor(projectName) {

    const editTitleBtn = document.getElementById('edit-title-btn');
    const titleEditModal = document.getElementById('title-edit-modal');
    const cancelTitleBtn = document.getElementById('cancel-title-btn');
    const updateTitleBtn = document.getElementById('update-title-btn');
    const editableTitleText = document.getElementById('editable-title-text');
    const querySelector = document.getElementById('query-selector');

    // Open title modal
    if (editTitleBtn && titleEditModal) {
        editTitleBtn.addEventListener('click', () => {
            titleEditModal.style.display = 'block';
            //set title to the current one
            editableTitleText.value = querySelector.options[querySelector.selectedIndex].text;
            editableTitleText.focus();
        });
    }

    // Close title modal functions
    function closeTitleModal() {
        if (titleEditModal) {
            titleEditModal.style.display = 'none';
        }
    }

    if (cancelTitleBtn) {
        cancelTitleBtn.addEventListener('click', closeTitleModal);
    }

    // Close title modal when clicking outside
    window.addEventListener('click', (event) => {
        if (event.target === titleEditModal) {
            closeTitleModal();
        }
    });

    // Update title functionality
    if (updateTitleBtn && editableTitleText && querySelector) {
        updateTitleBtn.addEventListener('click', async () => {
            const newTitle = editableTitleText.value.trim();
            if (!newTitle) return;

            try {
                // Show loading state
                updateTitleBtn.textContent = 'Updating...';
                updateTitleBtn.disabled = true;

                const queryId = document.getElementById('query-id').value;

                const response = await fetch('/update-analysis-title', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        project: projectName,
                        title: newTitle,
                        query_id: queryId,
                    })
                });

                if (response.ok) {
                    // Update the title in the dropdown
                    const selectedOption = querySelector.options[querySelector.selectedIndex];
                    selectedOption.text = newTitle;
                    // Close the modal
                    closeTitleModal();
                    // Show notification
                    showNotification('Title updated successfully!');
                } else {
                    const errorData = await response.text();
                    console.error('Failed to update title:', errorData);
                    alert('Failed to update title. Please try again.');
                }
            } catch (error) {
                console.error('Error updating title:', error);
                alert('An error occurred while updating the title.');
            } finally {
                updateTitleBtn.textContent = 'Update Title';
                updateTitleBtn.disabled = false;
            }
        });
    }
}