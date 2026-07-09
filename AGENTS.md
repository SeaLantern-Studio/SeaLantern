# AGENTS.md

本文件是 Sea Lantern 仓库的仓库级协作规范。

所有 AI 代理、自动化工具和协作者在修改仓库前，必须先阅读：

1. [README.md](README.md)
2. [frontend/README.md](frontend/README.md)
3. [backend/README.md](backend/README.md)

本文件是 AI/Agent 修改仓库时的硬约束。面向用户和贡献者的总览内容优先放到文档站；仓库内只保留和代码同步强相关的说明。

## 1. 工作原则

### 1.1 输出要求

- 不要空话、套话、寒暄式前摇。
- 先看当前仓库事实，再判断结构、行为和边界。
- 不要用占位实现、伪代码或无法编译的片段冒充完成。

### 1.2 修改方式

- 保持改动小、清楚、可审查。
- 优先沿用现有目录、命名、组件、服务、crate 和脚本。
- 不做无关重构、重命名、移动或重写。
- 旧说明与当前代码不一致时，以代码为准，并同步修正文档。
- 中等以上复杂任务先给出简短边界和风险，再进入实现。

### 1.3 编码底线

- 正确性优先，其次性能，最后才是开发体验；不要写炫技、过度抽象或明显 AI 味的代码。
- 编码前确认不变量、边界、失败模式和所有权；循环、缓冲、重试和批处理要有明确上限。
- 控制流保持简单直接；变量放在最小可用作用域，计算、校验和使用尽量靠近。
- 命名短、准、贴近领域；断言只用于程序员错误和重要不变量。
- 预期运行时错误必须显式处理并保留有用信息。
- 新增公共 API 要有简洁 doc comment；普通注释解释原因，不重复代码。
- 能回归的行为要补测试，优先覆盖边界、错误路径、资源清理、解析、校验和公开契约。

## 2. 仓库边界

Sea Lantern 是基于 Tauri 2、Vue 3 和 Rust 的 Minecraft 服务器管理工具。

- `frontend/`：前端界面、状态、语言、主题和 Tauri 命令调用封装。细节见 [frontend/README.md](frontend/README.md)。
- `backend/`：Rust workspace。共享规则放在 `*-core` crate，宿主编排放在 `backend/tauri-host`。细节见 [backend/README.md](backend/README.md)。
- `docs/`：仓库内技术参考、插件 API、发布流程和实现设计记录。不要把面向用户的总览文档继续堆到这里。
- `scripts/`：仓库级构建、Tauri、版本、notice 等辅助脚本。

## 3. 文档归属

- README 是入口页，不是完整手册。
- `frontend/README.md` 和 `backend/README.md` 是给人看的目录结构和开发入口说明，末尾 `To Agent` 才是给代理看的短约束。
- 插件 API、插件权限、运行时行为等和源码强绑定的内容可以留在仓库内；项目结构、用户教程、贡献者说明、功能总览等总体文档优先放到文档站。
- 不要添加生成感很强的文档元数据，例如无实际工具需要的 front matter、作者标签、更新时间戳和模板说明。

## 4. 前后端契约

改命令名、参数、返回结构、错误语义或事件字段时，至少同时检查：

1. `frontend/src/api/*.ts`
2. `backend/tauri-host/src/runtime/command_catalog.rs`
3. `backend/tauri-host/src/commands/`

不得只改其中一侧。Sea Lantern Lua 插件系统和 Minecraft 服务端插件文件管理不能混用 API、状态、权限模型或文案。

## 5. 前端规则

- 当前活跃入口是 `frontend/src/main.ts` 和 `frontend/src/main.next.ts`，不要假设旧 `frontend/next-src` 存在。
- `frontend/src/api/*.ts` 是前端命令契约入口。
- 用户可见文案应进入 `frontend/src/language/`。
- 新 UI 优先使用现有组件、store、CSS 变量和主题定义；界面应清楚、稳定、可重复使用。

## 6. 后端规则

- 可复用纯规则优先放进对应 `*-core` crate。
- `tauri-host` 负责命令暴露、服务编排、持久化、运行时状态、插件运行时和系统集成。
- `docker-entry` 和二进制入口应保持薄入口。
- preview 和真实执行需要一致时，优先共享同一套 core 层归一化结果。
- 不要把 Docker、启动、Java、日志、事件、插件信任等规则复制进 `tauri-host`。
- 高风险边界：启动配置优先级、JVM 参数顺序、Docker env 合成、事件 DTO、命令 payload、序列化字段、错误语义、插件权限和信任。

## 7. 验证

使用能证明改动范围的最小检查。默认基线如下。

前端：

```bash
pnpm lint
pnpm build:check
pnpm fmt:check
```

后端：

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace -- -D warnings
```

桌面集成：

```bash
pnpm tauri:dev
```

只改文档时，检查链接和路径即可。不能运行检查时，要在最终回复里说明。

## 8. 提交信息

使用 Conventional Commits：

```text
type(scope): 中文描述
```

允许类型：

- `feat`
- `fix`
- `docs`
- `style`
- `refactor`
- `perf`
- `test`
- `chore`

要求：

- 描述简短，使用祈使语气。

## 9. 本地注意事项

- 默认开发路径是 `pnpm tauri:dev`。
- 低内存开发路径是 opt-in，不要替代默认说明。
- 仓库级 Tauri 启动逻辑在 `scripts/tauri.mjs`。
- 如果任务涉及版本、Tauri 启动、插件运行时或发布流程，先读对应脚本和当前源码，不要凭记忆改。
