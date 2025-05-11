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
                        query: newQuery
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