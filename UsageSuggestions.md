## AI Code Assistant: Best Practice Guidelines

Here are some best practices to get the most accurate, relevant, and efficient results from the AI Code Assistant.

### 1. Let Context Guide Syntax, Let Prompts Guide Logic 🚀

This is the most powerful workflow:

* **For Syntax & Patterns (The "What"):** Let your code do the talking. The tool sends the *current state* of your files to the LLM. If you provide files using Svelte 5 runes, the AI will see and might replicate that pattern. Don't specify "please use Svelte 5" if the code you provide already reflects it.

* **For Optimal Design (The "How"):** **Omit specific versioning or syntax constraints in your initial request.** Focus purely on the **feature, goal, or design pattern** you want.

    * **Goal:** "Refactor this component to better handle asynchronous state using a custom store pattern."
    * **AI Output:** The AI may respond with an optimal, modern approach (e.g., using Svelte 5 runes or modern React Hooks patterns), even if your input code was based on older syntax (e.g., Svelte `export let` or class components).

* **Refinement (The Clean-up):** If the AI provides the optimal pattern but uses syntax incompatible with your current file context, perform the simple syntax conversion yourself, or make a follow-up request.

    * **Follow-up Prompt:** "The pattern is perfect, but my project is currently locked to **Svelte 4**. Please convert the provided code from runes back to the Svelte 4 `$store.subscribe` pattern."

This process ensures the AI remains focused on **high-level, architectural quality** and avoids getting bogged down in minor version-specific syntax details, which are often easier for you to handle manually or in a quick, targeted follow-up.

### 2. Curate Your Context: The AI Has a Limited Attention Span

"Only show the AI the files it needs" is the golden rule.

* **Why?** A flooded context (too many files) dilutes the AI's focus. It might pull unrelated patterns or get confused about the primary goal.
* **Best Practice:** Use the vector search query to find a strong starting point. Then, manually add/remove files from the chat context to keep the AI focused *only* on the files necessary for the specific feature you are building.

### 3. Start Fresh: A New Query for a New Feature

This is a core part of the tool's design.

* **The Workflow:** In the AI Code Assistant, every new chat session is born from a **query**. This query runs a semantic search on your project to find the most relevant files for your task, and then creates a new, isolated chat based on those files.
* **Best Practice:** Embrace this workflow. Don't try to pivot tasks mid-chat. If you've finished adding a feature and want to fix an unrelated bug, don't just change the subject.
* **How to Do It:** Finish your current task. Then, go back to the project page and start a *new query* related to the bug. This ensures the AI gets a fresh, relevant set of files and a clean history, leading to much more accurate results.

### 4. Git is Your Best Friend: Branch, Chat, Merge

Use the tool's Git integration to its full potential.

* **1. Branch Before You Chat:** Before starting a new feature, use the UI to **create a new branch** (e.g., `feature/add-user-login`). This isolates your work.
* **2. Enable Auto-Commit:** For most chats, **turn on the auto-commit option.** This creates an "iterative save history." As you and the AI work, each change is committed to your feature branch, providing an automatic, detailed log of the development process.
* **3. Merge When Done:** Once the feature is complete and working, you can merge that clean, fully-committed branch back into your main branch.

### 5. Be Smart With Errors: Save Credits, Be Verbose When Stuck

Don't use an LLM as your primary linter.

* **1. Fix Simple Errors Yourself:** If you see a red underline from a linter or a simple compiler error (e.g., a missing semicolon), **fix it yourself.** It's faster and saves your LLM credits for the hard problems.
* **2. When Stuck, Over-Explain:** If you're genuinely stuck on a complex bug (a logic error, a confusing framework issue), *then* ask the AI. When you do, be verbose.
    * **Good Error Prompt:** "I'm getting this error: `[paste error here]`. I think it's happening in `FileA.svelte` in the `handle_submit` function. I've already tried `[what you tried]`, but it's still not working. Here are the relevant files."


### 6. You Are the Senior Dev, The AI is Your Pair Programmer

The AI is a powerful collaborator, not an infallible oracle. You are still in charge.

* **Trust, but Verify:** The AI is brilliant at generating boilerplate, finding patterns, and suggesting solutions. However, it doesn't understand your *business goals*.
* **Your Role:** You are responsible for validating the code. The AI might produce code that *runs* but has the wrong *logic* or misses a critical security edge case. Always review and test its suggestions.

### 7. Break Down "Impossible" Tasks

Don't ask the AI to do too much at once. An "impossible" prompt will get a generic or wrong answer.

* **Avoid:** "Build the entire user authentication system."
* **Instead:** Break it down into the same steps *you* would follow, and use the AI for each one. This also works perfectly with the "New Query for a New Feature" model.
    1.  *Query 1:* "Help me design the database schema for a 'users' table with password hashing."
    2.  *Query 2:* "Write the API endpoint for user registration using this schema." (Provide schema in context).
    3.  *Query 3:* "Create the Svelte component for the registration form that calls this API endpoint."

### 8. Always Explain "Why" in Your Prompts

Don't just give orders; provide intent. This helps the AI make better "in-between" decisions when the code isn't perfectly clear.

* **Bad:** "Add an 'is_admin' check to this function."
* **Good:** "Add an 'is_admin' check before the 'delete_post' function. **This is a security measure to ensure only admins can delete posts.**"

This context helps the AI understand the *goal* (security) and not just the *task* (add a line of code).