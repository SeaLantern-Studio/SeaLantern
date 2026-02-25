# 贡献指南
[我们有了文档站!点击这里跳转!](https://docs.ideaflash.cn/zh/intro)

[We've got a document site,press here to redirect.](https://docs.ideaflash.cn/en/intro)

感谢你对 Sea Lantern 项目的关注！这份文档将帮助你了解如何为项目做出贡献。

## 开发环境要求

- **Node.js**: 20+
- **Rust**: 1.70+
- **pnpm**: 9.15.9（推荐使用项目指定的包管理器）
- **Git**: 最新版本

## 代码规范

### Rust 代码规范

1. **格式化**

   ```bash
   # 提交前必须运行
   cargo fmt --all
   ```

2. **代码检查**

   ```bash
   # 必须通过所有 clippy 检查
   cargo clippy --workspace -- -D warnings
   ```

3. **命名规范**
   - 文件名：使用 `snake_case`（如 `server_manager.rs`）
   - 函数名：使用 `snake_case`（如 `get_server_list`）
   - 结构体：使用 `PascalCase`（如 `ServerInstance`）
   - 常量：使用 `SCREAMING_SNAKE_CASE`（如 `MAX_MEMORY`）

4. **注释规范**
   - 公共 API 必须有文档注释（`///`）
   - 复杂逻辑需要添加行内注释（`//`）
   - 避免无意义的注释

5. **错误处理**
   - 使用 `Result<T, String>` 返回错误
   - 错误信息要清晰、用户友好
   - 避免使用 `unwrap()`，优先使用 `?` 或 `unwrap_or`

### 前端代码规范

1. **Vue 组件**
   - 组件名使用 `PascalCase`（如 `ServerCard.vue`）
   - 使用 `<script setup>` 语法
   - Props 和 emits 必须定义类型

2. **TypeScript**
   - 启用严格模式
   - 避免使用 `any`，优先使用具体类型
   - 接口名使用 `PascalCase`

3. **样式**
   - 使用 CSS 变量（`var(--sl-*)`）
   - 避免硬编码颜色值
   - 使用 scoped 样式

4. **格式化和检查**

   ```bash
   # 格式化代码
   pnpm run fmt

   # 检查格式
   pnpm run fmt:check

   # Lint 检查
   pnpm run lint

   # 自动修复 Lint 问题
   pnpm run lint:fix
   ```

5. **变量引用检查**
   - 声明变量时指定明确类型
   - 避免使用 `any`，使用具体类型
   - 使用 `ref<T>` 或 `reactive` 时指定泛型类型

### UI 组件与图标

- **优先使用 Headless UI（Vue v1）与按需导入的图标库（如 Lucide）**：
  - Headless UI（https://headlessui.com/v1/vue）提供无样式、可访问性的交互组件（如 `Listbox`、`Disclosure`、`Dialog`），推荐在需要复杂交互（弹出、折叠、可访问键盘支持）时优先复用它们，以减少手写 DOM 与键盘/ARIA 处理。
  - 图标使用可按需导入的组件库（如 Lucide：https://lucide.dev/icons/ 或示例图标 https://lucide.dev/icons/paint-roller?search=Palette），不要在项目中大量硬编码 `<svg>` 路径或重复 DOM。按需导入能保持包体积小并提高可维护性。
  - 推荐实践：使用 Headless UI 的 `Listbox` 替换自定义下拉/选择器；使用 Lucide 的图标组件（如 `Palette` / `Paint-roller`）替换硬编码图标。

  这样可减少冗余 DOM、统一可访问性处理，并把样式与行为分离，便于维护。

## Git 工作流

### 分支命名

- `feature/功能名` - 新功能
- `fix/问题描述` - Bug 修复
- `chore/任务描述` - 杂项任务
- `docs/文档说明` - 文档更新

### Commit 规范

使用约定式提交（Conventional Commits）：

```
<类型>: <简短描述>

<详细描述>（可选）

Co-Authored-By: 贡献者名 <email>
```

**类型**：

- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建/工具链相关

**示例**：

```
feat: 添加服务器备份功能

- 实现增量备份
- 支持自动备份计划
- 添加备份恢复功能
```

### Pull Request 流程

1. **Fork 项目并创建分支**

   ```bash
   git checkout -b feature/your-feature
   ```

2. **开发并提交**

   ```bash
   # 确保代码通过检查
   cargo fmt --all
   cargo clippy --workspace -- -D warnings
   pnpm run fmt
   pnpm run lint
   pnpm run build

   # 提交变更
   git add .
   git commit -m "feat: 你的功能描述"
   ```

3. **推送并创建 PR**

   ```bash
   git push origin feature/your-feature
   ```

4. **PR 标题和描述**
   - 标题简洁明了（不超过 70 字符）
   - 描述包含：
     - 变更摘要
     - 测试方法
     - 相关 Issue（如有）

## 代码审查标准

### 必须满足

- ✅ 通过所有 CI 检查
- ✅ 代码格式正确（cargo fmt / oxfmt）
- ✅ 无 clippy 警告
- ✅ 无 oxlint 警告
- ✅ 功能完整且可用
- ✅ 无明显的性能问题

### 建议满足

- 有适当的注释
- 有相关测试（如适用）
- 更新了相关文档
- UI 变更符合设计规范

## 常见问题

### 如何运行开发环境？

```bash
pnpm install
pnpm run tauri dev
```

### 如何构建发布版本？

```bash
pnpm run tauri build
```

### Clippy 检查失败怎么办？

1. 查看具体警告信息
2. 运行 `cargo clippy --fix` 自动修复（部分问题）
3. 手动修复剩余问题
4. 如果某些警告不合理，可以使用 `#[allow(clippy::...)]` 标记

### 格式化检查失败怎么办？

```bash
cargo fmt --all
```

## 获取帮助

- 在 Issue 中提问
- 联系维护者
- 查看项目文档

## 行为准则

- 尊重所有贡献者
- 保持友好和专业
- 接受建设性的反馈
- 帮助新手贡献者

---

再次感谢你的贡献！
