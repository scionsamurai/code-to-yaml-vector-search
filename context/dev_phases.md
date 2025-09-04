## 🔨 Phase 2: Dev Phase Planning

### ✅ Outline Details

* Organize the project into 3–6 phases (can be adjusted)
* Each phase should include:

  * Objective
  * Key modules/tasks
  * Tools
  * Deliverables

### 🧰 Prompt Template

'''plaintext
Help break down this project into the following phases:
1. [PHASE_1]
2. [PHASE_2]
...

For each phase, list:
- Objective
- Key tasks/modules
- Recommended tools/technologies
- Expected deliverables
'''

### 🧭 Guidance

* Auto-generate the first version based on feature list
* Let users edit:

  * Phase titles
  * Add/remove phases
  * Add/assign tasks per phase
* Consider a Kanban-style or linear list editor with drag/drop
* Store phase data in an array of objects:

  '''ts
  type Phase = {
    name: string;
    objective: string;
    tasks: string[];
    tools: string[];
    deliverables: string[];
  };
  '''
