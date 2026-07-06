<div align="center">
  
<img src="frontend/src/assets/logo.svg" alt="logo" width="200" height="200">

# 海晶灯（Sea Lantern）

一个轻量化的 Minecraft 服务器管理工具，基于 Tauri 2 + Rust + Vue 3

<div style="display: flex; justify-content: center; gap: 12px; margin-bottom: 12px; flex-wrap: wrap;">
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/stargazers"><img src="https://img.shields.io/github/stars/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Stars" alt="GitHub Stars"></a>
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/network/members"><img src="https://img.shields.io/github/forks/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Forks" alt="GitHub Forks"></a>
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/releases/latest"><img src="https://img.shields.io/github/v/release/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=%E6%9C%80%E6%96%B0%E7%89%88%E6%9C%AC" alt="GitHub Latest"></a>
</div>

<div style="display: flex; justify-content: center; gap: 12px; flex-wrap: wrap;">
  <a href="https://gitee.com/fps_z/SeaLantern/stargazers"><img src="https://gitee.com/fps_z/SeaLantern/badge/star.svg?theme=dark" alt="Gitee Stars"></a>
  <a href="https://gitee.com/fps_z/SeaLantern/members"><img src="https://gitee.com/fps_z/SeaLantern/badge/fork.svg?theme=dark" alt="Gitee Forks"></a>
</div>

<kbd>简体中文</kbd> <kbd>[English](docs/README/README-en.md)</kbd>

## 有问题？尝试→[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/SeaLantern-Studio/SeaLantern)

</div>

## 快速开始

