### Concise Structural Analysis Summary & "Rules File" Foundations

This application demonstrates a **well-defined, layered architecture** built with Actix-web in Rust, clearly separating concerns into `routes` (HTTP endpoints), `services` (business logic, orchestration, external integrations), and `models` (data structures).

**Foundational Structural "Rules" (Strengths to Maintain):**

1.  **Layered Responsibilities (and Allowed Dependencies):**
    *   **Routes are Thin Controllers:** Route handlers (in `src/routes/`) should primarily focus on HTTP request/response handling, input deserialization, and delegating complex business logic to `services`. **Routes *can* depend on `services` for business logic and *may* access `AppState` for global configuration, but they should *not* directly depend on `models` for data access or manipulation.**
    *   **Services Encapsulate Business Logic:** `src/services/` modules should contain the core business logic, orchestrate interactions between other services, and manage integrations with external systems (LLMs, Qdrant). **Services *can* depend on `models` for data structures and *may* depend on other services for orchestration, but they should avoid depending directly on `routes`.**
    *   **Models are Data Structures:** `src/models/` should define the core data entities and their immediate, self-contained properties or derived calculations. **Models should be passive data containers and should *not* depend on `services` or `routes`.**

2.  **Clear Module Organization (`mod.rs`):**
    *   Use `mod.rs` files extensively (`src/routes/mod.rs`, `src/services/mod.rs`, `src/models/mod.rs`, and their subdirectories) to aggregate and expose sub-modules. This creates a highly organized, navigable, and cohesive structure.
    *   Nested modules (e.g., `src/services/file/extract_imports/`) are excellent for organizing specialized, language-specific, or domain-specific logic.

3.  **Intuitive Directory & File Naming:**
    *   Directory names (e.g., `project`, `llm`, `ui` within `routes`; `file`, `yaml`, `template` within `services`) should clearly indicate the functional area they contain.
    *   File names (e.g., `create.rs`, `analyze_query.rs`, `project_service.rs`) should be descriptive and directly hint at the file's primary content or purpose. Your project generally excels at this, making it easy to locate specific functionalities.

4. Try to keep files under 200 lines.
