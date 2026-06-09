# Sea Lantern 容器化运行时设计草案（Java 版 / itzg）

## 背景

Sea Lantern 当前的服务器运行模型是“本地 Java 进程托管”：

- 服务器以宿主机子进程形式启动。
- 控制台日志直接读取子进程 `stdout/stderr`。
- 控制台命令通过子进程 `stdin` 发送。
- 启动参数依赖本地 `java_path`、`jar_path`、`startup_mode`、`custom_command`。

当前仓库已经支持把 Sea Lantern 自身打包为 Docker 容器并以 HTTP 服务运行，但这不等于“以容器方式运行 Minecraft 服务器”。

本草案目标是为 Java 版 Minecraft 服务器新增一个面向容器的运行时后端，首期优先兼容 `itzg/minecraft-server`。

## 为什么优先兼容 itzg

`itzg/minecraft-server` 是当前 Java 版 Minecraft 容器化最成熟、生态最广的方案之一，具备以下特点：

- 支持 `vanilla`、`paper`、`fabric`、`forge`、`neoforge` 等多种 Java 服务端类型。
- 通过环境变量统一配置 `EULA`、`VERSION`、`TYPE`、JVM 与大量 `server.properties`。
- 约定稳定：数据目录为 `/data`，默认游戏端口为 `25565`。
- 社区文档完整，后续维护成本低。
- 对 Sea Lantern 来说，它更像一种“标准化运行时”，而不是单一镜像。

因此，与其自己维护多套 Java 容器启动脚本，不如先把 Sea Lantern 适配为对 `itzg` 的管理层。

## 设计目标

- 在不破坏现有本地运行模式的前提下，新增容器运行时。
- 首期只覆盖 Java 版服务端。
- 首期只保证 `itzg/minecraft-server` 路线可用。
- 保持 Sea Lantern 现有“服务器列表 + 配置 + 启停 + 日志 + 命令发送”的产品形态。
- 为后续接入其他镜像或直接接入 Compose/Kubernetes 保留扩展位。

## 非目标

- 首期不支持 Bedrock。
- 首期不支持把现有 `bat/sh/ps1/custom` 本地启动脚本直接迁移为容器模式。
- 首期不做 Kubernetes 编排。
- 首期不做完整 TUI。
- 首期不追求兼容所有 Docker 发行版的高级特性，只先支持标准 Docker Engine API / CLI 路径。

## 结论先行

### 是否能兼容 itzg

能，而且值得优先兼容。

但兼容方式不应是“继续沿用当前本地 Java 进程模型”，而应是“新增 `docker_itzg` 运行时后端”。

### 是否有必要专门做 CLI / TUI

- `CLI`：有必要，且优先级高。
- `TUI`：不是首期必要项，优先级低于运行时抽象和 CLI。

原因：容器化环境首先需要的是可脚本化、可自动化、可远程执行的控制面，而不是额外再维护一套终端 UI。

## 总体方案

### 核心思路

把“服务器如何运行”从当前固定的本地进程模型中抽出来，抽象为 `runtime_kind`：

- `local`：现有模式，保留。
- `docker_itzg`：新增模式。

业务层不再默认“服务器就是一个本地 Java 子进程”，而是统一走运行时适配层。

### 分层建议

新增运行时抽象层，例如：

```text
services/server/runtime/
├── mod.rs
├── traits.rs              # 统一运行时接口
├── local/                 # 现有本地 Java 进程运行时
└── docker_itzg/           # itzg 容器运行时
```

统一接口建议覆盖：

- `prepare`
- `start`
- `stop`
- `force_stop`
- `status`
- `send_command`
- `stream_logs`
- `inspect`

现有 `ServerManager` 继续作为上层协调器，但不再直接绑定 `std::process::Child` 作为唯一运行形态。

## 数据模型设计

### 问题

当前 `ServerInstance` 偏向本地启动，字段例如：

- `java_path`
- `jar_path`
- `startup_mode`
- `custom_command`
- `jvm_args`

这些字段在 `itzg` 容器模式下并不天然成立，继续硬塞会让模型很脏。

### 建议方案

