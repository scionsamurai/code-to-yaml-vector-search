async function regenerate(projectName, yamlPath) {
    const response = await fetch(`/regenerate?project=${projectName}&yamlpath=${yamlPath}`, {
        method: 'POST'
    });
    const newContent = await response.text();
    const yamlFileElement = document.querySelector(`pre:has(+ button[onclick^="regenerate('${projectName}', '${yamlPath}')"])`);
    yamlFileElement.textContent = newContent;
}