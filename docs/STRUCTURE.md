# Sea Lantern 项目结构

## 项目概览

Sea Lantern（海晶灯）是一个基于 Tauri 2、Rust 与 Vue 3 的 Minecraft 服务器管理工具。当前仓库已经包含应用前端、Tauri 后端、Lua 插件运行时、文档、打包脚本与辅助入口等多层结构。

## 顶层目录

```text
sea-lantern/
├── .example_plugin/              # 示例插件目录，用于参考 manifest、脚本与资源组织方式
├── .github/                      # GitHub Issue、PR 模板与 CI 工作流
├── .vscode/                      # VS Code 工作区设置
├── .zed/                         # Zed 编辑器配置
├── docker/                       # Docker 构建相关文件
├── docs/                         # 项目文档
├── packaging/                    # Arch Linux 打包脚本与桌面文件
├── panic-log/                    # 崩溃日志目录占位
├── scripts/                      # 构建、版本、版权等辅助脚本
├── frontend/                     # 前端工程根目录（Vue 3 + TypeScript）
├── backend/                      # Tauri / Rust 后端目录
├── .editorconfig                 # 编辑器基础格式规范
├── .gitattributes                # Git 属性配置
├── .gitignore                    # Git 忽略规则
├── .oxfmtrc.json                 # oxfmt 配置
├── .oxlintrc.json                # Oxlint 配置
├── Cargo.lock                    # Rust 依赖锁文件（workspace）
├── Cargo.toml                    # Rust workspace 清单
├── frontend/index.html           # 前端 HTML 入口
├── LICENSE                       # GPLv3 许可证
├── NOTICE                        # 第三方声明
├── frontend/package.json         # 前端依赖与脚本定义
├── frontend/pnpm-lock.yaml       # pnpm 锁文件
├── frontend/pnpm-workspace.yaml  # pnpm 工作区与安装脚本策略
├── README.md                     # 中文说明文档
├── README-en.md                  # 英文说明文档
├── rustfmt.toml                  # Rust 格式化配置
```

## 前端结构 `frontend/`

```text
frontend/
├── index.html                    # 前端 HTML 入口
├── package.json                  # 前端依赖与脚本定义
├── vite.config.ts                # Vite 配置
├── tsconfig.json                 # TypeScript 配置
└── src/
    ├── api/                      # 前端到 Tauri 命令的调用封装
    ├── assets/                   # 静态资源
    ├── language/                 # 前端国际化资源
    ├── stores/                   # Pinia 状态管理
    ├── styles/                   # 全局与页面样式
    ├── themes/                   # 主题定义
    ├── utils/                    # 前端工具函数
    ├── App.vue                   # Vue 根组件
    ├── main.ts                   # 应用入口
    └── style.css                 # 样式总入口
```

### `frontend/src/api/`

当前仓库中的前端 API 封装主要包括：

- `java.ts`：Java 检测与环境管理
- `logging.ts`：日志能力
- `mcs_plugins.ts`：Minecraft 服务端插件文件管理
- `player.ts`：玩家管理
- `plugin.ts`：Sea Lantern 自身 Lua 插件系统、插件市场、UI 快照与权限日志接口
- `remoteLocales.ts`：远程语言包
- `server.ts`：服务器生命周期与文件相关操作
- `settings.ts`：应用设置
- `system.ts`：系统信息与系统级能力
- `tauri.ts`：统一 `invoke` 封装
- `tunnel.ts`：内网穿透/隧道相关接口
- `update.ts`：应用更新
- `upload.ts`：上传相关接口

其中 [`frontend/src/api/plugin.ts`](../frontend/src/api/plugin.ts) 与 [`frontend/src/api/mcs_plugins.ts`](../frontend/src/api/mcs_plugins.ts) 分别对应两套不同插件体系：

1. Sea Lantern 的 Lua 插件系统
2. Minecraft 服务端自身的插件文件管理

### `frontend/src/language/`

该目录存放前端语言包：

- `index.ts`：语言加载入口
- 多个 `*.json`：具体语言资源，如 `zh-CN.json`、`en-US.json`、`ja-JP.json` 等

### `frontend/src/stores/`

当前已存在的状态模块包括：

- `consoleStore.ts`：控制台日志缓存
- `contextMenuStore.ts`：上下文菜单状态
- `createServerDraft.ts`：创建服务器草稿状态
- `i18nStore.ts`：语言状态
- `pluginStore.ts`：插件状态
- `serverStore.ts`：服务器状态
- `settingsStore.ts`：设置状态
- `uiStore.ts`：界面状态
- `updateStore.ts`：更新状态
- `index.ts`：Pinia 入口

