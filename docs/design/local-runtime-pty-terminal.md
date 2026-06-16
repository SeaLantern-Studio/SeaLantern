# Sea Lantern 本地运行时 PTY 托管交互式终端设计稿

## 背景

Sea Lantern 当前的本地服务器运行模型是“宿主管道托管”：

- 启动器把本地服务端子进程的 `stdin`、`stdout`、`stderr` 都接成管道。
- 日志通过 `spawn_server_output_reader()` 按行读取并进入统一日志管线。
- 控制台命令通过宿主向子进程 `stdin` 写入文本发送。
- CLI 的 `--cli` 目前是一个宿主侧会话循环，不是真正 attach 到服务端终端。
- 前端控制台页目前是“日志流订阅/轮询 + 命令发送”，不承载交互式终端语义。

这套模型非常适合当前能力：

- 截获日志
- 写入 Sea Lantern 自定义系统日志
- 结构化日志解析
- 自动分析错误
- 插件 `sl.console.*` 接口
- 前端分页和回放

但它也带来一个根本限制：服务端进程看不到真实终端，因此像 JLine、TerminalConsoleAppender、NeoForge/Minecraft 自带的高级终端能力会被关闭或降级。

用户现在希望同时满足两件事：

1. 服务端拥有高级终端能力。
2. Sea Lantern 仍能保留现有日志、命令、分析和宿主扩展能力。

如果直接把进程切成“外部原生直连”，Sea Lantern 会失去对实时终端流的完整控制；如果继续只有 pipe 模式，服务端又拿不到终端能力。因此需要一个新的中间模型。

## 当前事实

### 本地运行时现状

- 本地启动命令当前显式设置 `stdout(Stdio::piped())`、`stderr(Stdio::piped())`、`stdin(Stdio::piped())`。
- `local_helper` 在子进程启动后接管 `stdout/stderr`，并把输出送入 `server_log_pipeline`。
- `LocalServerRuntime::send_command()` 仍然基于向托管子进程 `stdin` 写一行文本。
- Windows 启动路径当前还会使用 `CREATE_NO_WINDOW`，说明设计目标本来就是后台托管，而不是给服务端一个独立原生控制台窗口。

### 日志与控制台现状

- 日志管线当前假设输入是“可按行切分的普通控制台文本”。
- `spawn_server_output_reader()` 以 `read_until('\n')` 为边界读取日志。
- Sea Lantern 自定义日志通过 `[Sea Lantern] ...` 前缀写入统一日志视图。
- 前端 `ConsoleView` 依赖 `getLogs()` + `onLogLine()` 渲染日志，并通过 `sendCommand()` 发命令。
- CLI `attach_server_cli()` 是一个宿主提示符会话，不是服务端终端 attach。

### 兼容边界

- 现有 `sl.console.send/get_logs/get_status` 已对插件暴露，不应在本次方向上破坏。
- 现有日志存储、分页读取、结构化事件、玩家解析、错误分析都建立在“归一化日志流”之上，不应直接改成原始 PTY 字节流语义。

## 目标

- 为本地运行时新增一种由 Sea Lantern 托管的 PTY 终端模式，让服务端进程获得高级终端能力。
- 保留 Sea Lantern 对输入输出流的中介地位，以继续支持日志存储、分析、宿主系统消息和自动化能力。
- 让前端控制台与 CLI attach 最终都能接入真正的交互式终端会话，而不只是日志回放。
- 保持现有 console/log/plugin 接口强兼容，必要时只追加新能力。
- 为 Windows、Linux、macOS 定义统一抽象，但按实施顺序优先落地 Windows。

## 非目标

- 不把“真正外部原生终端直连”作为主实现路线。
- 不在首期重做或替换现有 `sl.console.*` 接口。
- 不要求首期立即删除现有 pipe 托管模式。
- 不在首期把所有日志消费方都改成依赖原始 PTY transcript。
- 不在首期解决 Docker runtime 的交互式终端问题；本稿只覆盖本地运行时。

## 结论先行

### 推荐方案

新增 `pty_managed` 本地终端模式，并保留现有 `pipe_managed` 作为兼容回退。

