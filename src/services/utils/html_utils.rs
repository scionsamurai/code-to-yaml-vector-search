// src/services/utils/html_utils.rs

pub async fn escape_html(text: String) -> String {
    // Process text line by line to handle code block markers vs inline triple backticks
    let processed_text = text.lines()
        .map(|line| {
            let trimmed = line.trim();
            // If the line starts with triple backticks after trimming, leave it as is
            if trimmed.starts_with("&grave;&grave;&grave;") {
                line.to_string()
            } else {
                // Replace any triple backticks in the middle of the line
                line.replace("&grave;&grave;&grave;", "&grave;&grave;&grave;")
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    // Perform normal HTML escaping on the processed text
    html_escape::encode_text(&processed_text)
        .to_string()
        .replace("\"", "&#34;")
}

pub fn unescape_html(text: String) -> String {
    let mut unescaped_text = text.replace("&#96;&#96;&#96;", "&grave;&grave;&grave;");

    unescaped_text = unescaped_text.replace("&lt;", "<");
    unescaped_text = unescaped_text.replace("&gt;", ">");
    unescaped_text = unescaped_text.replace("&quot;", "\"");
    unescaped_text = unescaped_text.replace("&#34;", "\""); // For &#34; (double quote)
    unescaped_text = unescaped_text.replace("&#39;", "'");  // For &#39; (single quote/apostrophe)
    unescaped_text = unescaped_text.replace("&apos;", "'"); // For &apos; (named entity for apostrophe, though less common)
    unescaped_text = unescaped_text.replace("&amp;", "&"); // This MUST be last

    unescaped_text
}

