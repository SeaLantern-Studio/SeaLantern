# 贡献指南

[我们有了文档站!点击这里跳转!](https://docs.ideaflash.cn/zh/intro)

[We've got a document site,press here to redirect.](https://docs.ideaflash.cn/en/intro)

感谢你对 Sea Lantern 项目的关注！这份文档将帮助你了解如何为项目做出贡献。

## ⭐可接受PR贡献的范围

对于非项目组织成员，你的PR可贡献范围如下

1. 对于已获得accepted标签的issue，你可以提交PR
2. 文档、i18n方向且改动极小的

⛔对于在可贡献范围以外的PR，项目组**有权直接拒绝**

### 示例

| <img width="677" height="113" alt="image" src="https://github.com/user-attachments/assets/c79127c5-0193-4a4d-9b91-de962b59f540" /> |
| ---------------------------------------------------------------------------------------------------------------------------------- |

### 为什么？

此举是为了保证贡献范围可控以及你的贡献方向不与开发组的计划相悖

我们的原则在于，**任何贡献对项目的价值都应大于审查它所需的工作量**

请与项目组保持良好的沟通

> 满足以下条件的，你可能会被邀请至开发组：
>
> 1.  贡献代码行数(仅算增加行) > 1000的
> 2.  基于其他方面考虑, 获得开发组主动邀请的

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

- `feat/功能名` - 新功能
- `fix/问题描述` - Bug 修复
- `refactor/任务描述` - 重构
- `chore/任务描述` - 杂项任务
- `docs/文档说明` - 文档更新

### Commit 规范

本仓库强制使用约定式提交（Conventional Commits），格式如下：

```
<type>: <subject>
<type>(scope): <subject>

Co-Authored-By: 贡献者名 <email>
```

其中 `type` 必须是小写，并且只能使用：

- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建/工具链相关
- `revert`: 回滚提交
- `security`: 安全修复

提交前会自动执行以下检查：

- `pre-commit`：运行 `lint-staged`，自动格式化暂存区前端文件（`oxfmt`）
- `commit-msg`：运行 `commitlint`，校验提交信息格式
- CI：再次校验提交信息，避免 `--no-verify` 绕过本地检查

**示例（符合规范）**：

```
feat(plugin): 增加插件下载重试机制
fix(server): 修复开服路径识别异常
chore(ci): 调整工作流缓存策略
docs(contributing): 更新提交规范说明
```

### 提交被拦截时如何处理

1. 看到 `commit-msg script failed`：说明提交信息不符合规范，按提示改为 `type: 描述` 或 `type(scope): 描述`。
2. 看到 `pre-commit` 执行后有文件变化：重新 `git add` 后再次提交（因为格式化可能修改了暂存文件）。
3. 想提前自检：运行 `pnpm run fmt:check && pnpm run lint`，再执行 `git commit`。

### Pull Request 流程

1. **Fork 项目并创建分支**

   ```bash
   git checkout -b feat/your-feature
   ```

2. **开发并提交**

   ```bash
   # 确保代码通过检查
   cargo fmt --all -- --check
   cargo clippy --workspace -- -D warnings
   pnpm run fmt:check
   pnpm run lint
   pnpm run build

   # 提交变更（commit-msg 会自动校验）
   git add .
   git commit -m "feat(scope): 你的功能描述"
   ```

3. **推送并创建 PR**

   ```bash
   git push origin feat/your-feature
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
