
async function deleteProject(projectName) {
    const confirmed = confirm(`Are you sure you want to delete the project '${projectName}'?`);
    if (confirmed) {
        const response = await fetch(`/delete/${projectName}`, {
            method: 'DELETE'
        });
        if (response.ok) {
            // Option 1: Refresh the page
            window.location.reload();
        } else {
            alert(`Failed to delete project '${projectName}'`);
        }
    }
}