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

export function decodeHtmlEntities(str) {
  const textarea = document.createElement("textarea");
  textarea.innerHTML = str;
  return textarea.value;
}