> Tips:实际上，我们拥有一个文档站!在那里你可以更直观和方便的观看各种文档!可以点击 [这里](https://docs.ideaflash.cn/zh/intro) 跳转

下载 [正式版](https://github.com/SeaLantern-Studio/SeaLantern/releases/latest)

下载 [开发预览版](https://github.com/SeaLantern-Studio/SeaLantern-Preview/releases/latest)

## 开发

你需要 `Node.js 22+` 和 `Rust 1.70+`。

同时请安装`pnpm`和`cargo`。

**你需要先 Fork 源仓库的 `beta` 分支，然后在你自己的仓库进行开发工作。**

如果你只是想要查看最新进度，可以直接拉取源仓库：

```bash
git clone https://github.com/SeaLantern-Studio/SeaLantern.git
cd SeaLantern
git switch beta
```

项目的包管理器经过投票，从`npm`切换至`pnpm`。

前端与后端：

```bash
pnpm --dir frontend install
pnpm --dir frontend run tauri:dev
```

开发入口：

```bash
# 桌面 full：当前默认桌面开发入口，会编译默认 Cargo graph
pnpm --dir frontend run tauri:dev
pnpm --dir frontend run tauri:dev:desktop-full

# 桌面 min：可选桌面最小入口，显式透传 --no-default-features
pnpm --dir frontend run tauri:dev:desktop-min

# headless backend：只启动 HTTP / Docker 后端入口，不启动桌面壳
pnpm --dir frontend run dev:http:backend

# 页面预览：只启动 Vite 并打开对应路由，不改变 Cargo graph
pnpm --dir frontend run dev
pnpm --dir frontend run next:home
pnpm --dir frontend run next:plugins
pnpm --dir frontend run next:settings
```

其中：

- `tauri:dev` / `tauri:dev:desktop-full` 会进入 `backend/tauri-host`。
- `tauri:dev:desktop-min` 会进入 `backend/tauri-host`，并通过 `tauri dev -- --no-default-features` 关闭默认 Cargo features；它是可选入口，不替代默认 full。
- `tauri:build` / `tauri:build:desktop-full` 对应默认桌面构建；`tauri:build:desktop-min` 对应 `--no-default-features` 的可选桌面构建入口。
- `dev:http:backend` 会进入 `backend/docker-entry`，只用于 headless HTTP 后端开发。
- `dev` / `next:*` 只是前端页面预览命令，不会减少 Rust 编译量。

`desktop-min` 的当前正式场景语义可按 **metadata-only desktop** 理解：

- 保留桌面壳、前端页面、基础 Tauri 命令、服务器元数据管理、插件元数据读取/安装/扫描/设置等非 runtime 面。
- 显式关闭默认 Cargo features：`docker`、`online-tunnel`、`plugin-local-runtime`、`plugin-runtime-bridge`、`plugin-builtin-runtime`。
- 因此它不是 `desktop-full` 的替代品，也不是新的默认入口，而是一个 opt-in 的场景化最小桌面路径。
- 在这个场景下，runtime 相关插件能力按契约降级：toggle / runtime callback / snapshot / permission log 等返回 unavailable、no-op 或空结果；Docker / online tunnel 相关能力也不应被当作可用主路径。

### 低内存机器可选开发路径

默认开发路径不变：

- 桌面 full 仍以 `pnpm --dir frontend run tauri:dev` 为主。
- 提交前后端检查仍以本节下方的 workspace 基线为主。
- 正常机器不需要因为这份说明主动降到 low-memory 档。

如果你在 `16GiB` 或更低配置机器上开发，可以按需启用下面这些本地建议；它们都是 opt-in，不是仓库默认配置：

1. **rust-analyzer 收口**
   - 在你自己的编辑器用户设置里，把 linked project 优先收窄到 `backend/tauri-host/Cargo.toml`。
   - 不要默认打开 `all-features`。
   - on-save check 优先只跑当前 package 或当前 linked project。
2. **日常 backend scoped check**
   - 日常迭代可先跑 `cargo check --manifest-path backend/tauri-host/Cargo.toml`。
   - 提交前再回到下方的 `cargo check --workspace` 和 `cargo clippy --workspace -- -D warnings` 基线。
3. **按场景切入口**
   - 如果你只做 HTTP / Docker backend 路径，可选 `pnpm --dir frontend run dev:http:backend`，不要默认起桌面壳。
   - 如果你明确想走不带默认 Cargo features 的桌面路径，可选 `pnpm --dir frontend run tauri:dev:desktop-min`；它是 opt-in，不要把它当成默认入口。
4. **本地轻量 debug/profile 建议**
   - 如果 `tauri:dev` 仍然容易顶高峰值内存，可以只在你自己的本地 Cargo / IDE 配置里尝试更轻的 debug 信息或调试档。
   - 这类设置不要直接提交成 `.cargo/config.toml`、`.vscode/settings.json` 之类的仓库默认降级配置。

部分 Linux 发行版，例如 Arch，如果直接使用 `pnpm --dir frontend run tauri:dev` 可能不会编译成功，请检查你的依赖库是否完全，建议你在运行上述命令时使用包管理器提前安装 `Tauri` 的依赖以避免出现依赖不存在问题。[点击前往"Tauri | 前置要求"](https://tauri.app/zh-cn/start/prerequisites/#linux)

仅前端：

```bash
pnpm --dir frontend run dev
```

### 代码质量检查

提交代码前，我们**建议**运行以下命令来检查代码质量：

<details><summary>前端检查</summary>

```bash
# 代码质量检查
pnpm --dir frontend run lint

# 类型检查并验证生产构建
pnpm --dir frontend run build:check

# 自动修复可修复问题
pnpm --dir frontend run lint:fix

# 格式化代码
pnpm --dir frontend run fmt

# 检查代码格式
pnpm --dir frontend run fmt:check
```

</details>

<details><summary>后端检查</summary>

```bash
# 检查代码格式
cargo fmt --all -- --check

# 编译检查
cargo check --workspace

# 运行 Clippy 检查
cargo clippy --workspace -- -D warnings

# 格式化代码
cargo fmt --all
```

</details>

项目已配置 CI 自动检查，确保所有提交的代码都符合规范。

### 提交检查

CI 会在 PR/推送时校验代码质量与相关规范。

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust + Tauri 2
- **通信**: Tauri invoke
- **Docker**: itzg/minecraft-server

没有 Electron，没有 Node 后端，没有 Webpack。启动快，体积小，内存省。

> 我们使用 Webview 作为前端渲染，Webview 是现代计算机系统中自带的应用，前后端内存占用基本不超过70MiB

### 项目结构

详见 [项目结构](docs/STRUCTURE.md)。

### CLI 服务器入口

当前仓库已经提供统一的 `sealantern server ...` CLI 入口，可同时覆盖本地 Java 服务端与 `itzg/minecraft-server` Docker 运行方式。

使用说明见 [CLI 服务器运行指南](docs/cli-server-runtime-guide.md)。

## 待开发功能

这些功能的位置都预留好了，代码骨架是现成的，等你来写：

- 备份管理 - 世界存档的增量备份和还原
- 内网穿透 - 集成 FRP
- 定时任务 - 自动重启、定时备份、定时执行命令
- 资源管理 - 从 Modrinth 和 CurseForge 搜索安装插件和模组

## 交流群

QQ 交流群：**293748695**，欢迎加入讨论！

## 参与开发

欢迎贡献代码！在开始之前，请阅读[贡献指南](docs/CONTRIBUTING.md)以了解代码规范和开发流程。

界面也是。颜色在 CSS 变量里，组件是独立的，不喜欢就换。
想做个主题皮肤？做。想把整个布局推翻重来？也行。

当然，这一切的前提是你有足够的理由和能力，并且与群内的各位商讨后才能做，不然我们很有可能会**拒收 PR**

### 怎么贡献

1. Fork 这个仓库的`beta`分支
2. 建分支写代码
3. 提 Pull Request
4. 你的名字会出现在关于页面的贡献者墙上

不会写代码也行。说你想要什么功能，或者画个 UI 草图发出来，只要核实有用，都算贡献。

### i18n 国际化支持指南

Sea Lantern 支持多语言国际化，包括简体中文、繁体中文和英文等. [i18n 国际化指南](docs/language-system.md)

除了当前已有的常见语言，想要加额外语言，请制作插件。

## License

[GNU General Public License v3.0](LICENSE)

## Star History

<a href="https://www.star-history.com/#SeaLantern-Studio/SeaLantern&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=SeaLantern-Studio/SeaLantern&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=SeaLantern-Studio/SeaLantern&type=date&legend=top-left" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=SeaLantern-Studio/SeaLantern&type=date&legend=top-left" />
 </picture>
</a>

## 贡献者

感谢所有为 Sea Lantern 做出贡献的人！

[![Contributors](https://sealentern-contributors.sb4893.workers.dev/)](https://github.com/SeaLantern-Studio/SeaLantern/graphs/contributors)

## 致谢

Sea Lantern 是一个开源项目，遵循 GPLv3 协议。

Minecraft 是 Mojang AB 的注册商标。
本项目未经 Mojang 或 Microsoft 批准，也不与 Mojang 或 Microsoft 关联。

“我们搭建了骨架，而灵魂，交给你们。”
