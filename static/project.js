// static/project.js

async function regenerate(projectName, yamlPath) {
  const response = await fetch(
    `/regenerate?project=${projectName}&yamlpath=${yamlPath}`,
    {
      method: "POST",
    }
  );
  const newContent = await response.text();
  const yamlFileElement = document.querySelector(
    `pre:has(+ button[onclick^="regenerate('${projectName}', '${yamlPath}')"])`
  );
  yamlFileElement.textContent = newContent;
}

async function validateFilePaths(projectName) {
  const response = await fetch(`/projects/${projectName}/validate_paths`, {
    method: "POST",
  });

  const results = await response.json();

  // Get the modal and the list
  const modal = document.getElementById("validationModal");
  const list = document.getElementById("validationList");

  // Clear existing list items
  list.innerHTML = "";

  if (results.length === 0) {
    const listItem = document.createElement("li");
    listItem.textContent =
      "All files correctly have the relative comment at top of file.";
    list.appendChild(listItem);
  } else {
    // Populate the list with results
    for (const [filePath, isValid] of results) {
      const listItem = document.createElement("li");
      listItem.textContent = `${filePath}: ${isValid ? "OK" : "Incorrect"}`;
      list.appendChild(listItem);
    }
  }
  // Show the modal
  modal.style.display = "block";

  // Get the close button and attach event listener
  const closeBtn = document.getElementsByClassName("close")[0];
  closeBtn.onclick = function () {
    modal.style.display = "none";
  };

  // When the user clicks anywhere outside of the modal, close it
  window.onclick = function (event) {
    if (event.target == modal) {
      modal.style.display = "none";
    }
  };
}

async function runClustering() {
  const projectName = document.getElementById('project-name').value;
  try {
      const response = await fetch(`/api/cluster/${projectName}`, { //adjust URL
          method: 'POST',
          headers: {
              'Content-Type': 'application/json'
          },
      });

      if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();

      // Display clustering results (modify as needed)
      alert(JSON.stringify(data));  // Simple alert for now

  } catch (error) {
      console.error('Error running clustering:', error);
      alert(`Error running clustering: ${error.message}`);
  }
}