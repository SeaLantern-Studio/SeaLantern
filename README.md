<div align="center">
  
<img src="src/assets/logo.svg" alt="logo" width="200" height="200">

# 海晶灯（Sea Lantern）

一个轻量化的 Minecraft 服务器管理工具，基于 Tauri 2 + Rust + Vue 3

<div style="display: flex; justify-content: center; gap: 12px; margin-bottom: 12px; flex-wrap: wrap;">
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/stargazers"><img src="https://img.shields.io/github/stars/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Stars" alt="GitHub Stars"></a>
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/network/members"><img src="https://img.shields.io/github/forks/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Forks" alt="GitHub Forks"></a>
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/releases/latest"><img src="https://img.shields.io/github/v/release/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=最新版本" alt="GitHub Latest"></a>
</div>

<div style="display: flex; justify-content: center; gap: 12px; flex-wrap: wrap;">
  <a href="https://gitee.com/fps_z/SeaLantern/stargazers"><img src="https://gitee.com/fps_z/SeaLantern/badge/star.svg?theme=dark" alt="Gitee Stars"></a>
  <a href="https://gitee.com/fps_z/SeaLantern/members"><img src="https://gitee.com/fps_z/SeaLantern/badge/fork.svg?theme=dark" alt="Gitee Forks"></a>
</div>

<kbd>简体中文</kbd> <kbd>[English](README-en.md)</kbd>

有问题？尝试→[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/SeaLantern-Studio/SeaLantern)
---

</div>

![img](https://gitee.com/fps_z/markdown/raw/master/img/about2.png)

## 能干什么

- 控制台实时看日志，直接输命令
- server.properties 图形化编辑，不用手改文件
- 白名单、封禁、OP 一键管理
- 关软件的时候自动帮你停服务器，不会丢存档
- 检查更新，一键下载新版本

## 快速开始

下载 [release](https://github.com/SeaLantern-Studio/SeaLantern/releases/latest) 版本，导入一个服务端 JAR 文件，选一个 Java，点启动。就这么简单。

## 开发

你需要 `Node.js 20+` 和 `Rust 1.70+`。

同时请安装`pnpm`和`cargo`。

**您需要先 Fork 源仓库，然后在你自己的仓库进行开发工作。**

如果你只是想要查看最新进度，可以直接拉取源仓库：

```bash
git clone https://github.com/SeaLantern-Studio/SeaLantern.git
cd SeaLantern
```

项目的包管理器经过投票，从`npm`切换至`pnpm`。

前端与后端：

```bash
pnpm install
pnpm run tauri dev
```

部分 Linux 发行版，例如 Arch，如果直接使用 `pnpm run tauri dev` 可能不会编译成功，请检查你的依赖库是否完全，建议你在运行上述命令时使用包管理器提前安装 `Tauri` 的依赖以避免出现依赖不存在问题。[点击前往"Tauri | 前置要求"](https://tauri.app/zh-cn/start/prerequisites/#linux)

仅前端：

```bash
pnpm dev
```

构建发布版：

```bash
pnpm run tauri build
```

产物在 `src-tauri/target/release/bundle/` 里。

### 代码质量检查

提交代码前，我们建议运行以下命令来检查代码质量：

<details><summary>前端检查</summary>

```bash
# 代码质量检查
pnpm run lint

# 自动修复可修复问题
pnpm run lint:fix

# 格式化代码
pnpm run fmt

# 检查代码格式
pnpm run fmt:check
```

</details>

<details><summary>后端检查</summary>

```bash
# 检查代码格式
cargo fmt --all -- --check

# 运行 Clippy 检查
cargo clippy --workspace -- -D warnings

# 格式化代码
cargo fmt --all
```

</details>

项目已配置 CI 自动检查，确保所有提交的代码都符合规范。

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite + Pinia
- **后端**: Rust + Tauri 2
- **样式**: 纯 CSS
- **通信**: Tauri invoke（前端调 Rust 函数，直接拿返回值）

没有 Electron，没有 Node 后端，没有 Webpack。启动快，体积小，内存省。

> 我们使用 Webview 作为前端渲染，Webview 是现代计算机系统中自带的应用，前后端内存占用基本不超过70MiB

### 项目结构

详见 [项目结构](docs/STRUCTURE.md)。

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

1. Fork 这个仓库的`dev`分支
2. 建分支写代码
3. 提 Pull Request
4. 你的名字会出现在关于页面的贡献者墙上

我们对AI编程，即`Vibe Coding`有一定限制：仅修复，不重构，不大改，人工审。

- 仅修复：由于目前大部分 AI 的能力局限性，如果要完全依赖 AI 是很不现实的。

- 不重构：AI 的上下文和抽象理解能力都不足以让AI重构已有内容，当然也许会有比较幸运的重构完还能用，但那只是个例。

- 不大改：**不要让 AI 擅自改动任何一个影响巨大内容**。

- 人工审：使用完 AI 一定要人工审查一遍是否有误，如果不会审，可以去群里找管理，要记得有礼貌的提问而不是骚扰管理。

不会写代码也行。说你想要什么功能，或者画个 UI 草图发出来，只要核实有用，都算贡献。

### 添加新功能

假设你要加一个「备份管理」功能：

#### 后端

1. `src-tauri/src/services/` 下建 `backup_manager.rs`，写逻辑
2. `src-tauri/src/commands/` 下建 `backup.rs`，写 Tauri 命令
3. 在 `commands/mod.rs` 里加 `pub mod backup`
4. 在 `lib.rs` 的 `generate_handler!` 宏里注册命令

#### 前端

1. `src/api/` 下建 `backup.ts`，封装 invoke 调用
2. `src/views/` 下建 `BackupView.vue`，画页面
3. `src/router/index.ts` 里加路由
4. `AppSidebar.vue` 的 `navItems` 数组里加一项

前后端各三个文件，路由和侧栏各改一行。

### i18n 国际化支持指南

Sea Lantern 支持多语言国际化，包括简体中文、繁体中文和英文等. [i18n 国际化指南](src/language/README.md)

除了当前已有的常见语言，想要加额外语言，请制作插件。

## License

[GNU General Public License v3.0](LICENSE)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=SeaLantern-Studio/SeaLantern&type=Date)](https://star-history.com/#SeaLantern-Studio/SeaLantern&Date)

## 贡献者

感谢所有为 Sea Lantern 做出贡献的人！

[![Contributors](https://sealentern-contributors.sb4893.workers.dev/)](https://github.com/SeaLantern-Studio/SeaLantern/graphs/contributors)

## 致谢

Sea Lantern 是一个开源项目，遵循 GPLv3 协议。

Minecraft 是 Mojang AB 的注册商标。
本项目未经 Mojang 或 Microsoft 批准，也不与 Mojang 或 Microsoft 关联。

“我们搭建了骨架，而灵魂，交给你们。”