保留现有 `ServerInstance` 主体，但补充运行时配置分层。

示意：

```rust
pub struct ServerInstance {
    pub id: String,
    pub name: String,
    pub core_type: String,
    pub core_version: String,
    pub mc_version: String,
    pub path: String,
    pub port: u16,
    pub created_at: u64,
    pub last_started_at: Option<u64>,
    pub runtime_kind: String,
    pub runtime: ServerRuntimeConfig,
}

pub enum ServerRuntimeConfig {
    Local(LocalRuntimeConfig),
    DockerItzg(DockerItzgRuntimeConfig),
}
```

其中：

```rust
pub struct LocalRuntimeConfig {
    pub jar_path: String,
    pub startup_mode: String,
    pub custom_command: Option<String>,
    pub java_path: String,
    pub jvm_args: Vec<String>,
}

pub struct DockerItzgRuntimeConfig {
    pub image: String,
    pub image_tag: String,
    pub container_name: String,
    pub type_value: String,
    pub version: String,
    pub data_dir_mount: String,
    pub published_game_port: u16,
    pub env: std::collections::BTreeMap<String, String>,
    pub extra_ports: Vec<PublishedPort>,
    pub volume_mounts: Vec<VolumeMount>,
    pub command_mode: DockerCommandMode,
    pub rcon: Option<RconConfig>,
}
```

### 字段映射建议

Sea Lantern 现有字段到 `itzg` 环境变量的映射建议：

- `mc_version` -> `VERSION`
- `core_type` -> `TYPE`
- `auto_accept_eula` 或建服时开关 -> `EULA=TRUE`
- 内存配置 -> `MEMORY` 或 `JVM_XX_OPTS` / `JVM_OPTS`
- `port` -> 宿主机映射端口，不建议直接改容器内 `SERVER_PORT`
- `online_mode` -> `ONLINE_MODE`
- 其余 `server.properties` -> 优先映射到已知环境变量，其次写入 `CUSTOM_SERVER_PROPERTIES`

### `TYPE` 映射建议

首期先做保守映射：

- `paper` -> `PAPER`
- `fabric` -> `FABRIC`
- `forge` -> `FORGE`
- `neoforge` -> `NEOFORGE`
- `spigot` -> `SPIGOT`
- `vanilla` 或未知基础类型 -> `VANILLA`

以下类型首期不建议直接承诺完全兼容：

- `arclight-*`
- `spongeforge`
- 其它混合核心

这些可以在后续阶段再判断是否通过自定义镜像或额外启动变量支持。

## 容器控制模型

### 日志

本地模式：

- 读取子进程 `stdout/stderr`

Docker 模式：

- 读取 `docker logs --follow` 或 Docker Engine log stream

因此日志管线需要新增“日志来源抽象”，而不是默认来源于 `Child`。

### 状态

本地模式：

- 通过 `Child::try_wait()` 判断

Docker 模式：

- 通过容器状态判断，例如 `created`、`running`、`exited`、`restarting`

Sea Lantern 前端仍可继续显示：

- `stopped`
- `starting`
- `running`
- `stopping`
- `error`

但中间需要做运行时状态映射。

### 命令发送

这是容器模式的关键差异点。

当前本地模式使用子进程 `stdin` 发命令，Docker 模式不能直接复用。

建议优先级：

1. 优先使用 `RCON`
2. 次选支持附加到容器 stdin
3. 最后才考虑镜像内辅助脚本或 `docker exec`

推荐首期将 `RCON` 作为 `docker_itzg` 的标准控制通道，因为：

- 接口稳定
- 与容器实现解耦
- 不依赖当前客户端是否持有 `attach` 会话
- 后续更容易做远程/分布式控制

代价是需要：

- 自动配置 `ENABLE_RCON=true`
- 安全生成并保存 `RCON_PASSWORD`
- 管理 `RCON_PORT`

## 路径与挂载设计

### 核心原则

对 `itzg` 而言，Minecraft 服务端的数据根目录就是容器内 `/data`。

因此 Sea Lantern 中“服务器路径”的语义需要区分：

