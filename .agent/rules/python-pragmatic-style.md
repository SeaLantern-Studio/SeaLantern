---
trigger: model_decision
description: When writing Python code
---

**Rule Content:**

You are an AI coding partner for a solo developer. Your goal is to write clean, pragmatic, and highly maintainable Python code. Prioritize development speed, readability, and practical IDE support over rigid enterprise architectural patterns.

### 1. Pragmatic Formatting (PEP 8 Lite)
* **Core Conventions:** Follow fundamental PEP 8 conventions (4 spaces for indentation, `snake_case` for variables/functions, `PascalCase` for classes).
* **Readability First:** Keep code readable, but do not enforce strict line-length limits if breaking the line makes it harder to read.
* **Import Grouping:** Group imports simply: standard library, third-party packages, and local modules.

### 2. High-ROI Type Hinting
* **Signature Annotations:** Add clear type hints to function signatures (arguments and return types) to maximize autocompletion and static analysis in tools like PyCharm and Neovim.
* **Local Variables:** Do not strictly type obvious local variables unless the context is highly ambiguous.

### 3. Lean Documentation
* **Docstrings:** Write brief docstrings only for core classes or complex utilities. Skip boilerplate documentation for obvious, self-explanatory functions.
* **Inline Comments:** Use inline comments sparingly, and strictly to explain the "WHY" behind a non-obvious decision or workaround, never the "WHAT".

### 4. Flat & Simple Logic
* **Guard Clauses:** Keep functions small and focused. Use early returns (guard clauses) to avoid deep nested `if-else` blocks.
* **Context Managers:** Always use context managers (`with` statements) for file operations and resource management.
* **Simplicity:** Favour built-in standard libraries and straightforward comprehensions over complex, over-engineered abstractions.

### 5. Practical Error Handling
* **Specific Exceptions:** Catch specific exceptions (e.g., `ValueError`, `KeyError`) instead of using bare `except:` blocks.
* **Actionable Messages:** Provide clear, actionable error messages for fast debugging, but avoid building heavy custom exception hierarchies unless necessary for the core logic.