// static/analyze-query/syntax-highlighting.js
import { codeToHtml } from 'https://esm.sh/shiki@3.0.0';

export async function applySyntaxHighlighting(div=document) {
    
    const codeBlocks = div.querySelectorAll('.shiki-block');
    
    for (const block of codeBlocks) {
        const language = block.getAttribute('data-language') || 'plaintext';
        const originalCode2 = decodeURIComponent(block.getAttribute('data-original-code'));
        const originalCode = decodeHtmlEntities(originalCode2);
        
        if (originalCode) {
            try {
                const highlightedHtml = await codeToHtml(originalCode, {
                    lang: language,
                    theme: 'github-dark'
                });
                
                // Replace the content while preserving the data attributes
                block.innerHTML = highlightedHtml;
                
                // Add classes for styling
                block.classList.add('highlighted');
            } catch (error) {
                console.error('Error highlighting code:', error);
            }
        }
    }
}

function decodeHtmlEntities(str) {
    const textarea = document.createElement('textarea');
    textarea.innerHTML = str;
    return textarea.value;
}

export async function updateCopyLinks(el=document) {
	// Select all pre elements
	const codeBlocks = el.querySelectorAll('pre')

	// Loop through each pre element
	for (const codeBlock of codeBlocks) {
		// Get the language from the data-lang attribute
		const lang = codeBlock.getAttribute('data-language')
		if (lang !== null) {
			// Create the new span element
			const linkAndCopySpan = document.createElement('span')
			linkAndCopySpan.className = 'hljs__link_and_copy'

			// Create and set up the copy link
			const copyLink = document.createElement('span')
			copyLink.className = 'link_and_copy__copy_link pointer'
			copyLink.innerHTML = '&#x2398;&nbsp;'

			// Create and set up the language text
			const langText = document.createElement('span')
			langText.className = 'link_and_copy__text'
			langText.textContent = lang

			// Append the copy link and language text to the main span
			linkAndCopySpan.appendChild(copyLink)
			linkAndCopySpan.appendChild(langText)

			// Append the main span to the pre element
			codeBlock.appendChild(linkAndCopySpan)

			// Add click event listener to the copy link
			copyLink.addEventListener('click', function handleClick(event) {
				event.preventDefault()

				// Get the code content (excluding the newly added span)
				const codeContent = codeBlock.cloneNode(true)
				const linkAndCopySpan = codeContent.querySelector('.hljs__link_and_copy')
				if (linkAndCopySpan) {
					linkAndCopySpan.remove()
				}
				const cleanCodeContent = codeContent.textContent.trim()

				// Copy the content to the clipboard
				if (navigator.clipboard) {
					navigator.clipboard.writeText(cleanCodeContent).then(
						() => console.log('Code copied successfully!'),
						(error) => {
							console.error('Error copying code:', error)
							fallbackCopyTextToClipboard(cleanCodeContent)
						}
					)
				} else {
					// Fallback for browsers that do not support navigator.clipboard
					fallbackCopyTextToClipboard(cleanCodeContent)
				}

				copyLink.innerHTML = '&#x2713;&nbsp;'
				setTimeout(() => (copyLink.innerHTML = '&#x2398;&nbsp;'), 2000)
			})
		}
	}
}