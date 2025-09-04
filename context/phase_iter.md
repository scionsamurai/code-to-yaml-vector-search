## 🔁 Phase 4+: Iterative Phase Implementation

**Repeat for each phase created in Phase 2**

Each dev phase enters a detailed, collaborative loop with the LLM:

---

### 🧩 Phase \[N]: \[Phase Name] Implementation

#### ✅ Outline Details

* Re-focus only on this single dev phase (from Phase 2)
* Begin by asking which **design patterns** apply to this phase
* Then narrow down the **directory structure parts** relevant to this phase
* Ask the user if edits are needed to tailor these files/folders
* Finally, open a **chat-based implementation session** to apply all the decisions

---

#### Step 1: Ask About Design Patterns

##### 🧰 Prompt Template

'''plaintext
We’re beginning development on the “[PHASE_NAME]” phase.

Its goals are:
- [OBJECTIVE]
- [TASK_1], [TASK_2], ...

What design patterns would be useful in this phase? Prioritize those that improve:
- [ ] Testability
- [ ] Modularity
- [ ] Maintainability
- [ ] Performance
'''

##### 🧭 Guidance

* Allow user to check which goals are important
* Show a generated list of design patterns
* Link patterns back to tasks or files where they’ll be used

---

#### Step 2: Extract Relevant Directory Structure

##### 🧰 Prompt Template

'''plaintext
Given the global directory structure, extract only the parts relevant to this phase: “[PHASE_NAME]”.

Include:
- Files/folders to create or modify
- File descriptions or comments
- References to where logic should go (e.g. inside `/api/auth/handlers.ts`)
'''

##### 🧭 Guidance

* Auto-extract a filtered tree view:

  * Show only relevant folders/files
  * Grey out irrelevant parts
* Highlight where new logic will be added
* Allow user to confirm or modify this substructure

---

#### Step 3: Ask for Updates Before Coding

##### 🧰 Prompt Template

'''plaintext
Here’s the proposed directory structure for this phase. Would you like to:
- Rename folders/files?
- Add or remove anything?
- Change file purposes?

Once confirmed, we’ll move into live collaboration to write or update the code.
'''

##### 🧭 Guidance

* Treat this as a checkpoint
* If the user changes something, update both:

  * `globalDirectoryStructure`
  * The per-phase `workingStructure`
* Once approved, move into chat

---

#### Step 4: Enter LLM Chat for Code Generation

##### 🧰 Prompt Template (LLM entry point)

'''plaintext
We are now beginning implementation for the “[PHASE_NAME]” phase.

The relevant folders/files are:
- [LIST OF FILES AND THEIR PURPOSES]

Design patterns selected:
- [LIST OF PATTERNS]

Tasks to implement:
- [LIST OF TASKS]

Let’s work step by step. Ask me questions, generate code, and let’s iterate together.
'''

##### 🧭 Guidance

* This should be a real-time conversational loop
* Let the LLM suggest:

  * File skeletons
  * Function signatures
  * Flow diagrams or architecture previews
* Allow the user to:

  * Approve, edit, or reject generated code
  * Move between files or tasks
* Save outputs to disk or a preview panel as needed
