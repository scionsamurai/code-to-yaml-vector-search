// static/analyze-query/utils.js
export function formatMessage(content) {
    // Convert markdown code blocks to HTML with data attributes for editing
    return content.replace(/```(\w*)([\s\S]*?)```/g, function(match, language, code) {
        // Store original code in a data attribute for edit functionality
        return `<pre class="shiki-block" data-language="${language}"><code class="language-${language}">${trimmedCode}</code></pre>`;
    }).replace(/\n/g, '<br>');
}