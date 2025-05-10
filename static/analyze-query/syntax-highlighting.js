// static/analyze-query/syntax-highlighting.js
import { codeToHtml } from "https://esm.sh/shiki@3.0.0";

export async function applySyntaxHighlighting() {
  // Format existing messages from Markdown to HTML
  const chatMessages = document.querySelectorAll(".chat-message");
  chatMessages.forEach((messageDiv) => {
    const messageContent = messageDiv.querySelector(".message-content");
    const originalContent =
    messageContent.innerHTML || messageContent.textContent;
    messageContent.innerHTML = marked.parse(originalContent);
  });

  const codeBlocks = document.querySelectorAll(".shiki-block");

  for (const block of codeBlocks) {
    const language = block.getAttribute("data-language") || "plaintext";
    const originalCode = decodeURIComponent(
      block.getAttribute("data-original-code")
    );

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

// Function to apply highlighting to a single message
export async function highlightMessage(messageDiv) {
  const codeBlocks = messageDiv.querySelectorAll(".shiki-block");

  for (const block of codeBlocks) {
    const language = block.getAttribute("data-language") || "plaintext";
    const originalCode = decodeURIComponent(
      block.getAttribute("data-original-code")
    );

    if (originalCode) {
      try {
        const highlightedHtml = await codeToHtml(originalCode, {
          lang: language,
          theme: "github-dark",
        });

        block.innerHTML = highlightedHtml;
        block.classList.add("highlighted");
      } catch (error) {
        console.error("Error highlighting code:", error);
      }
    }
  }
}
