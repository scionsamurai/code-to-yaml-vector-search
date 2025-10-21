// static/analyze-query/utils.js
import { decodeHtmlEntities } from './syntax-highlighting.js';

export function formatMessage(content) {
  const renderer = new marked.Renderer();

  // Override the 'code' method (for fenced code blocks, e.g., ```rust ... ```)
  renderer.code = (code) => {
    const language = code.lang || "plaintext";
    const encodedCode = encodeURIComponent(code.text);
    return `<pre class="shiki-block" data-language="${language}" data-original-code="${encodedCode}"><code class="language-${language}">${code.raw}</code></pre>`;
  };

  // Override the 'codespan' method (for inline code, e.g., `code`)
  renderer.codespan = (text) => {
    const literalCodeContent = decodeHtmlEntities(text.text);

    const tempDiv = document.createElement('div');
    tempDiv.textContent = literalCodeContent;
    const htmlSafeCodeContent = tempDiv.innerHTML;

    return `<code>${htmlSafeCodeContent}</code>`;
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

// This function normalizes a file path, making it absolute if necessary.
function normalizeFilePath(filePath, baseProjectSourceDir) {
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
    }

    return absolutePath.replace(/\\/g, "/"); // Normalize to forward slashes
}


function createVsCodeLink(filePath, baseProjectSourceDir) {
  const normalizedPath = normalizeFilePath(filePath, baseProjectSourceDir);
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

      // Normalize the file path for matching
      const normalizedMatch = normalizeFilePath(match, projectSourceDirectory);

      // Check if the file is currently selected in the file list
      const fileId = match.replace(/[/\\.]/g, '-'); // Create a unique ID
      const isChecked = document.querySelector(`.file-list input[type="checkbox"][value="${normalizedMatch}"]`)?.checked || false;


      return `<label class="file-link-container"><input type="checkbox" class="file-path-checkbox" data-file-path="${escapedMatch}" value="${normalizedMatch}" id="checkbox-${fileId}" ${isChecked ? 'checked' : ''}><a href="${vscodeUri}" class="file-path-link">${escapedMatch}</a></label>`;
    });

    if (newHtml !== originalText) {
      const span = document.createElement("span");
      span.innerHTML = newHtml;
      textNode.parentNode.replaceChild(span, textNode);
    }
  });
}