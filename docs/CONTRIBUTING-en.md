# Contributing Guidelines

Change Language(切换语言)

[简体中文](CONTRIBUTING.md)
[English version Doc](CONTRIBUTING-en.md)

Thank you for your interest in Project Sea Lantern! This document will help you understand how to contribute to the project.

## Development Environment Requirements

- **Node.js**: 20+
- **Rust**: 1.70+
- **Git**: Latest version

## Code Standards

### Rust Code Standards

1. **Formatting**

   ```bash
   # Must be run before submission
   cargo fmt --all
   ```

2. **Code Linting**

   ```bash
   # Must pass all clippy checks
   cargo clippy --workspace -- -D warnings
   ```

3. **Naming Conventions**
   - File names: Use `snake_case`(e.g., `server_manager.rs`)
   - Function names: Use `snake_case`(e.g., `get_server_list`)
   - Structs: Use `PascalCase`(e.g., `ServerInstance`)
   - Constants：Use `SCREAMING_SNAKE_CASE`(e.g., `MAX_MEMORY`)

4. **Comment Standards**
   - Public APIs must have documentation comments（`///`）
   - Complex logic requires inline comments（`//`）
   - Avoid meaningless comments

5. **Error Handling**
   - Use Result<T, String> to return errors
   - Error messages should be clear and user-friendly
   - Avoid using unwrap(), prefer ? or unwrap_or instead

### Frontend Code Standards

1. **Vue Components**
   - Component names: Use PascalCase (e.g., ServerCard.vue)
   - Use <script setup> syntax
   - Props and emits must define types

2. **TypeScript**
   - Enable strict mode
   - Avoid using any, prefer specific types
   - Interface names: Use PascalCase

3. **Styling**
   - Use CSS variables(`var(--sl-*)`)
   - Avoid hardcoded color values
   - Use scoped styles

## Git Workflow

### 分支命名

- feature/feature-name - New features
- fix/issue-description - Bug fixes
- chore/task-description - Miscellaneous tasks
- docs/documentation-description - Documentation updates

### Commit Standards

Follow Conventional Commits:

```
<type>: <short description>

<detailed description> (optional)

Co-Authored-By: Contributor Name <email>
```

**Types**：

- feat: New feature
- fix: Bug fix
- docs: Documentation update
- style: Code formatting (no functional changes)
- refactor: Code refactoring
- perf: Performance optimization
- test: Testing related changes
- chore: Build/toolchain related changes
- **Example**:

```
feat: add server backup functionality

- Implement incremental backup
- Support scheduled automatic backups
- Add backup restoration feature
```

### Pull Request Process

1. **Fork the project and create a branch**

   ```bash
   git checkout -b feature/your-feature
   ```

2. **Develop and commit changes**

   ```bash
   # Ensure code passes all checks
   cargo fmt --all
   cargo clippy --workspace -- -D warnings
   npm run build

   # Commit changes
   git add .
   git commit -m "feat: your deature description"
   ```

3. **Push and create PR**

   ```bash
   git push origin feature/your-feature
   ```

4. **PR Title and Description**
   - Title should be concise and clear (no more than 70 characters)
   - Description should include:
     - Summary of changes
     - Testing methods
     - Related Issues (if any)

## Code Review Standards

### Must Meet

- ✅ Pass all CI checks
- ✅ Correct code formatting (cargo fmt)
- ✅ No clippy warnings
- ✅ Complete and functional features
- ✅ No obvious performance issues
-

### Recommended to Meet

- Appropriate comments
- Relevant tests (if applicable)
- Updated relevant documentation
- UI changes comply with design standards

## Frequently Asked Questions

### How to run the development environment?

```bash
npm install
npm run tauri dev
```

### How to build a release version?

```bash
npm run tauri build
```

### What to do if Clippy checks fail?

1. Check the specific warning messages
   2.Run cargo clippy --fix to auto-fix (some issues)
   3.Manually fix remaining issues
   4.If some warnings are unreasonable, use #[allow(clippy::...)] annotation

### What to do if formatting checks fail?

```bash
cargo fmt --all
```

## Getting Help

- Ask questions in Issues
- Contact maintainers
- Check project documentation

## Code of Conduct

- Respect all contributors
- Maintain friendliness and professionalism
- Accept constructive feedback
- Help new contributors

---

Thank you again for your contributions!