Sea Lantern 通过 PTY/ConPTY 作为终端宿主：

- 对服务端来说，它连接的是一个真实终端。
- 对 Sea Lantern 来说，它仍然位于中间，可以观察、转发、记录和分析终端流。

### 不推荐的方案

不推荐把“外部原生控制台直连”作为主路线。

原因：

- Sea Lantern 将失去对实时终端流的稳定观测。
- 日志截获、结构化处理、自动分析、前端控制台同步都会退化。
- CLI 和前端都无法自然共享同一个会话。

## 终端模式设计

本地运行时明确分成三种模式：

- `pipe_managed`
  - 当前默认模式。
  - 优点：简单、稳定、适合日志处理。
  - 缺点：服务端拿不到高级终端能力。
- `pty_managed`
  - 推荐目标模式。
  - Sea Lantern 创建 PTY，并把服务端绑定到 PTY slave。
  - Sea Lantern 读取 PTY master，并向其写入输入。
  - 这是未来的默认本地终端模式。
- `direct_external`
  - 只作为对照模型保留在设计稿中。
  - 不作为 Sea Lantern 产品主路径实现。

## 总体架构

### 核心思路

把“本地进程托管方式”从单一 pipe 模型抽成终端后端抽象：

```text
Local runtime
├── process launcher
├── terminal backend
│   ├── pipe backend
│   └── pty backend
├── transcript pipeline
├── normalized log pipeline
└── command/control adapters
```

### 新抽象

本地运行时需要新增统一终端后端接口，至少覆盖：

- `spawn()`：以指定终端后端启动进程
- `write_input()`：向终端写输入
- `read_output()`：读取原始终端输出块
- `resize()`：调整终端大小
- `close_input()`：关闭输入侧
- `supports_interactive_features()`：声明能力
- `backend_kind()`：返回 `pipe` / `pty`

`PipeBackend` 和 `PtyBackend` 都实现这套接口，但上层 `local_helper` 与 `LocalServerRuntime` 不再直接假设 `Child.stdin/stdout/stderr` 就是唯一形态。

## 日志与终端双通道

### 问题

PTY 输出不是普通逐行日志流。它可能包含：

- ANSI 颜色序列
- 光标移动
- 行内重绘
- 补全与回显
- 屏幕清理

如果把这类原始输出直接塞进现有日志系统，会污染日志数据库、结构化解析和错误分析。

### 推荐拆分

新增两条并行通道：

1. `terminal transcript`
2. `normalized log stream`

#### terminal transcript

- 保存 PTY 原始输出块或最小损失的渲染事件。
- 服务于：
  - 前端交互式终端视图
  - CLI attach
  - 调试和故障复盘
- 不直接承诺兼容现有 `getLogs()` 分页语义。

#### normalized log stream

- 从 transcript 中经过 ANSI 清洗、控制序列折叠、行归并后得到。
- 继续服务于：
  - 当前日志数据库
  - `getLogs()` / `sl.console.get_logs`
  - 结构化事件
  - 玩家列表解析
  - 错误分析
  - Sea Lantern 现有日志页

### 关键原则

- transcript 是“交互终端事实来源”。
- normalized log stream 是“日志与自动化事实来源”。
- 两者不能再混为一个数据结构。

## 宿主系统消息设计

### 问题

当前 Sea Lantern 自定义日志直接写入统一控制台日志流，这在 pipe 模式下问题不大。

但在 PTY 模式下，如果宿主消息直接插入前台终端流：

- 会打断用户正在编辑的命令行。
- 会干扰服务端自己的行编辑与补全显示。
- 会让 transcript 与真实服务端输出混杂。

### 推荐方案

把 Sea Lantern 自定义消息改为独立“宿主系统消息”侧通道：

- 默认写入 normalized log stream 和宿主事件流。
- 前端日志视图继续可见。
- 前端交互式终端默认不主动注入该消息。
- 只有在明确安全的场景下，才允许将宿主消息镜像到终端视图。

这样可以保留用户熟悉的 `[Sea Lantern] ...` 信息，但不破坏 PTY 交互体验。

