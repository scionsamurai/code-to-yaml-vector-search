// static/analyze-query/context.js
export function updateContext(projectName, queryText) {
    const statusMessage = document.getElementById('context-status');
    if (statusMessage) {
        statusMessage.textContent = 'Updating context...';
        statusMessage.style.display = 'block';
    }

    const selectedFiles = [];
    document.querySelectorAll('.file-checkbox:checked').forEach(checkbox => {
        selectedFiles.push(checkbox.value);
    });

    fetch('/update-analysis-context', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            project: projectName,
            query: queryText,
            files: selectedFiles,
            query_id: document.getElementById('query-id').value,
        })
    })
    .then(response => response.json())
    .then(data => {
        if (data.success) {
            if (statusMessage) {
                statusMessage.textContent = `Context updated: ${selectedFiles.length} files selected`;
                setTimeout(() => {
                    statusMessage.style.opacity = '0';
                    setTimeout(() => {
                        statusMessage.style.display = 'none';
                        statusMessage.style.opacity = '1';
                    }, 500);
                }, 2000);
            }
        } else {
            if (statusMessage) {
                statusMessage.textContent = 'Error: Failed to update context.';
            }
        }
    })
    .catch(error => {
        console.error('Error:', error);
        if (statusMessage) {
            statusMessage.textContent = 'Error: Could not update context.';
        }
    });
}