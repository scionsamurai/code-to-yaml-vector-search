## 📋 Phase 1: Project Definition

### ✅ Outline Details

* Target audience
* Feature list
* Backend + frontend stack
* Constraints (team size, budget, deadlines, etc.)
* Optional: preferred number of dev phases

### 🧰 Prompt Template

'''plaintext
Here are the details for the project "[PROJECT_NAME]":

Target audience: [TARGET_AUDIENCE]
Main features:
- [FEATURE_1]
- [FEATURE_2]
...

Tech stack:
- Backend: [TECH_STACK_BACKEND]
- Frontend: [TECH_STACK_FRONTEND]

Constraints: [CONSTRAINTS]

Please help outline the major development phases needed to deliver this project, and suggest any missing features or adjustments to the stack.
'''

### 🧭 Guidance

* Use a multi-step form or accordion-style UI
* Inputs to collect:

  * Audience (text or dropdown)
  * Features (repeating text fields or checklist)
  * Backend/frontend stack (dropdown with multi-select or text)
  * Constraints (text area)
  * Preferred phase count (optional number input)
* Store as `projectDefinition` object to pass to later phases
