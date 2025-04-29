// Write this to static/split-file.js
function suggestSplit(project, filePath) {
    // Show a loading indicator
    const loadingDiv = document.createElement('div');
    loadingDiv.id = 'loading-overlay';
    loadingDiv.innerHTML = '<div class="loading-spinner"></div><div>Analyzing file structure...</div>';
    document.body.appendChild(loadingDiv);
    
    // Make the request
    fetch(`/suggest-split?project=${encodeURIComponent(project)}&file_path=${encodeURIComponent(filePath)}`, {
        method: 'POST',
    })
    .then(response => response.text())
    .then(data => {
        // Remove loading indicator
        document.body.removeChild(document.getElementById('loading-overlay'));
        
        // Show the result in a modal
        const modal = document.createElement('div');
        modal.className = 'modal';
        modal.innerHTML = `
            <div class="modal-content">
                <span class="close" onclick="this.parentElement.parentElement.remove()">&times;</span>
                <h2>File Split Suggestion</h2>
                <h3>File: ${filePath}</h3>
                <div class="split-suggestion"><pre>${data}</pre></div>
            </div>
        `;
        document.body.appendChild(modal);
    })
    .catch(error => {
        // Remove loading indicator
        document.body.removeChild(document.getElementById('loading-overlay'));
        alert('Error: ' + error);
    });
}