### `frontend/src/styles/`

样式目录已按“全局基础样式 + 页面样式”拆分：

- `animations.css`
- `app.css`
- `glass.css`
- `initial.css`
- `plugin-list.css`
- `reset.css`
- `typography.css`
- `variables.css`
- `views/`：各页面专用样式，如 `ConfigView.css`、`ConsoleView.css`、`CreateServerView.css`、`TunnelView.css`

### `frontend/src/themes/`

主题目录包含：

- `default.ts`
- `midnight.ts`
- `ocean.ts`
- `rose.ts`
- `sunset.ts`
- `index.ts`

## 后端结构 `backend/`

当前后端不是单一 crate，而是“主宿主 crate + 多个共享 core crate”的结构。

```text
backend/
├── docker-core/                 # Docker 启动参数、预览、JVM env 合成等共享纯逻辑
├── docker-entry/                # Docker / headless HTTP 模式下的独立可执行入口
├── java-installer-core/         # Java 运行时下载、解压与安装逻辑
├── runtime-core/                # 日志、headless HTTP、路径、状态、Tokio 运行时等共享工具
├── server-config-core/          # 启动配置、JVM 参数、CPU policy、server.properties 共享规则
├── server-download-links-core/  # 服务端下载链接解析
├── server-installer-core/       # 服务端核心类型识别、压缩包解压、MC 版本推断
├── server-local-setup-core/     # 本地建服、接入、启动前路径与元数据辅助
├── server-plugin-core/          # Minecraft 服务端插件 JAR 与配置文件扫描
├── server-startup-scan-core/    # 启动候选项扫描
├── starter-links-core/          # Starter 下载链接缓存与解析
├── tauri-host/                  # 主 Tauri 宿主 crate
└── update-core/                 # 更新检查、下载、挂起安装与安装计划
```

每个 crate 根目录现在都应以 `Agents.md` 作为该模块的低上下文说明入口。需要快速判断模块职责时，优先阅读对应 crate 根目录下的 `Agents.md`。

## 主宿主结构 `backend/tauri-host/`

```text
backend/tauri-host/
├── capabilities/                 # Tauri capability 配置
├── icons/                        # 应用图标资源
├── locales/                      # 后端本地化资源
├── src/                          # Rust 源码
├── tests/                        # Rust / Tauri 集成测试
├── .gitignore                    # Git 忽略配置
├── build.rs                      # 构建脚本
├── Cargo.toml                    # 后端依赖清单
└── tauri.conf.json               # Tauri 应用配置
```

```text
backend/docker-entry/
├── Cargo.toml                    # Docker 环境下的独立 Rust 入口 crate
└── src/
    └── main.rs                   # 容器启动入口，转调 sea-lantern 的 docker 运行模式
```

### `backend/tauri-host/src/`

```text
backend/tauri-host/src/
├── commands/                     # Tauri 命令层，直接暴露给前端 invoke
├── models/                       # 序列化数据结构与领域模型
├── services/                     # 业务服务层
├── utils/                        # 通用工具
├── lib.rs                        # Tauri 应用装配入口
└── main.rs                       # 桌面程序入口
```

### `backend/tauri-host/src/commands/`

当前命令模块包括：

- `config.rs`
- `debug.rs`
- `downloader.rs`
- `java.rs`
- `join.rs`
- `logging.rs`
- `mcs_plugin.rs`
- `mods.rs`
- `player.rs`
- `plugin.rs`
- `server.rs`
- `server_id.rs`
- `settings.rs`
- `system.rs`
- `tunnel.rs`
- `update.rs`
- 以及多个更新细分模块：`update_arch.rs`、`update_checksum.rs`、`update_cnb.rs`、`update_download.rs`、`update_github.rs`、`update_install.rs`、`update_types.rs`、`update_version.rs`

其中：

- [`backend/tauri-host/src/commands/plugin.rs`](../backend/tauri-host/src/commands/plugin.rs) 负责 Sea Lantern Lua 插件系统的安装、启停、市场、UI 快照、上下文菜单回调与权限日志读取。
- [`backend/tauri-host/src/commands/mcs_plugin.rs`](../backend/tauri-host/src/commands/mcs_plugin.rs) 负责指定 Minecraft 服务器目录下的插件文件扫描、启停、删除与安装。

### `backend/tauri-host/src/models/`

当前模型文件包括：

- `config.rs`
- `download.rs`
- `mcs_plugin.rs`
- `plugin.rs`
- `server.rs`
- `settings.rs`
- `mod.rs`

