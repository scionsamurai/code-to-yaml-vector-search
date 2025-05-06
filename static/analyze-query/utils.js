// static/analyze-query/utils.js
export function formatMessage(content) {
        // Convert markdown code blocks to HTML
        return content.replace(/```(\w*)([\s\S]*?)```/g, function(match, language, code) {
            return `<pre><code class="${language}">${code}</code></pre>`;
        }).replace(/\n/g, '<br>');
}