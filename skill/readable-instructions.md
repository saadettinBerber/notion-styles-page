---
name: readable-instructions
description: "Write or reformat Markdown instructions into short, scannable rules with grouped, indented bullets. Use for agent instructions, checklists and procedural guidelines, not general prose summaries."
---

# Readable Instructions

---

**Make rules easy to scan without changing their meaning.**

- **Preserve the request:**

  - Keep requirements, conditions, exceptions and scope. Do not invent rules.

  - Follow the requested language and format. Preserve existing headings and intentionally empty sections.

  - Provide previews when requested; edit files only within the authorized scope.

- **Shape the text:**

  - Open each filled section with a brief, bold guiding sentence.

  - Give each bullet one action. Keep its condition attached.

  - Group related rules under short, bold parent bullets. Indent children by two spaces.

  - Leave blank lines between bullets and put `---` below headings.

  - Keep simple lists flat. Use natural wording without fixed word limits.

Before finishing, compare meanings: preserve obligations, permissions, limits and order. More bullets may make the same content easier to read.

## Examples

---

These examples demonstrate formatting; their policies apply only to the examples.

### Split and Group

---

**Before**

```markdown
## Local Refinement

Preserve existing conventions and unaffected behavior; improve touched code in small, verified steps only when a concrete benefit supports the task.
```

**After**

```markdown
## Local Refinement

---

**Improve in small steps. Preserve behavior.**

- **When editing:**

  - Follow existing conventions.

  - Preserve unaffected behavior.

  - Improve touched code only when a concrete benefit supports the task.

  - Verify each small change.
```

### Keep Conditions

---

**Before**

```markdown
## Retries

Check the error before retrying. Retry temporary failures at most twice; report permanent failures without retrying.
```

**After**

```markdown
## Retries

---

**Check the error before retrying.**

- Retry temporary failures at most twice.

- Report permanent failures without retrying.
```
