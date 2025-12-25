// static/analyze-query/code-block-actions.js
import { handleApplyToFile } from "./apply-to-file-action.js";

/**
 * Adds copy-to-clipboard and apply-to-file action buttons to all preformatted code blocks
 * and initializes their event listeners.
 * @param {HTMLElement} el - The HTML element to search within (defaults to document).
 */
export async function initCodeBlockActions(el = document) {
  const codeBlocks = el.querySelectorAll("pre");

  for (const codeBlock of codeBlocks) {
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
        handleApplyToFile(event, codeBlock, fileIcon)
      );
    }
  }
}

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