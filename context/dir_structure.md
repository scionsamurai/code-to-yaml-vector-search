## 🧱 Phase 3: Global Directory Structure

### ✅ Outline Details

* Build a global directory layout for the entire application
* Base it on tech stack and all dev phases
* Include all folders and files, each with a short description

### 🧰 Prompt Template

'''plaintext
Now that we’ve defined the full project and dev phases, please generate a **global directory structure** for the entire application.

Use the following stack:
- Backend: [BACKEND_STACK]
- Frontend: [FRONTEND_STACK]

Include directories and major files needed across all phases. Add brief comments explaining the purpose of each folder or file.
'''

### 🧭 Guidance

* Show a tree with comments or notes:

  '''plaintext
  /project-root
    /backend           # API and DB logic
    /frontend          # Web UI
    /shared            # Shared types/helpers
    /config            # CI/CD, env, etc.
    ...
  '''
* Allow edit and feedback loop before finalizing
* Save the result as the canonical source of truth (`globalDirectoryStructure`)
* This will be **referenced in each later phase**
