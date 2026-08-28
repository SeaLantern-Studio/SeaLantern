# 贡献与验证

本页面向仓库外贡献者，所有要求都应能从公开仓库验证。涉及架构、宿主边界或插件安全模型的修改，先阅读 [当前架构](../design/architecture.md) 和[插件设计](../design/plugin.md)。

## 开发环境

以仓库当前配置为准：

| 工具    | 要求                                                   |
| ------- | ------------------------------------------------------ |
| Node.js | 24 LTS；CI 使用 LTS 通道                               |
| pnpm    | 9.15.9，版本由 `package.json` 的 `packageManager` 固定 |
| Rust    | stable，见 `rust-toolchain.toml`                       |
| Only    | 0.4.0+，可选；仅用于调用 `Onlyfile` 任务               |

安装依赖：

```bash
pnpm install
```

## 开发与检查命令

前端页面预览和 Desktop 开发：

```bash
pnpm dev
pnpm tauri dev
```

Web/Axum 宿主开发、构建和打包：

```bash
pnpm axum dev
pnpm axum build
pnpm axum package
```

如果安装了 Only，可从根目录运行：

```bash
only                 # 查看任务
only check           # 前后端检查
only test            # 前后端测试
only ci              # 完整本地 CI
only front ci        # 只检查前端
only back ci         # 只检查后端
```

不使用 Only 时，提交前至少运行与改动相关的检查：

```bash
pnpm fmt:check
pnpm lint
pnpm build:check
pnpm test

cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --all-targets --workspace
```

后端 CI 会按 workspace crate 拆分测试；涉及单个 crate 时可以先运行对应的 `cargo test -p <package>`，但这不能替代受影响的完整检查。

## 代码落位

- 新的共享业务用例进入 `application/src/service`，能力端口进入 `application/src/port`，公共 DTO 和错误进入 `crates/contract`，领域逻辑进入 `crates/core`，功能实现进入 `crates/feature`，通用技术能力进入 `crates/infra`。
- Tauri command/event 只做 Desktop 传输和宿主上下文适配；Axum handler 只做 HTTP 传输和宿主上下文适配。
- 页面通过 `src/api` 调用业务。不要在页面中复制 Rust 业务规则或手写一套只在某个宿主有效的共享逻辑。
- 新增错误优先使用现有的领域错误或具名错误类型，在宿主边界转换为适合 Tauri/HTTP 的响应；不存在统一的 `Result<String>` 规则。
- Rust 使用 `snake_case` 文件和函数名、`PascalCase` 类型名；公共 API 和非显然逻辑补充简短文档注释。
- Vue 组件使用 `<script setup>` 和 TypeScript，复用项目现有 UI 与图标依赖，不复制已经过期的组件清单。

## Pull Request

1. 一个 PR 保持一个清晰主题；架构或安全边界变化应同时更新对应 `docs/design` 文档。
2. PR 描述写明问题、处理方式、影响范围和已执行的验证。
3. 外部作者提交的 PR 需要关联当前仓库的 Issue；`issue-check.yml` 对项目成员、所有者和带 `internal` 标签的 PR 有明确豁免。
4. 只修改文档时也要检查内部链接、源码路径和命令是否仍然存在。
5. 不提交构建产物、运行时日志或根目录 `tmp/` 内容；临时文档按[文档维护规则](./documentation.md)处理。
6. 在 Review 或 PR 过程中发现 `docs/tmp` 中的文档已经过期，可以直接删除该文档；即使它与当前 PR 的主题无关，也不需要为了保留它而修改或关联当前 PR 内容。

## 不确定时

如果实现和文档不一致，先以源码、配置和 CI 的可复现行为为准，再在同一变更中修文档。不要依赖未公开的团队文档作为 PR 通过条件。
