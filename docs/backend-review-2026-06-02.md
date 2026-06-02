# 后端审查记录（2026-06-02）

这轮只做后端审查和并行拆分，不直接改业务实现。

## 审查范围

- `src-tauri/src`
- `docker-entry/src`
- `vendor/sculk/src` 仅作为依赖边界观察，不作为本轮主修改目标

## 主要结论

### 1. 严重问题：Headless HTTP 默认外网暴露，且没有鉴权边界

- 证据：`src-tauri/src/runtime/mode.rs:29` 默认绑定 `0.0.0.0:3000`
- 证据：`src-tauri/src/adapters/http/server.rs:114` 到 `src-tauri/src/adapters/http/server.rs:117` 允许任意来源、任意方法、任意请求头
- 证据：`src-tauri/src/adapters/http/command_registry/registry.rs:14` 到 `src-tauri/src/adapters/http/command_registry/registry.rs:22` 直接注册了服务器、插件、设置、更新、隧道等高权限操作
- 证据：`src-tauri/src/adapters/http/command_registry/server.rs:13` 到 `src-tauri/src/adapters/http/command_registry/server.rs:41` 暴露了建服、导入、删服、发命令、复制目录等能力
- 影响：只要容器端口被映射或网络可达，外部请求就可以直接调用管理端接口，风险已经不是信息泄露，而是完整的远程管理面暴露
- 建议：
  1. 默认仅绑定 `127.0.0.1`，把外网监听改成显式配置。
  2. HTTP 层增加强制鉴权，至少是启动期生成的 bearer token。
  3. 把“高危命令”单独分层，未鉴权时禁止 `create_server`、`delete_server`、`install_plugin`、`send_command` 一类操作。
  4. 为 `run_http_server` 增加安全测试，覆盖未鉴权访问和跨域策略。

### 2. 严重问题：上传接口存在路径穿越和内存放大风险

- 证据：`src-tauri/src/adapters/http/server.rs:209` 到 `src-tauri/src/adapters/http/server.rs:212` 直接把用户提供的 `file_name` 拼进保存路径
- 证据：`src-tauri/src/adapters/http/server.rs:193` 到 `src-tauri/src/adapters/http/server.rs:200` 使用 `field.bytes().await` 一次性把整个文件读入内存
- 证据：`src-tauri/src/adapters/http/server.rs:125` 把 body limit 设到 `500 MiB`
- 影响：
  1. 若上传文件名包含 `../` 或平台路径分隔符，存在写出上传目录的风险。
  2. 多文件或大文件会把内容整块堆进内存，容易在容器里放大峰值内存占用，形成 DoS 面。
- 建议：
  1. 文件名只保留 `file_name()` 的 basename，额外过滤分隔符、控制字符和保留名。
  2. 目标路径改为 `PathBuf::join` 后再做 canonicalize 与目录边界校验。
  3. 改成流式写盘，不再用 `bytes()` 整块缓存。
  4. 把 body limit 下调，并按单文件大小、文件数分别限流。

### 3. 高风险问题：插件进程前台执行的“超时”没有真正生效

- 证据：`src-tauri/src/plugins/runtime/process/commands/exec.rs:131` 到 `src-tauri/src/plugins/runtime/process/commands/exec.rs:141` 在 `wait_with_output()` 返回之后才检查耗时
- 影响：当前实现并不会在达到 `MAX_FOREGROUND_EXEC_DURATION` 时中断子进程，它只会在进程已经跑完后才报超时。结果是：
  1. 长命令仍然会阻塞调用线程。
  2. 插件侧可以借此前台卡住运行时。
  3. “有超时保护”的代码表象会误导后续维护者。
- 建议：
  1. 改成真实超时控制，使用等待 + kill 机制。
  2. 前台执行和后台执行统一走一层受控进程管理，减少两套行为差异。
  3. 给 `execute_program` 加上超时回归测试，覆盖超时后子进程被回收的场景。

### 4. 中风险问题：插件权限模型在“运行时 / 文档 / 加载器”之间不一致

- 证据：`src-tauri/src/plugins/runtime/filesystem/common.rs:54` 到 `src-tauri/src/plugins/runtime/filesystem/common.rs:79` 已支持 `fs.data.read`、`fs.server.write` 这类 action 级权限
- 证据：`src-tauri/src/plugins/runtime/core/setup.rs:74` 到 `src-tauri/src/plugins/runtime/core/setup.rs:78` 也会按细粒度权限决定是否挂载 `sl.fs`
- 证据：`src-tauri/src/plugins/loader.rs:110` 到 `src-tauri/src/plugins/loader.rs:131` 的白名单却没有纳入这些 action 级 `fs.*` 权限
- 证据：`docs/plugin_api.md:34`、`docs/lua-api/filesystem.md:121` 已经把 action 级权限写进文档
- 影响：实现、测试、文档都认为细粒度权限存在，但清单校验层不承认，最终会出现“功能存在但插件不能合法声明”的落差。
- 建议：
  1. 把权限清单收敛为单一来源，不要让加载器手写一份、运行时再写一份。
  2. 给权限定义增加枚举或常量表，文档从定义生成或至少共用同一份数据。
  3. 增加 manifest 校验测试，覆盖 action 级权限的合法性。

## 复杂度与可维护性观察

### 1. 服务器运行时相关模块已经偏大，后续改动风险会继续上升