- 逻辑服务器目录：Sea Lantern 认为这是一台服务器的数据目录
- 宿主机挂载目录：Docker bind mount 的源路径
- 容器内目录：通常固定为 `/data`

### 推荐做法

对于 `docker_itzg`：

- `ServerInstance.path` 继续表示宿主机上的服务器数据目录
- 容器内目录固定使用 `/data`
- 不再允许把容器启动目标指向任意 `jar_path`

### 容器中的 Sea Lantern 管宿主机 Docker

这是一个必须正视的问题。

若 Sea Lantern 自己运行在容器内，那么它看到的路径可能是：

- Sea Lantern 容器内路径：`/app/data/servers/foo`
- 宿主机真实路径：`/srv/sealantern/servers/foo`

Docker Engine 创建新容器时，bind mount 需要宿主机真实路径，而不是 Sea Lantern 容器内路径。

因此如果支持“Sea Lantern 容器管理 Docker 容器”，必须增加路径映射配置，例如：

- `SERVERS_HOST_ROOT`
- `SERVERS_CONTAINER_ROOT`

并在运行时完成路径换算。

如果没有这层映射，容器中的 Sea Lantern 无法稳定创建兄弟 Minecraft 容器。

## 创建与导入流程

### 新建服务器

新增运行时选项：

- 本地运行
- Docker（itzg）

选择 `Docker（itzg）` 后，创建流程需要：

- 选择服务端类型
- 选择 Minecraft 版本
- 选择宿主机数据目录
- 设置宿主机映射端口
- 配置内存
- 可选启用 RCON
- 可选补充环境变量 / 自定义属性

### 导入已有服务器

容器模式的“导入”应拆成两类：

1. 导入已有宿主机数据目录，并由 Sea Lantern 新建 itzg 容器
2. 接管一个已有的 itzg 容器

首期建议只做第一种，第二种放后续。

原因：接管已有容器时，需要反向解析镜像、环境变量、挂载、端口和命名规则，复杂度明显更高。

## Docker 调用方式

### 方案 A：调用 Docker CLI

优点：

- 实现快
- 依赖少
- 便于先做 MVP

缺点：

- 对环境要求更高，需要系统安装 Docker CLI
- 输出解析脆弱
- Windows / Linux / macOS 差异更明显

### 方案 B：接 Docker Engine API

优点：

- 结构化返回
- 更稳
- 更适合长期维护

缺点：

- 初始工作量更大
- Windows/WSL/Unix Socket 适配更复杂

### 建议

MVP 允许先走 Docker CLI，但代码结构必须预留一个 `docker client adapter`，避免业务逻辑直接散落一堆 `Command::new("docker")`。

## CLI / TUI 设计建议

### CLI：建议首期纳入范围

现有 CLI 只覆盖本地模式下的少量管理动作，不足以支撑容器场景。

建议新增分层明确的子命令：

```text
sealantern server list
sealantern server create
sealantern server inspect <id>
sealantern server start <id>
sealantern server stop <id>
sealantern server restart <id>
sealantern server logs <id> [--follow]
sealantern server send <id> <command>
sealantern server runtime <id>
sealantern docker doctor
sealantern compose generate <id>
```

首期 CLI 目标：

- 可脚本化
- 可在 SSH/终端环境直接管理
- 可服务未来 TUI

### TUI：不建议首期投入

TUI 只有在以下条件同时成立时才值得做：

- CLI 已稳定
- 确认存在大量纯终端用户
- 明确需要在无浏览器环境提供持续交互体验

否则首期做 TUI 的收益低于成本。

建议顺序：

1. 先做好运行时抽象
2. 再做好 CLI
3. 最后判断是否需要 TUI

## 前端改动建议

### 创建页

新增运行时选择：

- 本地 Java
- Docker（itzg）

根据运行时切换字段显示：

- 本地模式显示 `java_path`、`jar_path`、`startup_mode`
- Docker 模式隐藏本地启动字段，显示镜像与挂载配置

### 服务器详情页

增加运行时标识：

- `Local`
- `Docker / itzg`

### 控制台页

