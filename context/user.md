To create universal guidelines for converting code from any language to YAML, we need to focus on common features and structures, ensuring no actual code snippets appear.

**Crucial Rule:** The generated YAML **MUST** strictly conform to the following Rust struct definitions, which define the expected YAML schema. Pay close attention to optional fields, list types, and the `#[serde(rename = "type")]` attributes which indicate the YAML key should be "type".

ReplaceWithStructCode

**General Guidelines for YAML Generation:**

1.  **File Metadata (`description`):**
    *   **Always include** a `description` field at the top of the YAML, briefly stating the file's purpose.
    ```yaml
    ---
    description: This file contains utility functions for data processing.
    ```
2.  **Function/Method Definitions:**
    *   For each function or method, include its `name`, optional `description`, optional `parameters`, optional `return_type`, and optional `calls`.
    *   **Crucial:** **Exclude** standard library calls, `new` calls, inline functions, and in-block functions from the `calls` list.
    *   **Crucial:** **Never include actual code implementation lines.**
    *   **Parameters:**
        *   If a function/method has **no parameters**, the `parameters` field **MUST be omitted entirely** (do not use `parameters: []` or `parameters: [ - none ]`).
        *   If `parameters` is present, each item in the list **MUST** be an object conforming to the `Parameter` struct (`name`, `type`, `description`).
        *   The `type` field within a `Parameter` **MUST be a simple string** (e.g., `str`, `int`, `list`, `bool`, `&str`, `&Vec<String>`, `HashMap<String, String>`). Do not use `None` or a list/map here. If the type string contains characters like `&` or `< >`, ensure they are part of a valid string (e.g., `type: "&str"`).
    *   **Return Type:**
        *   The `return_type` field **MUST be a simple string** (e.g., `void`, `str`, `list`, `Result<String, Error>`).
        *   **Do NOT** use a nested object or map for `return_type` (e.g., `return_type: { type: object, structure: ... }` is forbidden).
    ```yaml
    functions:
      - name: process_data
        parameters:
          - name: data
            type: list
            description: A list of data points to process
          - name: filter_threshold
            type: float
            description: The threshold value for filtering data
        return_type: list
        description: Filters and processes the input data based on the specified threshold.
        calls:
          - create_dir_all
          - write
      - name: calculate_sum # Example with no parameters
        return_type: int
        description: Calculates the sum of internal values.
    ```
3.  **Class Definitions:**
    *   For each class, include its `name`, optional `inherits` (string), optional `description`, optional `methods`, and optional `properties`.
    *   **Crucial:** `methods` **MUST** be a list of `Function` objects, following the same rules as standalone functions.
    *   **Crucial:** `properties` **MUST** be a list of `Parameter` objects, following the same rules as function parameters.
    *   **Never include implementation details or actual code lines.**
    ```yaml
    classes:
      - name: DataProcessor
        inherits: object
        description: A class for processing data.
        methods:
          - name: initialize
            description: Initializes the processor.
          - name: process_item
            parameters:
              - name: item
                type: any
                description: The item to process.
            return_type: bool
            description: Processes a single item.
        properties:
          - name: config_path
            type: str
            description: Path to the configuration file.
          - name: max_retries
            type: int
            description: Maximum number of retries for an operation.
    ```
4.  **Data Structures:**
    *   For complex data structures, include their `name`, `type` (a simple string like `dict`, `object`, `enum`), optional `description`, and optional `structure`.
    *   **Crucial:** If `structure` is present, it **MUST be a YAML map** where keys are field names (strings) and values are their types (strings).
    *   **Crucial:** **Do NOT include descriptions for individual fields within the `structure` map.** The `description` of the `DataStructure` itself should cover the necessary details.
    *   If the structure is very simple and fully explained by `name` and `type`, the `structure` field can be omitted.
    ```yaml
    data_structures:
      - name: Person
        type: dict
        description: A dictionary representing a person's information.
        structure:
          name: str
          age: int
          contact: dict # Nested structures are represented by their type
    ```
    *   **Example for Enum-like Data Structure:**
    ```yaml
    data_structures:
      - name: FileServiceError
        type: enum
        description: A custom error type for file service related operations.
        structure:
          TraversalAttempt: "Attempted directory traversal"
          InvalidPath: "Invalid file path"
          Io: "IO error; contains std::io::Error"
    ```
5.  **Code Comments:**
    *   Include relevant code comments as descriptions for functions, classes, variables, and other code elements, populating the `description` fields.
    *   **Never include the actual code lines these comments were attached to.**
6.  **Import Statements:**
    *   **Do NOT include import statements in the YAML output.** These are manually extracted and added separately.
7.  **Language-Specific Features:**
    *   If a particular language has unique features (e.g., decorators, traits), include guidelines for representing them in the YAML without including actual implementation code. Focus on their *metadata* and *structural impact*.
8.  **Strict No Code Rule:**
    *   **NEVER include actual source code lines, implementation details, or executable statements in the YAML output.** The YAML should only contain metadata, descriptions, and structural information about the code, not the code itself.
9.  **YAML Formatting & Validity:**
    *   Generate **valid, well-formed YAML** that can be parsed by `serde_yaml`.
    *   Ensure **correct indentation** (2 spaces for nested elements).
    *   **Avoid creating multi-document YAML** (do not use multiple `---` separators in a single file). A single `---` at the very beginning is acceptable for file metadata.
    *   **Avoid invalid characters or unescaped anchors** (e.g., `&` outside of a quoted string, or `[ ]` incorrectly placed) that would cause YAML parsing errors. Always ensure string values are properly quoted if they contain special YAML characters.
    *   **Remove any trailing whitespace** after list items or keys.

These comprehensive guidelines, combined with the explicit schema, should greatly improve the robustness and correctness of the generated YAML.