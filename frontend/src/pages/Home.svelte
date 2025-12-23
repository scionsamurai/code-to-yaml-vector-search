<!-- frontend/src/pages/Home.svelte -->
<script lang="ts">
    let { extraData } = $props();
    let { projects } = $derived(extraData);

    function updateProject(projectName: string) {
        window.location.href = `/update/${projectName}/yaml`;
    }

    function resetProject(projectName: string) {
        window.location.href = `/update/${projectName}/yaml?force=true`;
    }

    function deleteProject(projectName: string) {
        if (confirm(`Are you sure you want to delete the project '${projectName}'?`)) {
            fetch(`/delete/${projectName}`, {
                method: 'DELETE'
            })
            .then(response => {
                if (response.ok) {
                    window.location.reload();
                } else {
                    alert(`Failed to delete project '${projectName}'`);
                }
            });
        }
    }
</script>

<svelte:head>
    <title>Project Manager</title>
    <link rel="stylesheet" href="/static/home.css">
    <script src="/static/home.js"></script>
</svelte:head>

<h1>Welcome to the Project-to-YAML converter!</h1>
<p>Here, you can convert projects files to YAML representations.</p>
<a href="/update-env">Update Environment Variables</a>

<h2>Projects</h2>
<ul>
    {#each projects as project}
        <li>
            <a href="/projects/{project.name}">{project.name}</a>
            {#if project.needs_update}
                <button onclick={() => updateProject(project.name)} style="background-color: green; color: white;">Update</button>
            {/if}
            <button onclick={() => resetProject(project.name)} style="background-color: darkred; color: white;">reset</button>
            <button onclick={() => deleteProject(project.name)} style="background-color: red; color: white;">Delete</button>
        </li>
    {/each}
</ul>

<form action="/projects" method="post" class="form-container">
    <label for="name">Project Name:</label>
    <input type="text" id="name" name="name" required>

    <label for="languages">File Extensions (comma-separated):</label>
    <input type="text" id="languages" name="languages" required>

    <label for="source_dir">Source Directory:</label>
    <input type="text" id="source_dir" name="source_dir" required>

    <label for="llm_select">Choose a Model:</label>
    <select name="llms" id="llm_select">
        <option value="gemini">Gemini</option>
        <option value="openai">OpenAI</option>
        <option value="anthropic">Anthropic</option>
    </select>

    <button type="submit">Create Project</button>
</form>