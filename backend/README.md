# 后端说明

`backend/` 是 Sea Lantern 的 Rust workspace，负责 Tauri 命令、服务器启动、Docker 运行、日志、更新、插件运行时和后端共享逻辑。

根目录的 `Cargo.toml` 管理整个 workspace。

## 目录结构

```text
backend/                       backend/
├── docker-entry/              └── Docker / headless 入口
├── runtime-core/                  运行时工具
├── event-core/                    事件模型
├── i18n-core/                     后端语言
├── server-config-core/            启动配置
├── docker-core/                   Docker 规则
├── server-local-setup-core/       本地服务器接管
├── server-installer-core/         服务端安装
├── server-startup-scan-core/      启动候选扫描
├── java-installer-core/           Java 安装
├── server-log-core/               服务器日志
├── server-plugin-core/            服务端插件文件
├── plugin-trust-core/             插件信任
├── lua-runtime-core/              Lua 边界
├── server-download-links-core/    下载链接解析
├── starter-links-core/            Starter 链接
├── update-core/                   更新流程
└── tauri-host/                    桌面 / headless 宿主
    ├── tauri.conf.json            └── Tauri 配置
    └── src/                           主代码
        ├── lib.rs                     └── 宿主库入口
        ├── main.rs                        桌面入口
        ├── runtime/                       启动和命令注册
        ├── models/                        数据模型
        ├── hardcode_data/                 内置数据
        ├── utils/                         宿主工具
        ├── commands/                      Tauri 命令
        │   ├── app/                       └── 应用命令
        │   ├── downloads/                     下载命令
        │   ├── online/                        联机命令
        │   ├── plugins/                       插件命令
        │   ├── server/                        服务器命令
        │   └── update/                        更新命令
        ├── services/                      宿主服务
        │   ├── server/                    └── 服务器服务
        │   ├── online/                        在线服务
        │   └── download/                      下载服务
        └── plugins/                       Lua 插件系统
            ├── api/                       └── 插件宿主 API
            └── runtime/                       插件运行时 API
```

## 表格对照

| 改动内容 | 优先查看 |
| --- | --- |
| 新增或修改 Tauri 命令 | `tauri-host/src/runtime/command_catalog.rs`、`tauri-host/src/commands/`、`../frontend/src/api/` |
| 应用、设置、系统、i18n 命令 | `tauri-host/src/commands/app/` |
| 下载、Java、Mod 下载命令 | `tauri-host/src/commands/downloads/` |
| 联机、Join ID、隧道命令 | `tauri-host/src/commands/online/` |
| Sea Lantern Lua 插件命令 | `tauri-host/src/commands/plugins/` |
| 服务器管理、配置、玩家、扩展、服务端插件命令 | `tauri-host/src/commands/server/` |
| 更新命令 | `tauri-host/src/commands/update/`、`update-core/` |
| 后端服务编排 | `tauri-host/src/services/` |
| 桌面 / headless 宿主行为 | `tauri-host/` |
| Headless HTTP、运行模式、端口、panic report | `runtime-core/`、`tauri-host/src/runtime/` |
| CLI server / compose / docker doctor | `tauri-host/src/utils/cli/` |
| Docker 服务器启动 | `docker-core/`、`tauri-host` 中调用 Docker 的服务 |
| Docker / headless 独立入口 | `docker-entry/` |
| 本地 Java 服务器启动 | `server-local-setup-core/`、`server-config-core/`、`tauri-host/src/services/server/manager/runtime_start/` |
| 服务器配置和 JVM 参数 | `server-config-core/` |
| 创建、导入、modpack provisioning | `tauri-host/src/services/server/manager/provisioning/`、`server-installer-core/`、`server-local-setup-core/` |
| 服务端安装和核心识别 | `server-installer-core/`、`server-download-links-core/`、`starter-links-core/`、`tauri-host/src/services/download/` |
| 扫描已有服务器目录 | `server-startup-scan-core/` |
| Java 下载和安装 | `java-installer-core/` |
| 日志持久化和读取 | `server-log-core/`、`tauri-host` 输出读取逻辑 |
| 应用事件和服务器事件 | `event-core/`、`tauri-host` 事件转发 |
| 后端多语言 | `i18n-core/` |
| Sea Lantern Lua 插件运行时 | `tauri-host/src/plugins/`、`lua-runtime-core/`、`plugin-trust-core/` |
| 插件 UI snapshot、sidebar、context menu、component mirror | `tauri-host/src/plugins/api/`、`tauri-host/src/plugins/runtime/ui/` |
| Sea Lantern 插件权限和信任 | `plugin-trust-core/` |
| Minecraft 服务端插件文件 | `server-plugin-core/` |
| 在线服务、隧道、OneBot | `tauri-host/src/services/online/`、`tauri-host/src/commands/online/` |
| 更新检查、下载和安装 | `update-core/`、`tauri-host` 更新命令 |

