# Project AI Code Assistant (or similar name)

## Intelligent Code Management Powered by AI

Project AI Code Assistant is a powerful Rust web application designed to help developers effortlessly understand, manage, and evolve their source code projects using cutting-edge AI. By converting your codebase into a structured YAML format and leveraging advanced vector embeddings and Large Language Models (LLMs), it provides unparalleled capabilities for code exploration, analysis, and modification.

## **Why Use Project AI Code Assistant?**

In large codebases, finding relevant information, understanding complex interactions, and making informed changes can be a significant challenge. This tool solves that by:

  * **Breaking Down Complexity:** Transforms raw code into a clean, structural YAML representation, making code intent clear.
  * **Unlocking Semantic Search:** Go beyond keyword searches to find code based on *what it does*, not just what it says.
  * **Empowering AI Assistants:** Chat with LLMs about your specific code, getting intelligent suggestions and analysis in real-time.
  * **Streamlining Code Updates:** Iteratively refine code with AI guidance, always ensuring the LLM understands the current state of your files.

## Features

  * **Automated YAML Generation:** Converts your source code files into a structured, language-agnostic YAML format, focusing on code structure, documentation, and intent (no implementation details included\!).
  * **Centralized YAML Management:** Create, retrieve, update, and delete YAML representations for your projects via intuitive API endpoints.
  * **Intelligent Code Search (Semantic Search):**
      * Leverages **vector embeddings** and a **Qdrant vector database** for highly accurate semantic search.
      * Find relevant code files across your project using natural language queries – discover code based on its purpose, not just keywords.
  * **AI-Powered Code Analysis & Chat:**
      * **Context-Aware LLM Interactions:** Engage in interactive chats with LLMs (e.g., Anthropic, OpenAI, Gemini) to analyze and discuss your code.
      * **Smart File Selection:** Vector search results provide a starting point, and the LLM can further recommend additional relevant files.
      * **Dynamic Context Control:** You have full control over which files are shown to the LLM at any point in the conversation, ensuring the LLM always has the precise context you need.
      * **Live Code Updates:** The LLM is explicitly informed that the code context can be updated with each message, enabling seamless, iterative code modification discussions.
      * **Syntax-Highlighted Responses:** LLM code suggestions are automatically parsed from Markdown and rendered with beautiful syntax highlighting for optimal readability.
  * **Project & File Management:**
      * Dedicated project pages to view generated YAML representations.
      * Selective regeneration of YAML for individual files.
      * Save and resume previous analysis chats and query histories.
  * **Integrated Git Workflow:**
      * **Streamlined Branch Management:** Easily create new feature branches directly from your chat session to isolate development, allowing you to work on significant updates without affecting your main codebase.
      * **Seamless Branch Switching:** Use the built-in branch selector to view the current active branch and switch between branches within the application.
      * **Flexible Commit Options:**
          * **Auto-Commit:** Enable per-chat auto-commits to automatically save changes between user and AI messages to the active branch, associating conversation progress directly with code milestones.
          * **Manual Commit:** Take granular control by manually committing changes with a custom message using the dedicated "Commit" button.
      * **Push to Remote:** Synchronize your local commits with your remote repository (e.g., GitHub) at any time with a single click.
      * **Effortless Merging:** Merge your feature branch back into the main development branch once your changes are complete and reviewed.
      * **Integrated Git Credentials:** Configure your Git author name, email, and remote authentication credentials (like GitHub Personal Access Tokens for HTTPS) directly through the application's UI, ensuring smooth `push` operations.
      * **Conflict Handling:** In case of merge conflicts, the application will notify you, requiring manual resolution outside the app to maintain full control over your codebase.
      * **AI's Role:** The AI assistant provides code suggestions and insights, while all Git operations (branching, committing, merging, pushing) remain fully under user control.

  * **User-Friendly Web Interface:** Access all features through a modern web application.

## Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/scionsamurai/code-to-yaml-vector-search.git
    cd code-to-yaml-vector-search
    ```
2.  **Start Qdrant Vector Database:** Project AI Code Assistant uses Qdrant for blazing-fast vector search. The easiest way to run it is with Docker:
    ```bash
    docker run -p 6334:6334 -p 6333:6333 qdrant/qdrant
    ```
## Usage

1.  **Configure API Keys:**
      * **Option 1 (Recommended):** Create a `.env` file in the project root and add your API keys (e.g., `OPENAI_API_KEY`, `GEMINI_API_KEY`, `ANTHROPIC_API_KEY`). See the `llm_api_access` crate documentation for example `.env` structure.
      * **Option 2:** Add/update keys directly through the web interface (see step 4).
2.  **Run the Application:**
    ```bash
    cargo run
    ```
3.  **Access the Web Interface:** Open your browser and navigate to `http://localhost:8080`.
4.  **(Optional) Update Environment Variables:** If you skipped step 1, click the "Update Environment Variables" button on the home page.
5.  **Create a New Project:** Provide a project name, the path to your source code directory, and the programming languages used. The application will automatically generate YAML files and their embeddings.
6.  **Explore & Analyze:**
      * View the generated YAML representations on the dedicated project page.
      * Use the **semantic search** bar to find relevant code files.
      * Initiate a **code analysis chat** to discuss and modify your codebase with the LLM.
      * Dynamically select which files are visible to the LLM within the chat interface.

## Configuration

The application can be configured using environment variables or a `.env` file in the project's root directory.

  * `OPENAI_API_KEY`: API key for OpenAI models (embeddings, chat).
  * `OPEN_AI_ORG`: Organization ID for OpenAI API (if applicable).
  * `GEMINI_API_KEY`: API key for Google Gemini models.
  * `ANTHROPIC_API_KEY`: API key for Anthropic models (default for chat analysis).
  * `QDRANT_SERVER_URL`: URL for the Qdrant vector database (defaults to `http://localhost:6334`).

## License

This project is licensed under the [MIT License](https://www.google.com/search?q=LICENSE).
