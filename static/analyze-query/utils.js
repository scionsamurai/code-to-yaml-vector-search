// static/analyze-query/utils.js
export function formatMessage(content) {
  const renderer = new marked.Renderer();

  // Override the 'code' method
  renderer.code = (code) => {
    const language = code.lang || "plaintext";
    const encodedCode = encodeURIComponent(code.text);
    return `<pre class="shiki-block" data-language="${language}" data-original-code="${encodedCode}"><code class="language-${language}">${code.raw}</code></pre>`;
  };

  marked.setOptions({
    renderer: renderer,
    breaks: true,
  });

  return marked.parse(content);
}

let projectSourceDirectory = "";

export function setProjectSourceDirectory(path) {
  projectSourceDirectory = path;
}

function createVsCodeLink(filePath, baseProjectSourceDir) {
  if (!filePath) {
    return filePath;
  }

  let absolutePath = filePath;

  const isAbsolutePath =
    (filePath.startsWith("/") && !filePath.startsWith("/src")) ||
    (filePath.startsWith("\\") && !filePath.startsWith("\\src")) ||
    /^[a-zA-Z]:[/\\]/.test(filePath) ||
    /^\\\\[a-zA-Z0-9_.-]+\\[a-zA-Z0-9_.-]+/.test(filePath);

  if (!isAbsolutePath && baseProjectSourceDir) {
    const cleanedFilePath =
      filePath.startsWith("/") || filePath.startsWith("\\")
        ? filePath.slice(1)
        : filePath;
    const baseDir =
      baseProjectSourceDir.endsWith("/") || baseProjectSourceDir.endsWith("\\")
        ? baseProjectSourceDir
        : baseProjectSourceDir + "/";
    absolutePath = `${baseDir}${cleanedFilePath}`;
  } else if (!isAbsolutePath && !baseProjectSourceDir) {
    console.warn(
      "Could not determine absolute path for file, projectSourceDir not available. Using relative path for VS Code link:",
      filePath
    );
  }

  const normalizedPath = absolutePath.replace(/\\/g, "/");

  const encodedPath = encodeURIComponent(normalizedPath);

  return `vscode://file/${encodedPath}`;
}

export function linkFilePathsInElement(element) {
  if (!projectSourceDirectory) {
    console.warn(
      "projectSourceDirectory is not set. File links will not be generated."
    );
    return;
  }

  const pathRegex =
    /(?:(?:[a-zA-Z]:[\\\/])|(?:\.{1,2}[\\\/])|(?:\/))?(?:[a-zA-Z0-9_\-]+\/)+(?:[a-zA-Z0-9_\-.]+\.[a-zA-Z0-9_\-]+)\b/g;

  const preElements = element.querySelectorAll("pre");

  const walk = document.createTreeWalker(element, NodeFilter.SHOW_TEXT, null);

  let node;
  const nodesToProcess = [];
  while ((node = walk.nextNode())) {
    let isInPre = false;
    for (const pre of preElements) {
      if (pre.contains(node)) {
        isInPre = true;
        break;
      }
    }
    if (!isInPre) {
      nodesToProcess.push(node);
    }
  }

  nodesToProcess.forEach((textNode) => {
    let originalText = textNode.nodeValue;
    let newHtml = originalText;

    newHtml = newHtml.replace(pathRegex, (match) => {
      const vscodeUri = createVsCodeLink(match, projectSourceDirectory);
      const escapedMatch = new Option(match).innerHTML;
      return `<a href="${vscodeUri}" class="file-path-link">${escapedMatch}</a>`;
    });

    if (newHtml !== originalText) {
      const span = document.createElement("span");
      span.innerHTML = newHtml;
      textNode.parentNode.replaceChild(span, textNode);
    }
  });
}
