// static/yaml-checkbox-logic.js

function initYamlLogic() {
  const projectName = document.getElementById("project-name").value;
  // Add event listeners to yaml checkboxes
  document.querySelectorAll(".yaml-checkbox").forEach((checkbox) => {
    checkbox.addEventListener("change", function () {
      const filePath = this.value;
      const useYaml = this.checked;
      updateFileYamlOverride(projectName, filePath, useYaml);
    });
  });
}

// Initialize the chat when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initYamlLogic);
} else {
  initYamlLogic();
}

async function updateFileYamlOverride(projectName, filePath, useYaml) {
  console.log("Updating YAML override for:", projectName, filePath, useYaml);
  try {
    const response = await fetch("/update-file-yaml-override", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        project: projectName,
        file_path: filePath,
        use_yaml: useYaml,
      }),
    });
    console.log("response", response);

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const result = await response.json();
    console.log("File YAML override updated:", result);

    // Optionally, update the UI to reflect the change
    const contextStatus = document.getElementById("context-status");
    contextStatus.textContent = `YAML setting for ${filePath} updated.`;
    contextStatus.style.display = "block";
    setTimeout(() => {
      contextStatus.style.opacity = 0;
      setTimeout(() => {
        contextStatus.style.display = "none";
        contextStatus.style.opacity = 1;
      }, 500);
    }, 3000);
  } catch (error) {
    console.error("Error updating file YAML override:", error);
    alert("Failed to update YAML setting. See console for details.");
  }
}
