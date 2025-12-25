// frontend/src/lib/analyze-query/syntax-highlighting.js
import { codeToHtml } from "shiki";

export function decodeHtmlEntities(str) {
  const textarea = document.createElement("textarea");
  textarea.innerHTML = str;
  return textarea.value;
}

export function highlightAction(node) {
  const process = async () => {
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
      } catch (error) {
        console.error("Error highlighting code:", error);
      }
    }
  }
};

  process();

  return {
    update() {
      process(); // Re-run if message content updates
    }
  };
}