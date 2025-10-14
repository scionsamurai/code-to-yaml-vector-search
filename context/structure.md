## 🧩 **Architecture Overview**

### **1. Entry Point & Server Setup**

* **`src/main.rs`**

  * Launches the **Actix-Web** server.
  * Configures shared `AppState` (Project, Qdrant client, LLM clients, etc.).
  * Registers all route modules from `src/routes/mod.rs`.

---

### **2. Core Models**

* **`src/models.rs`** — Core structs for `Project`, `QueryData`, `ChatMessage`, and `EmbeddingMetadata`.
* **`src/models/utils.rs`** — Helper utilities for model serialization, path resolution, and validation.

---

### **3. Route Organization**

#### **Top-level: `src/routes/mod.rs`**

Central routing hub — wires together all domain route modules.

#### **A. Git Routes**

`src/routes/git/*`
CRUD-like API over local git repositories for each project:

* Checkout, create, merge, push, commit, and status endpoints.
* Used by the UI to manipulate branches directly.

#### **B. LLM Routes**

`src/routes/llm/*`
Handles all **AI-driven analysis, chat, and YAML regeneration** features:

* `analyze_query.rs` — Handles LLM query analysis.
* `chat_analysis/*` — Rich interactive chat interface with code editing & branch management.
* `search_files.rs`, `suggest_split.rs`, `regenerate_yaml.rs` — Smart assistants for code search and refactoring.
* Integrates with `LlmService` (under `/services/llm_service.rs`).

#### **C. Project Routes**

`src/routes/project/*`
Core project lifecycle endpoints:

* Create, delete, retrieve, and update projects.
* Handles YAML overrides, clustering, and Git environment settings.
* `update_yaml.rs` and `update_file_yaml_override.rs` support regeneration & override management.

#### **D. Query Routes**

`src/routes/query/*`
Small module managing per-query configuration (e.g. auto-commit).

#### **E. UI Routes**

`src/routes/ui/*`
Renders web pages (HTML templates served from `/static`):

* `home.rs` → dashboard for projects.
* `update_env.rs` → edit environment variables via UI.

---

### **4. Services Layer**

`src/services/*`
Implements backend logic behind routes — a clean separation of concerns.

#### **A. Git, LLM, Qdrant, File, Project, Search**

* `git_service.rs` — Thin wrapper around Git CLI (checkout, commit, merge, etc.).
* `llm_service.rs` — Abstraction over OpenAI/Anthropic/Gemini APIs.
* `qdrant_service.rs` — Manages vector embeddings, similarity search, and collection setup.
* `file/*` — File management (reading, import extraction, YAML updates).
* `project_service/*` — Handles persistence of projects, chat management, and query data.
* `search_service.rs` — Semantic search across embedded code/YAML.

#### **B. Template Rendering**

`src/services/template/*`
Renders HTML (server-side templates) using project and LLM data:

* File graph visualization, search results, project page, and analysis query pages.

#### **C. YAML Management**

`src/services/yaml/*`
The backbone of this repo’s **code-to-YAML** concept:

* `management/` — YAML generation, embedding updates, cleanup.
* `processing/` — Parsing, HTML conversion, orphan file handling.
* Bridges code files and their vector representations in Qdrant.

#### **D. Clustering & Utilities**

* `clustering_service.rs` — K-means clustering on embeddings.
* `utils/html_utils.rs` — Escaping and formatting HTML safely.

---

### **5. Frontend (Static Assets)**

`/static/` folder holds all JavaScript and CSS for the web UI:

* Modularized per feature: `home`, `project`, `analyze-query`, etc.
* Heavy use of **AJAX + Fetch API** to hit Actix routes.
* Supports real-time LLM chat (`analyze-query/*` scripts).

---

### **6. Shared Config**

* **`src/shared.rs`** — Constants and shared type aliases.

---

## ⚙️ **Data Flow**

```
Browser UI (JS) 
    ↓ REST / JSON
Actix Routes (src/routes/*)
    ↓
Services Layer (src/services/*)
    ↓
Qdrant / LLM APIs / Filesystem
```

---

## 🧠 **Key Design Ideas**

* **Separation of concerns:** Each feature domain (Git, LLM, Project, YAML) has a dedicated module tree.
* **Extendable LLM layer:** You can plug in other models via `LlmService`.
* **Tight Git integration:** Users can iteratively refactor and commit AI-generated edits.
* **Vector search pipeline:** Code → YAML → Embedding → Qdrant → Semantic retrieval.
