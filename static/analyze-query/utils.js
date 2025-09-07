// static/analyze-query/utils.js
export function formatMessage(content) {
  const renderer = new marked.Renderer();

  // Override the 'code' method
  renderer.code = (code) => {
    const language = code.lang || "plaintext";
    const encodedCode = encodeURIComponent(code.text);
    return `<pre class="shiki-block" data-language="${language}" data-original-code="${encodedCode}"><code class="language-${language}">${code.raw}</code></pre>`;
  };

  // Configure marked to use your custom renderer
  marked.setOptions({
    renderer: renderer,
    breaks: true,
  });

  return marked.parse(content);
}

// --- NEW CODE FOR VS CODE LINKING ---

let projectSourceDirectory = "";

/**
 * Sets the base project source directory for constructing absolute file paths.
 * @param {string} path - The absolute path to the project's source directory.
 */
export function setProjectSourceDirectory(path) {
  projectSourceDirectory = path;
}

/**
 * Constructs a VS Code URI for a given file path.
 * It attempts to make the path absolute if it's relative and projectSourceDir is available.
 * @param {string} filePath - The file path (can be relative or absolute).
 * @param {string} baseProjectSourceDir - The project's source directory.
 * @returns {string} The VS Code URI or the original filePath if URI cannot be constructed.
 */
function createVsCodeLink(filePath, baseProjectSourceDir) {
  if (!filePath) {
    return filePath;
  }

  let absolutePath = filePath;

  // Check if filePath is already an absolute path (Unix, Windows drive letter, or UNC path)
  const isAbsolutePath =
    filePath.startsWith("/") ||
    filePath.startsWith("\\") ||
    filePath.match(/^[a-zA-Z]:[/\\]/) ||
    filePath.match(/^\\\\[a-zA-Z0-9_.-]+\\[a-zA-Z0-9_.-]+/);

  if (!isAbsolutePath && baseProjectSourceDir) {
    // If it's a relative path and we have a project source directory, construct the full path
    const baseDir =
      baseProjectSourceDir.endsWith("/") || baseProjectSourceDir.endsWith("\\")
        ? baseProjectSourceDir
        : baseProjectSourceDir + "/";
    absolutePath = `${baseDir}${filePath}`;
  } else if (!isAbsolutePath && !baseProjectSourceDir) {
    // If it's a relative path and projectSourceDir is missing, we can't make it absolute.
    // VS Code *might* open relative paths based on its CWD, so we'll still create a link,
    // but log a warning.
    console.warn(
      "Could not determine absolute path for file, projectSourceDir not available. Using relative path for VS Code link:",
      filePath
    );
  }

  // Normalize path for consistent URI (forward slashes for cross-platform compatibility)
  const normalizedPath = absolutePath.replace(/\\/g, "/");

  // Encode the path to be safe in a URI
  const encodedPath = encodeURIComponent(normalizedPath);

  return `vscode://file/${encodedPath}`;
}

/**
 * Finds file paths within the given HTML element's text nodes and converts them into clickable VS Code links.
 * It explicitly avoids modifying content within <pre> blocks.
 * @param {HTMLElement} element - The HTML element to process (e.g., a message-content div).
 */
export function linkFilePathsInElement(element) {
  if (!projectSourceDirectory) {
    console.warn(
      "projectSourceDirectory is not set. File links will not be generated."
    );
    return;
  }

  const pathRegex =
    /(?:(?:[a-zA-Z]:[\\\/])|(?:\.{1,2}[\\\/])|(?:\/))?(?:[a-zA-Z0-9_\-]+\/)+(?:[a-zA-Z0-9_\-.]+\.(?:rs|js|ts|tsx|jsx|html|css|py|java|cpp|h|hpp|c|go|yml|yaml|json|txt|md|toml|lock|gitignore|editorconfig|env|sh|ps1|bat|dockerfile|properties|xml|sql|conf|cfg|ini)|(?:[a-zA-Z0-9_\-+]+\.svelte))\b/g;

  // Get all <pre> elements within the target element. We will skip text nodes inside these.
  const preElements = element.querySelectorAll("pre");

  // Create a TreeWalker that visits all text nodes. We'll filter them manually.
  const walk = document.createTreeWalker(element, NodeFilter.SHOW_TEXT, null);

  let node;
  const nodesToProcess = [];
  while ((node = walk.nextNode())) {
    // Check if the current text node is contained within any of the <pre> elements
    let isInPre = false;
    for (const pre of preElements) {
      if (pre.contains(node)) {
        isInPre = true;
        break;
      }
    }
    // If the text node is NOT inside a <pre> block, add it for processing
    if (!isInPre) {
      nodesToProcess.push(node);
    }
  }

  nodesToProcess.forEach((textNode) => {
    let originalText = textNode.nodeValue;
    let newHtml = originalText;

    // Apply the regex to find and replace file paths
    newHtml = newHtml.replace(pathRegex, (match) => {
      const vscodeUri = createVsCodeLink(match, projectSourceDirectory);
      // Ensure the match itself is properly HTML escaped before embedding in <a> tag
      const escapedMatch = new Option(match).innerHTML;
      // REMOVED
      return `<a href="${vscodeUri}" class="file-path-link">${escapedMatch}</a>`;
    });

    // If modifications were made, replace the text node with a new span containing the HTML
    if (newHtml !== originalText) {
      const span = document.createElement("span");
      span.innerHTML = newHtml;
      textNode.parentNode.replaceChild(span, textNode);
    }
  });
}
