// static/analyze-query/utils.js
export function formatMessage(content) {
    // Convert markdown code blocks to HTML with data attributes for editing
    return content.replace(/```(\w*)([\s\S]*?)```/g, function(match, language, code) {
        // Store original code in a data attribute for edit functionality
        return `<pre class="shiki-block" data-language="${language}" data-original-code="${code}<"><code class="language-${language}">${code}</code></pre>`;
    });
}