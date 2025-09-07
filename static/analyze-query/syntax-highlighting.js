// static/analyze-query/syntax-highlighting.js
import { codeToHtml } from "https://esm.sh/shiki@3.0.0";

export async function applySyntaxHighlighting(div = document) {
  const codeBlocks = div.querySelectorAll(".shiki-block");

  for (const block of codeBlocks) {
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
      } catch (error) {
        console.error("Error highlighting code:", error);
      }
    }
  }
}

function decodeHtmlEntities(str) {
  const textarea = document.createElement("textarea");
  textarea.innerHTML = str;
  return textarea.value;
}

export async function updateCopyLinks(el = document) {
  const codeBlocks = el.querySelectorAll("pre");

  for (const codeBlock of codeBlocks) {
    // Get the language
    const lang = codeBlock.getAttribute("data-language");

    if (lang !== null) {
      // Create the main control container (reusing .hljs__link_and_copy)
      const linkAndCopySpan = document.createElement("div"); // Changed from span to div for better flex behavior if needed
      linkAndCopySpan.className = "hljs__link_and_copy";

      const copyLinkWrapper = document.createElement("span"); // This will be the new .link_and_copy__copy_link
      copyLinkWrapper.className = "link_and_copy__copy_link pointer"; // Reuse existing class
      copyLinkWrapper.setAttribute("title", "Copy code to clipboard");

      const copyIcon = document.createElement("span");
      copyIcon.className = "action-icon"; // New class for the actual icon
      copyIcon.innerHTML = "&#x1F4CB;";
      copyLinkWrapper.appendChild(copyIcon);

      linkAndCopySpan.appendChild(copyLinkWrapper);

      const applyToFileWrapper = document.createElement("span");
      applyToFileWrapper.className = "link_and_copy__apply_link pointer"; // New class for apply button
      applyToFileWrapper.setAttribute("title", "Apply code to file");

      const fileIcon = document.createElement("span");
      fileIcon.className = "action-icon";
      fileIcon.innerHTML = "&#x2398;";
      applyToFileWrapper.appendChild(fileIcon);

      linkAndCopySpan.appendChild(applyToFileWrapper);

      // Create and set up the language text
      const langText = document.createElement("span");
      langText.className = "link_and_copy__text";
      langText.textContent = lang;
      linkAndCopySpan.appendChild(langText);

      // Append the main controls container to the pre element
      codeBlock.appendChild(linkAndCopySpan);

      // Add click event listener to the copy button
      copyLinkWrapper.addEventListener("click", function handleClick(event) {
        event.preventDefault();
        event.stopPropagation(); // Prevent propagation if there are other listeners on parent

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
          copyIcon.innerHTML = "&#x2398;"; // Original icon (your chosen one for copy)
          copyIcon.style.color = ""; // Reset color
        }, 2000);
      });

      // Add click event listener to the apply to file button
      applyToFileWrapper.addEventListener(
        "click",
        async function handleApplyToFile(event) {
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
            /^\s*(?:\/\/|#)\s*([a-zA-Z0-9_\-./\\]+(?:\.[a-zA-Z0-9_\-.]+)+)/;
          const match = firstLine.match(pathRegex);

          let derivedPath = null;
          if (match && match[1]) {
            derivedPath = match[1].trim();
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

          const projectName = document.getElementById("project-name").value;

          try {
            const response = await fetch("/apply-code-to-file", {
              method: "POST",
              headers: {
                "Content-Type": "application/json",
              },
              body: JSON.stringify({
                project: projectName,
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
              fileIcon.innerHTML = "&#x2398;"; // Original icon (your chosen one for apply)
              fileIcon.style.color = ""; // Reset color
            }, 2000);
          }
        }
      );
    }
  }
}

// Fallback for older browsers for copy to clipboard
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