## 前端设计

### 当前状态

当前 `ConsoleView` 是日志视图，不是真正的终端仿真器。

### 目标形态

前端控制台拆成两层能力：

- `LogConsoleView`
  - 保持现有日志回放、搜索、滚动、系统日志可见性。
- `InteractiveTerminalView`
  - 在 PTY 模式下连接 transcript/session。
  - 承载交互式输入、颜色、终端尺寸变化、复制、粘贴等行为。

### 产品行为

- 当本地运行时是 `pipe_managed`：保留当前日志控制台体验。
- 当本地运行时是 `pty_managed`：
  - 默认展示交互式终端视图。
  - 同时保留日志视图切换入口。
- 两种视图共享同一台服务器状态、启停与命令能力。

### 首期前端要求

- 必须支持连接交互式终端会话。
- 必须支持终端尺寸同步。
- 必须支持滚动回看近期 transcript。
- 必须保留现有日志页的可读性与稳定性。

## CLI 设计

### 当前状态

当前 CLI attach 只是宿主读一行、发一行的循环，不是真正 attach 到服务端终端。

### 目标形态

CLI `--cli` / attach 需要升级为连接交互式终端会话：

- 在 `pipe_managed` 模式下，保留当前简单会话行为。
- 在 `pty_managed` 模式下，CLI attach 直接接入 PTY transcript/input 通道。

### 首期要求

- CLI attach 必须能看到与前端一致的终端会话内容。
- CLI attach 退出不能导致服务端退出。
- 前端与 CLI 可以顺序 attach 到同一会话，不要求首期支持完全并发多人编辑协作。

## 插件与公共接口兼容

### 强兼容要求

以下接口在 PTY 模式下必须继续可用：

- `sl.console.send`
- `sl.console.get_logs`
- `sl.console.get_status`
- 前端 `getLogs()` / `sendCommand()` / `getStatus()`
- 现有日志推送事件

### 兼容策略

- 旧接口继续面向 normalized log stream 和宿主统一命令抽象。
- 不要求插件直接消费原始 PTY transcript。
- 后续如需开放更强能力，可新增 transcript/interactive 相关接口，但不能替换旧接口。

## 平台抽象

### 统一要求

设计首版就按跨平台抽象写定：

- Windows：ConPTY
- Linux：POSIX PTY
- macOS：POSIX PTY

### 实施顺序

落地顺序按下面执行：

1. Windows ConPTY
2. Linux PTY
3. macOS PTY

原因：

- 当前用户环境以 Windows 为主。
- 当前本地运行链路已包含较多 Windows 分支。
- ConPTY 是这次产品收益最高的首个目标。

### 首期平台约束

- 抽象层接口一次设计到位。
- 实现与验证允许按平台分阶段推进。
- 未实现的平台在选择 PTY 模式时必须显式回退到 `pipe_managed`，不能静默假装成功。

## 数据模型与配置

本地运行时配置新增终端模式字段，例如：

- `terminal_mode = pipe_managed | pty_managed`

并为 PTY 会话保留必要元数据：

- `backend_kind`
- `interactive_supported`
- 终端尺寸
- transcript capability
- attach capability

这些字段可以是运行时状态，不要求全部持久化进服务器配置；但“默认终端模式”必须可配置。

## 对现有代码结构的影响

### 需要重构的模块

- 本地运行时启动链路
- `local_helper` 中对子进程 stdio 的直接假设
- 日志输入采集层
- CLI attach 实现
- 前端控制台视图与会话协议

### 尽量保留的模块

- 现有服务器状态模型
- 现有 `send_command` 命令入口
- 现有日志数据库与分页读取语义
- 现有插件 `sl.console.*` 入口
- 当前 `pipe_managed` 路径

## 分阶段实施建议

### Phase 0：设计收敛

- 明确终端后端抽象
- 明确 transcript 与 normalized log 双通道
- 明确前端与 CLI attach 的目标行为
- 明确兼容与默认切换策略

### Phase 1：本地运行时终端后端抽象

- 把现有 pipe 模式接入统一终端后端接口
- 保证现有行为不变
- 为 PTY 后端预留 hook 点

