# Project YAML Generator

Project YAML Generator is a Rust web application that allows users to generate and manage YAML representations of their source code projects. It provides a convenient way to convert source code files into a structured format, which can be useful for documentation, analysis, or other use cases involving structured code representations.

## Features

- **YAML Management**: Create, retrieve, update, and delete YAML generations through API endpoints.
- **Source Code Conversion**: Read source code files from specified directories and convert their contents to YAML format using selected language models.
- **YAML Generation**: Generate YAML files for each project and store them in a dedicated folder within the application's centralized output directory.
- **YAML Viewing**: Dedicated page for each project to view the generated YAML representations of all converted files.
- **Selective Regeneration**: Ability to selectively regenerate the YAML for individual files within a project.

## Installation

1. Clone the repository: `git clone https://github.com/scionsamurai/code-to-yaml-generator.git`
2. Navigate to the project directory: `cd code-to-yaml-generator`

## Usage

1. Run the application: `cargo run`
1.5 Add API keys to ".env" file or add them with webpage in step 2.5. See [llm_api_access](https://crates.io/crates/llm_api_access) for example ".env" file structure.
2. Access the web interface at `http://localhost:8080`
2.5 Add API keys by clicking "Update Environment Variables" button on home page (if keys weren't already added in step 1.5)
3. Create a new project by providing the project name, source directory path, and programming languages used.
4. The application will convert the source code files with specified extensions to YAML format and generate YAML files in the output directory.
5. View the generated YAML representations for each project on the dedicated project page.
6. Regenerate YAML for individual files as needed.

## Configuration

The application can be configured through environment variables or a `.env` file in the project root directory. The following variables are available:

- `OPENAI_API_KEY`: The API key for the OpenAI language model (if used).
- `OPEN_AI_ORG`: The organization ID for the OpenAI API (if used).
- `GEMINI_API_KEY`: The API key for the Gemini language model (if used).
- `ANTHROPIC_API_KEY`: The API key for the Anthropic language model (if used).

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any bug fixes, improvements, or new features.

## License

This project is licensed under the [MIT License](LICENSE).