## 开发命令

格式检查：

```bash
cargo fmt --all -- --check
```

编译检查：

```bash
cargo check --workspace
```

Clippy：

```bash
cargo clippy --workspace -- -D warnings
```

不管改了什么，最后仍然建议跑 workspace 级别检查。

## 注意事项

### 分层

- 纯规则优先放进对应的 `*-core` crate。
- `tauri-host` 负责命令暴露、服务编排、持久化、运行时状态和系统集成。
- `docker-entry` 和二进制入口应保持薄入口。
- 不要把 Docker、启动、Java、日志、事件、插件信任等规则复制到 `tauri-host`。
- 如果 preview 和真实执行都需要同一套规则，优先把归一化逻辑放到 core crate。

### 命令契约

改前后端命令时，至少同时检查：

1. `frontend/src/api/*.ts`
2. `backend/tauri-host/src/runtime/command_catalog.rs`
3. `backend/tauri-host/src/commands/`

命令名、参数、返回结构、错误语义和事件字段都是前端可见契约。

`command_catalog.rs` 是桌面 IPC 权威注册表。新增命令时，不要只写 command 函数，必须确认前端 wrapper、注册表和实现三处一致。

### 服务器启动

- 启动配置优先级是实例配置、运行时配置、应用默认值。
- JVM 参数顺序会影响行为，不要随意重排内存、编码、预设参数、用户参数和 `ActiveProcessorCount`。
- preview 和真实启动应尽量使用同一套归一化后的数据。
- 本地 jar、starter、bat、sh、ps1、custom start 是不同路径，不要为了简化代码随意合并。
- `server-config-core` 负责启动配置、JVM 参数、CPU policy 和 `server.properties` 规则。
- `server-local-setup-core` 负责本地目录检查、启动模式推断、Java 路径和本地启动预览辅助。

### Docker

- Docker preview 应描述真实 `docker run` 形状。
- preview 中的敏感信息需要脱敏。
- JVM env 合成不能覆盖运行时明确提供的 `JVM_OPTS` 或 `JVM_XX_OPTS`。
- Docker CPU policy 应和 `server-config-core` 保持一致，除非明确要拆分行为。
- `docker-core` 不负责实际执行 Docker 进程，也不负责宿主状态流转。

### 日志和事件

- 日志要保持 append 顺序、历史读取顺序、flush 生命周期和 ready-event 检测语义。
- 事件 DTO 和序列化字段是前端可见契约。
- 改事件字段时，要检查前端订阅和状态更新逻辑。
- `runtime-core` 负责共享 runtime 工具，不应放 Minecraft 业务规则。

### 插件

- Sea Lantern Lua 插件系统和 Minecraft 服务端插件文件管理是两套逻辑。
- Lua 插件权限、信任和权限归一化应集中在 `plugin-trust-core`。
- Lua runtime trait 签名是插件运行时边界，改动时要同步 host 和 runtime。
- Minecraft 服务端插件 jar / 配置扫描应留在 `server-plugin-core`。
- 插件 UI、元素、存储、HTTP、文件系统、server、console 等运行时 API 位于 `tauri-host/src/plugins/runtime/`。
- 插件管理、市场、安装、启停和 snapshot 相关命令位于 `tauri-host/src/commands/plugins/`。

### 更新

- 更新逻辑应保持检查、下载、待安装状态和安装启动计划分离。
- 不要为了减少调用层数，把不同阶段揉成一个不可恢复的流程。

## To Agent

- Put reusable rules in the owning `*-core` crate.
- Keep `tauri-host` focused on commands, orchestration, persistence, runtime state, and host integration.
- For command changes, inspect frontend API wrappers, command catalog, and command implementations together.
- Keep preview and execution behavior aligned.
- Do not mix Sea Lantern Lua plugin logic with Minecraft server plugin file scanning.
- Treat event DTOs, command payloads, serialized fields, and error semantics as frontend-facing contracts.
- Run `cargo fmt --all -- --check`, `cargo check --workspace`, and `cargo clippy --workspace -- -D warnings` when practical.
