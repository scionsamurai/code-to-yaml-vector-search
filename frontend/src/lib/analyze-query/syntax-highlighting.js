// frontend/src/lib/analyze-query/syntax-highlighting.js
import { codeToHtml } from "shiki";
import { handleApplyToFile } from "./apply-to-file-action.js";
import { linkFilePathsInElement } from "./utils.js";

/**
 * Fallback for older browsers for copy to clipboard.
 * @param {string} text - The text to copy.
 */
function fallbackCopyTextToClipboard(text) {
  const textarea = document.createElement("textarea");
  textarea.value = text;
  textarea.style.position = "fixed"; // Avoid scrolling to bottom
  textarea.style.left = "-9999px"; // Move off-screen
  document.body.appendChild(textarea);
  textarea.focus();
  textarea.select();
  try {
    document.execCommand("copy");
    console.log("Fallback: Code copied successfully!");
  } catch (err) {
    console.error("Fallback: Error copying code:", err);
  }
  document.body.removeChild(textarea);
}

/**
 * Adds copy-to-clipboard and apply-to-file action buttons to a code block.
 * @param {HTMLElement} codeBlock - The preformatted code block element.
 * @param {string} projectName - The name of the current project.
 */
function addCodeBlockActions(codeBlock, projectName) {
  // Check if actions are already added to avoid duplicates
  if (codeBlock.querySelector('.hljs__link_and_copy')) {
    return;
  }

  // Get the language
  const lang = codeBlock.getAttribute("data-language");

  if (lang !== null) {
    // Create the main control container (reusing .hljs__link_and_copy)
    const linkAndCopySpan = document.createElement("div");
    linkAndCopySpan.className = "hljs__link_and_copy";

    // --- Copy Link Button ---
    const copyLinkWrapper = document.createElement("span");
    copyLinkWrapper.className = "link_and_copy__copy_link pointer";
    copyLinkWrapper.setAttribute("title", "Copy code to clipboard");

    const copyIcon = document.createElement("span");
    copyIcon.className = "action-icon";
    copyIcon.innerHTML = "&#x1F4CB;"; // Clipboard icon
    copyLinkWrapper.appendChild(copyIcon);
    linkAndCopySpan.appendChild(copyLinkWrapper);

    // --- Apply to File Button ---
    const applyToFileWrapper = document.createElement("span");
    applyToFileWrapper.className = "link_and_copy__apply_link pointer";
    applyToFileWrapper.setAttribute("title", "Apply code to file");

    const fileIcon = document.createElement("span");
    fileIcon.className = "action-icon";
    fileIcon.innerHTML = "&#x2398;"; // Apply icon (original)
    applyToFileWrapper.appendChild(fileIcon);
    linkAndCopySpan.appendChild(applyToFileWrapper);

    // --- Language Text ---
    const langText = document.createElement("span");
    langText.className = "link_and_copy__text";
    langText.textContent = lang;
    linkAndCopySpan.appendChild(langText);

    // Append the main controls container to the pre element
    codeBlock.appendChild(linkAndCopySpan);

    // --- Event Listeners ---

    // Copy to clipboard
    copyLinkWrapper.addEventListener("click", function handleClick(event) {
      event.preventDefault();
      event.stopPropagation();

      const codeContentElement = codeBlock.querySelector("code");
      const cleanCodeContent = codeContentElement
        ? codeContentElement.textContent.trim()
        : "";

      if (navigator.clipboard) {
        navigator.clipboard.writeText(cleanCodeContent).then(
          () => console.log("Code copied successfully!"),
          (error) => {
            console.error("Error copying code:", error);
            fallbackCopyTextToClipboard(cleanCodeContent);
          }
        );
      } else {
        fallbackCopyTextToClipboard(cleanCodeContent);
      }

      copyIcon.innerHTML = "&#x2713;"; // Checkmark
      copyIcon.style.color = "lightgreen";
      setTimeout(() => {
        copyIcon.innerHTML = "&#x1F4CB;"; // Original copy icon
        copyIcon.style.color = ""; // Reset color
      }, 2000);
    });

    // Apply to file (delegated to separate function)
    applyToFileWrapper.addEventListener("click", (event) =>
      handleApplyToFile(event, codeBlock, fileIcon, projectName)
    );
  }
}


export function decodeHtmlEntities(str) {
  const textarea = document.createElement("textarea");
  textarea.innerHTML = str;
  return textarea.value;
}

export function highlightAction(node, selectedFiles) {
  // Access project_name from the component's context
  // component is not available from actions.  Need some other solution
  linkFilePathsInElement(node, selectedFiles);

  const process = async (projectName) => {
    const blocks = node.querySelectorAll('.shiki-block:not(.highlighted)');
    if (blocks.length === 0) return;

    for (const block of blocks) {
      const language = block.getAttribute("data-language") || "plaintext";
      const originalCode2 = decodeURIComponent(
        block.getAttribute("data-original-code")
      );
      const originalCode = decodeHtmlEntities(originalCode2);

      if (originalCode) {
        try {
          const highlightedHtml = await codeToHtml(originalCode, {
            lang: language,
            theme: "github-dark",
          });

          // Replace the content while preserving the data attributes
          block.innerHTML = highlightedHtml;

          // Add classes for styling
          block.classList.add("highlighted");

          // Add copy/apply buttons
          addCodeBlockActions(block, projectName);
        } catch (error) {
          console.error("Error highlighting code:", error);
        }
      }
    }
  };

  // Initial call to process - requires a project name
  // The project_name isn't in scope, so lets use a default empty string
  process("")

  return {
    update(projectName) {
      process(projectName); // Re-run if message content updates, passing projectName
    }
  };
}