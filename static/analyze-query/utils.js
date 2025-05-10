// static/analyze-query/utils.js
export function formatMessage(content) {

  const renderer = new marked.Renderer();

  // Override the 'code' method
  renderer.code = (code) => {
    const language = code.lang || "plaintext";
    return `<pre class="shiki-block" data-language="${language}" data-original-code="${code.text.replaceAll('"', '&#34;')}"><code class="language-${language}">${code.raw}</code></pre>`;
  };

  // Configure marked to use your custom renderer
  marked.setOptions({
    renderer: renderer,
    breaks: true, 
  });

  return marked.parse(content);
}