### Phase 2：Windows ConPTY MVP

- 增加 `pty_managed` 后端
- 完成本地服务端通过 PTY 启动
- 建立 transcript 读取链路
- 保持 normalized log stream 继续可用

### Phase 3：前端交互式终端接入

- 新增交互式终端视图
- 接入 transcript 显示与输入写回
- 保留现有日志视图

### Phase 4：CLI attach 接入 PTY 会话

- 让 `--cli` 连接交互式会话
- 保留 pipe 模式旧行为

### Phase 5：兼容层与自动化增强

- 稳定 Sea Lantern 宿主系统消息侧通道
- 验证 `sl.console.*` 与现有日志接口行为
- 接入自动分析对 normalized log stream 的兼容校准

### Phase 6：跨平台扩展与默认切换评估

- 补 Linux/macOS PTY 后端
- 达到默认切换门槛后，把 `pty_managed` 升为默认

## 默认切换策略

### 当前阶段

- 默认继续使用 `pipe_managed`
- `pty_managed` 作为显式可选项与试验路径

### 切默认门槛

只有满足下面全部条件，才允许把 `pty_managed` 切为默认：

- Windows ConPTY 路径在主流本地服务端类型上稳定
- 前端交互式终端可用
- CLI attach 可用
- normalized log stream 在 PTY 模式下保持可读和可分析
- `sl.console.*` 与旧日志接口无破坏性回归
- PTY 失败时回退路径可靠

### 默认切换后的回退

- 当平台不支持 PTY
- 当 PTY 初始化失败
- 当终端代理能力检测失败
- 当用户显式关闭交互终端模式

以上场景全部自动回退到 `pipe_managed`。

## 验收标准

### 本地服务端行为

- NeoForge、Fabric、Paper 至少能在 PTY 模式下正常启动。
- 服务端可启用其自带高级终端能力。
- Sea Lantern 仍可执行启动、停服、强停、发命令、状态读取。

### 前端行为

- 控制台页可进入交互式终端。
- 用户输入、颜色和终端大小变化可正常工作。
- 同时仍可查看归一化日志。

### CLI 行为

- `--cli` 可 attach 到 PTY 会话。
- 退出 CLI attach 不影响服务端进程。

### 日志与插件行为

- `getLogs()`、`sl.console.get_logs` 继续返回可读日志。
- 错误分析、玩家解析、结构化事件继续可用。
- Sea Lantern 自定义系统消息不会打断终端当前输入行。

### 回退行为

- PTY 初始化失败必须明确记录原因。
- 系统自动切回 `pipe_managed` 后，基本控制台能力仍然可用。

## 风险与难点

### 1. transcript 与日志流边界

如果仍把交互式终端输出当普通逐行日志处理，后续会持续污染日志、分析和插件接口。

### 2. 宿主系统消息插入时机

如果宿主消息继续直接写进 PTY 前台流，会破坏用户编辑体验。

### 3. 多消费端 attach

前端与 CLI 如果都能 attach，同步策略必须清晰。首期可以允许“顺序 attach、单活交互”，不要一开始就承诺多人同时编辑。

### 4. 平台差异

ConPTY 与 POSIX PTY 在窗口大小、编码、会话生命周期和控制序列上都存在差异，抽象层需要提前为这些差异预留位。

### 5. 默认切换回归风险

如果没有明确门槛就切默认，很容易把当前稳定的日志和插件生态拖入回归。

## 推荐决策

为避免实现阶段继续摇摆，本稿把以下决策直接固定：

1. 本地主路线采用 `pty_managed`，不是外部直连。
2. 设计首版按跨平台抽象写，但实施顺序 Windows 优先。
3. transcript 与 normalized log stream 必须拆开。
4. 现有 `sl.console.*`、前端日志接口、日志存储语义强兼容。
5. Sea Lantern 宿主消息改走侧通道，而不是默认写入前台 PTY。
6. 未来可以切默认，但只能在验收门槛全部满足后进行。

这组决策能在保住 Sea Lantern 现有扩展能力的同时，给服务端和用户带来真正的高级终端体验。