其中 [`backend/tauri-host/src/models/plugin.rs`](../backend/tauri-host/src/models/plugin.rs) 既定义插件清单结构，也定义权限元数据、依赖描述、插件市场信息、侧边栏配置、UI 页面配置等核心类型。

### `backend/tauri-host/src/services/`

当前可见服务模块包括：

- `async_loader.rs`
- `settings_manager.rs`

从命令层引用关系还能看出，项目内还存在若干全局服务/管理器，例如服务器管理器、MCS 插件管理器、插件管理器与国际化服务等，它们承担底层业务逻辑并被命令层复用。

### `backend/tauri-host/src/utils/`

当前工具模块包括：

- `cli.rs`
- `constants.rs`
- `downloader.rs`
- `logger.rs`
- `path.rs`
- `mod.rs`

## Lua 插件运行时结构

Sea Lantern 的插件系统核心位于 `backend/tauri-host/src/plugins/runtime/`。虽然当前工作区文件列表未完整展开该目录，但从命令调用与源码引用可确认其主要结构与职责如下：

```text
backend/tauri-host/src/plugins/runtime/
├── core/                         # 运行时创建、沙箱与 `sl` 命名空间挂载
├── filesystem/                   # `sl.fs` 文件系统 API
├── plugins_api/                  # `sl.plugins` 插件目录访问 API
├── ...                           # 其余 API 命名空间，如 console / element / server / ui 等
```

### `runtime/core/`

- 负责创建 Lua VM。
- 在初始化时将旧权限 `fs` 归一化为 `fs.data`。
- 根据插件声明的权限动态挂载 `sl.log`、`sl.storage`、`sl.fs`、`sl.api`、`sl.ui`、`sl.element`、`sl.server`、`sl.console`、`sl.system`、网络模块、进程执行模块与 `sl.plugins`。
- 未授权命名空间会被替换为权限拒绝模块。
- 始终挂载 i18n 相关能力。

### `runtime/plugins_api/`

该目录实现 `sl.plugins`：

- `mod.rs`：注册 `list`、`get_manifest`、`read_file`、`file_exists`、`list_files`、`write_file`
- `common.rs`：插件根目录解析、清单读取与权限日志辅助
- `query.rs`：只读类接口
- `write.rs`：写入类接口

已知行为特征：

- 仅当插件声明 `plugin_folder_access` 时启用。
- 根目录基于“当前插件目录的父目录”推导。
- 所有目标路径都会经过沙箱校验。
- 单文件读取/写入大小上限为 10 MiB。
- 写入时禁止修改 `manifest.json`，并拒绝符号链接路径。

### `runtime/filesystem/`

该目录实现 `sl.fs`：

- 通过 `data` / `server` / `global` 三个 scope 管理文件访问范围。
- 既支持 scope 级权限，也支持 action 级权限，例如 `fs.data.read`、`fs.server.write`。
- 显式拒绝符号链接与重解析点。
- 删除时拒绝删除 sandbox 根目录本身。
- 会记录所有 API 调用的权限日志。

## 其它目录

### `docs/`

当前文档目录包括：

- `Agents-Example-Module.md`
- `Agents.md`
- `CONTRIBUTING.md`
- `design/` # 设计方案与设计草案目录
- `STRUCTURE.md`
- `language-system.md` # 前端 i18n 系统使用说明（原 src/language/README.md）
- `plugin_api.md`
- `plugin/` # 按命名空间和单个 API 拆分的插件 Lua API 文档
- `theme-system.md` # 主题系统使用说明（原 src/themes/README.md）
- `lua-api/` # 插件运行时各模块 Lua API 聚合文档，作为原始参考来源保留

### `scripts/`

当前脚本包括：

- `notice.mjs`
- `version.mjs`

分别用于声明文件与版本流程的辅助处理。

### `panic-log/`

当前仅包含 `.gitkeep`，用于保留崩溃日志目录结构。

## 结构理解建议

如果你准备继续扩展功能，可按以下路径定位：

- 新增前端调用封装：`frontend/src/api/`
- 新增 Tauri 命令：`backend/tauri-host/src/commands/`
- 新增领域模型：`backend/tauri-host/src/models/`
- 新增后端业务逻辑：`backend/tauri-host/src/services/`
- 扩展 Lua 插件 API：`backend/tauri-host/src/plugins/runtime/`
- 更新项目文档：`docs/`

对于插件相关开发，建议先明确你操作的是：

1. Sea Lantern 自身 Lua 插件系统
2. Minecraft 服务端插件文件管理

两者入口、权限模型与代码位置均不同，文档与实现也应分别维护。