- 证据：`src-tauri/src/services/server/runtime/docker_itzg.rs` 约 1365 行
- 证据：`src-tauri/src/services/server/runtime/local_helper.rs` 约 741 行
- 证据：`src-tauri/src/services/server/manager.rs` 约 637 行
- 证据：`src-tauri/src/utils/cli/server_setup/docker_support.rs` 约 954 行
- 判断：这些文件已经同时承担了流程编排、平台分支、参数解析、错误映射和日志输出，多数逻辑还带环境耦合，后续修一个点很容易牵出别的分支。
- 建议：
  1. 先按“命令构建 / 状态探测 / IO / 错误翻译 / 平台兼容”拆子模块。
  2. `manager.rs` 保留编排，具体运行时细节下沉到 `runtime_*` 子模块。
  3. 先拆测试最薄弱的路径，不要一次性大搬家。

### 2. HTTP 适配层目前更像“把桌面命令直接平移到 Web”

- 证据：`src-tauri/src/adapters/http/command_registry/server.rs` 与插件注册器里大量 handler 只是参数解包后直调原命令层
- 判断：这样做前期快，但 Web 暴露面和桌面内调用的信任边界并不一样，继续堆接口会让安全补丁越来越难补。
- 建议：
  1. 为 HTTP 模式单独定义“允许暴露”的服务接口。
  2. 命令注册表按风险等级分组，不要默认全开。
  3. 把请求校验、鉴权、审计、限流前置到统一中间层。

## 测试缺口

- 目前能看到 `validate_ssrf_url` 的安全测试，但没有看到对 `handle_file_upload` 的路径穿越和内存行为测试。
- 没看到 `run_http_server` 的鉴权、CORS 策略、暴露命令面测试。
- `execute_program` 的前台超时目前缺少“超时后子进程被终止”的回归测试。

## 并行 Agent 拆分

下面 4 个 Agent 可以并行工作，刻意按文件边界拆开，避免同时编辑同一份文件。

### Agent 1：HTTP 安全面收口

目标：收紧 headless HTTP 的暴露面，优先处理绑定地址、鉴权、CORS 和危险命令暴露。

工作边界：

- 只改 `src-tauri/src/runtime/*`
- 只改 `src-tauri/src/adapters/http/server.rs`
- 只改 `src-tauri/src/adapters/http/mod.rs` 与必要测试文件
- 不要编辑 `src-tauri/src/adapters/http/command_registry/*`

提示词：

> 你负责审查并收紧 SeaLantern 的 headless HTTP 暴露面。先确认 `RuntimeMode::detect()`、`run_http_server()`、CORS 策略和鉴权边界，再提出最小可落地的修正方案。重点检查默认绑定 `0.0.0.0`、`allow_origin(Any)`、未鉴权调用高权限接口的问题。不要编辑 `src-tauri/src/adapters/http/command_registry/*`，避免和别的 Agent 冲突。输出内容要包含风险说明、修改建议、需要补的测试，以及如果要落代码应改哪些文件。

### Agent 2：上传与文件 IO 安全

目标：审查并修正上传接口和文件路径处理，重点查路径穿越、流式写入、大小限制。

工作边界：

- 只改 `src-tauri/src/adapters/http/server.rs`
- 只改与上传相关的新增测试文件
- 不要编辑 `src-tauri/src/runtime/*`
- 不要编辑 `src-tauri/src/adapters/http/command_registry/*`

提示词：

> 你负责 SeaLantern HTTP 上传链路的安全审查。重点看 `handle_file_upload()` 是否存在文件名拼接、目录逃逸、一次性读入内存、缺少单文件限制的问题。请给出明确的修复思路，优先使用 basename 归一化、目录边界校验、流式写盘和更细的限额控制。不要改 `src-tauri/src/runtime/*` 和 `src-tauri/src/adapters/http/command_registry/*`，避免文件冲突。输出时请给出需要补的测试场景。

### Agent 3：插件运行时进程与权限模型

目标：审查插件执行进程与权限声明的一致性，重点查假超时和权限白名单漂移。

工作边界：

- 只改 `src-tauri/src/plugins/runtime/**`
- 只改 `src-tauri/src/plugins/loader.rs`
- 只改 `src-tauri/tests/unit/plugins_runtime_*` 与 `plugins_loader_tests.rs`
- 不要编辑 `src-tauri/src/adapters/http/**`

提示词：

> 你负责 SeaLantern 插件运行时审查，重点检查 `sl.process.exec` 的前台执行超时是否真实生效，以及 `plugins/loader.rs` 的权限白名单是否和运行时、文档一致。请优先找出会导致阻塞、权限声明失效或维护分裂的点。不要编辑 `src-tauri/src/adapters/http/**`，避免和 HTTP 审查 Agent 冲突。输出需要给出风险、建议修复方向，以及建议新增的测试。

### Agent 4：服务器管理复杂度拆分建议

目标：从可维护性和复杂度角度审查服务器管理主链路，给出拆分优先级。

工作边界：

- 只读 `src-tauri/src/services/server/**`
- 只读 `src-tauri/src/utils/cli/**`
- 不直接编辑任何代码文件
- 只允许新增一份审查文档

提示词：

> 你负责 SeaLantern 后端复杂度审查，不直接改代码。请围绕 `services/server` 和 `utils/cli` 的大文件、职责耦合和测试难点，给出一个能分阶段执行的拆分建议。重点看 `docker_itzg.rs`、`local_helper.rs`、`manager.rs`、`server_setup/docker_support.rs` 这些大文件。不要编辑源码，只输出审查结论、拆分优先级、风险和建议的测试补位方案。

## 建议执行顺序

1. 先做 Agent 1 和 Agent 2，因为这是直接暴露面的安全问题。
2. 再做 Agent 3，补插件运行时的真实约束。
3. 最后让 Agent 4 出拆分方案，避免在安全修补前做大重构。

## Review 标准

- 完成度：是否真的覆盖了复杂度、安全性、可维护性三个维度。
- 额外异常：是否在修复过程中引入了新的暴露面、权限漂移、路径问题或测试回退。
- 合格线：不仅指出问题，还要能落到具体文件、具体边界和具体测试。