前端不应再假设“命令总是发往本地 stdin”，而应统一调用后端 `send_command`，由运行时适配层决定具体走本地 stdin 还是 RCON。

## 后端改动建议

### 需要重构的点

- `ServerManager` 当前以 `Child` 作为运行时状态核心，需要抽象。
- 日志管线当前默认绑定本地进程输出，需要抽象日志源。
- 停服逻辑当前默认向本地进程发送 `stop`，需要抽象。
- `ServerInstance` 当前以本地启动字段为中心，需要引入运行时配置。

### 尽量保留的点

- 服务器列表与基本元数据管理
- `server.properties` 解析与编辑能力
- 插件文件管理能力
- 前端整体页面结构

## 风险与难点

### 1. 数据模型演进

如果直接在旧模型上不断打补丁，后期会很难维护。

建议尽早引入 `runtime_kind + runtime config` 分层。

### 2. 容器中管理容器的路径问题

这是 Docker 场景里最容易被低估的问题。

如果不先定义宿主路径映射策略，很多“看起来能跑”的实现都会在 bind mount 时失败。

### 3. 命令控制通道

如果首期强依赖 attach stdin，日志与命令会话管理会变复杂。

建议优先把 RCON 设计好。

### 4. 镜像兼容边界

`itzg` 支持的类型很多，但 Sea Lantern 不应首期承诺“所有核心类型都稳定支持”。

建议首期白名单支持：

- Vanilla
- Paper
- Fabric
- Forge
- NeoForge

### 5. 本地模式回归风险

运行时抽象改造如果做得粗糙，最容易影响现有本地启动链路。

因此实现时应坚持：

- 先把接口抽出来
- 再把现有本地模式包进接口
- 最后再加 Docker runtime

## 分阶段实施建议

### Phase 0：设计收敛

- 确认数据模型
- 确认 Docker 控制方式（CLI 还是 Engine API）
- 确认命令通道是否首期强制 RCON
- 确认 Sea Lantern 容器部署时的宿主路径映射方案

### Phase 1：运行时抽象

- 抽出统一运行时接口
- 把现有本地模式接入接口
- 保证本地模式行为不变

### Phase 2：`docker_itzg` MVP

- 新建 Docker 服务器
- 启停容器
- 查看状态
- 查看日志
- 通过 RCON 发命令

### Phase 3：前端接入

- 创建页运行时切换
- 服务器卡片显示运行时类型
- 控制台页兼容容器命令通道

### Phase 4：CLI 增强

- 补全容器相关子命令
- 增加 `docker doctor`
- 增加 `compose generate`

### Phase 5：增强能力

- 接管已有容器
- 支持更多 `itzg` 类型映射
- 评估 TUI 是否值得做

## MVP 范围建议

如果只做一个最小可用版本，建议范围控制为：

- 新增 `docker_itzg` runtime
- 支持新建一台 Paper/Fabric/Forge/NeoForge/Vanilla 容器服务器
- 支持端口映射
- 支持数据目录挂载
- 支持日志查看
- 支持启停
- 支持通过 RCON 发命令
- 支持 CLI 基础命令

明确不进 MVP：

- 接管已有容器
- TUI
- Kubernetes
- 混合核心全量兼容

## 待决策问题

后续进入 Plan 模式前，建议优先明确这些问题：

1. `docker_itzg` 是否作为唯一首期容器运行时。
2. Docker 接入首期是 Docker CLI 还是 Engine API。
3. 容器命令发送首期是否强制使用 RCON。
4. 是否首期只支持宿主机运行的 Sea Lantern 管理宿主机 Docker。
5. 是否要同步增强 CLI。
6. 是否把 `compose.yaml` 生成能力纳入 MVP。

## 推荐决策

如果目标是尽快落地且控制风险，推荐如下：

1. 首期只做 `docker_itzg`
2. 首期用 Docker CLI，但封装为单独适配层
3. 首期命令发送优先 RCON
4. 首期只支持“宿主机运行的 Sea Lantern 管理宿主机 Docker”
5. CLI 纳入 MVP
6. TUI 不纳入 MVP

这个组合能用最小复杂度换来真正可交付的容器化 Java 版支持。
