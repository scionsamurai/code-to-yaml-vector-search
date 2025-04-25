# Project YAML Generator

Project YAML Generator is a Rust web application that allows users to generate and manage YAML representations of their source code projects. It provides a convenient way to convert source code files into a structured format, which can be useful for documentation, analysis, or other use cases involving structured code representations.

## Features

- **YAML Management**: Create, retrieve, update, and delete YAML generations through API endpoints.
- **Source Code Conversion**: Read source code files from specified directories and convert their contents to YAML format using selected language models.
- **YAML Generation**: Generate YAML files for each project and store them in a dedicated folder within the application's centralized output directory.
- **YAML Viewing**: Dedicated page for each project to view the generated YAML representations of all converted files.
- **Selective Regeneration**: Ability to selectively regenerate the YAML for individual files within a project.
- **Semantic Search**: Search through project files using natural language queries via vector embeddings.
- **Code Analysis**: Use LLMs to analyze and answer questions about your code.
- **Query History**: Save previous queries and their results for future reference.

## Installation

1. Clone the repository: `git clone https://github.com/scionsamurai/code-to-yaml-vector-search.git`
2. Navigate to the project directory: `cd code-to-yaml-vector-search`
3. Install Qdrant for vector search functionality. You can use Docker:
   ```
   docker run -p 6334:6334 -p 6333:6333 qdrant/qdrant
   ```

## Usage

1. Add API keys to ".env" file now or add them with webpage in step 4. See [llm_api_access](https://crates.io/crates/llm_api_access) for example ".env" file structure.
2. Run the application: `cargo run`
3. Access the web interface at `http://localhost:8080`
4. Add API keys by clicking "Update Environment Variables" button on home page (if keys weren't already added in step 2).
5. Create a new project by providing the project name, source directory path, and programming languages used.
6. The application will convert the source code files with specified extensions to YAML format and generate YAML files in the output directory.
7. View the generated YAML representations for each project on the dedicated project page.
8. Regenerate YAML for individual files as needed.
9. Use the search functionality to find relevant code files based on natural language queries.
10. Execute queries against specific code files to get detailed analysis from LLMs.

## Configuration

The application can be configured through environment variables or a `.env` file in the project root directory. The following variables are available:

- `OPENAI_API_KEY`: The API key for the OpenAI language model (used for embeddings and optionally for analysis).
- `OPEN_AI_ORG`: The organization ID for the OpenAI API (if used).
- `GEMINI_API_KEY`: The API key for the Gemini language model (if used).
- `ANTHROPIC_API_KEY`: The API key for the Anthropic language model (if used).
- `QDRANT_SERVER_URL`: The URL for the Qdrant vector database (defaults to "http://localhost:6334" if not specified).

## Advanced Features

### Semantic Search

The application uses OpenAI embeddings and Qdrant vector database to provide powerful semantic search capabilities:

1. Enter a natural language query on a project page
2. The system generates embeddings for your query
3. The query embeddings are compared against the embeddings of your code files
4. The most semantically similar files are returned as results

### Code Analysis

You can analyze specific code files with LLMs:

1. Search for relevant files using the semantic search feature
2. Select code files to analyze
3. Enter your query about the selected code
4. The system will use an LLM (Anthropic by default) to analyze the code and provide a detailed response

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any bug fixes, improvements, or new features.

## License

This project is licensed under the [MIT License](LICENSE).