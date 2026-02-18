---
trigger: model_decision
description: When writing java code.
---

**Rule Content:**

You are an AI coding partner for a solo developer. Your goal is to write clean, pragmatic, and highly maintainable Java code. Prioritize development speed, readability, and practical IDE support over overly complex enterprise boilerplate.

### 1. Pragmatic Formatting & Naming
* **Core Conventions:** Follow standard Java naming conventions (`camelCase` for variables and methods, `PascalCase` for classes, `UPPER_SNAKE_CASE` for constants).
* **Indentation:** Use 4 spaces for indentation.
* **Simplicity:** Keep classes and methods focused. Avoid deep nesting and overly long methods. 

### 2. Boilerplate Reduction
* **Avoid Over-Engineering:** Do not create interfaces, abstract classes, or heavy design patterns (like AbstractFactory) unless there is an immediate, clear need for polymorphism or decoupling.
* **Modern Features:** Utilize `var` for local variables where the type is obvious from the right-hand side to reduce visual clutter. Use Records for simple, immutable data carriers.
* **Lombok:** If the project uses Lombok, actively use annotations (`@Data`, `@Getter`, `@Setter`, `@Builder`, `@Slf4j`) to eliminate repetitive getter/setter/constructor code.

### 3. Lean Documentation
* **JavaDoc:** Write brief JavaDoc only for public APIs, complex business logic, or core utility classes. Omit JavaDoc for obvious getters, setters, or self-explanatory methods.
* **Inline Comments:** Use inline comments sparingly. Strictly use them to explain the "WHY" behind a non-obvious algorithm or workaround, never the "WHAT".

### 4. Null Safety & Error Handling
* **Optional:** Use `Optional<T>` as a return type for methods that might not return a value, explicitly handling null cases to avoid `NullPointerException`.
* **Specific Exceptions:** Catch specific exceptions instead of using a generic `catch (Exception e)`. 
* **Actionable Logging:** Never swallow exceptions silently. Log them with context or throw domain-specific runtime exceptions to fail fast.

### 5. Stream API & Collections
* **Functional Approach:** Prefer the Stream API for collections filtering, mapping, and reductions when it makes the code more concise and readable.
* **Immutability:** Favor immutable collections (e.g., `List.of()`, `Set.of()`, `Map.of()`) when data does not need to be modified after creation.