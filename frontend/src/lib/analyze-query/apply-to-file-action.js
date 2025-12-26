// frontend/src/lib/analyze-query/apply-to-file-action.js

/**
 * Handles the logic for applying code content from a code block to a specified file.
 * @param {Event} event - The click event.
 * @param {HTMLElement} codeBlock - The preformatted code block element.
 * @param {HTMLElement} fileIcon - The icon element for visual feedback.
 * @param {string} projectName - The name of the current project.
 */
export async function handleApplyToFile(event, codeBlock, fileIcon, projectName) {
  event.preventDefault();
  event.stopPropagation();

  const codeContentElement = codeBlock.querySelector("code");
  const codeContent = codeContentElement
    ? codeContentElement.textContent
    : "";

  // Attempt to derive file path from the first line comment
  let filePath = "";
  const firstLine = codeContent.split("\n")[0];
  // Regex to match // path/to/file.ext or # path/to/file.ext at the start of the line
  const pathRegex =
    /(?:(?:[a-zA-Z]:[\\\/])|(?:\.{1,2}[\\\/])|(?:\/))?(?:[a-zA-Z0-9_\-.+\[\]]+\/)+(?:[a-zA-Z0-9_\-.+\[\]]+\.[a-zA-Z0-9_\-]+)\b/g;
    

  const match = firstLine.match(pathRegex);

  let derivedPath = null;
  if (match) {
    derivedPath = match[0].trim();
    console.log("Derived path from comment:", derivedPath);
  } else {
    console.log("No file path derived from first line comment.");
  }

  if (derivedPath) {
    const confirmApply = confirm(
      `Apply code to '${derivedPath}'? (Click Cancel to enter manually)`
    );
    if (confirmApply) {
      filePath = derivedPath;
    } else {
      filePath = prompt(
        "Enter the relative file path to apply this code to:",
        derivedPath
      );
    }
  } else {
    filePath = prompt(
      "Enter the relative file path to apply this code to (e.g., src/main.rs):"
    );
  }

  if (!filePath) {
    console.log("File path not provided. Aborting apply to file.");
    return;
  }

  try {
    const response = await fetch("/apply-code-to-file", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        project: projectName, // Use the passed projectName
        file_path: filePath,
        content: codeContent,
      }),
    });

    if (response.ok) {
      fileIcon.innerHTML = "&#x2713;"; // Checkmark
      fileIcon.style.color = "lightgreen";
      alert(`Code successfully applied to ${filePath}`);
    } else {
      const errorText = await response.text();
      fileIcon.innerHTML = "&#x2716;"; // X mark
      fileIcon.style.color = "red";
      alert(`Failed to apply code to ${filePath}: ${errorText}`);
    }
  } catch (error) {
    console.error("Error applying code to file:", error);
    fileIcon.innerHTML = "&#x2716;"; // X mark
    fileIcon.style.color = "red";
    alert(
      `A network error occurred while applying code to file: ${error.message}`
    );
  } finally {
    setTimeout(() => {
      fileIcon.innerHTML = "&#x2398;"; // Original apply icon
      fileIcon.style.color = ""; // Reset color
    }, 2000);
  }
}