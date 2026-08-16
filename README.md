<div align="center">
  
<img src="src/assets/logo.svg" alt="logo" width="200" height="200">

# 海晶灯（Sea Lantern）

一个轻量化的 Minecraft 服务器管理工具

<div style="display: flex; justify-content: center; gap: 12px; margin-bottom: 12px; flex-wrap: wrap;">
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/stargazers"><img src="https://img.shields.io/github/stars/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Stars" alt="GitHub Stars"></a>
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/network/members"><img src="https://img.shields.io/github/forks/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Forks" alt="GitHub Forks"></a>
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/releases/latest"><img src="https://img.shields.io/github/v/release/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=%E6%9C%80%E6%96%B0%E7%89%88%E6%9C%AC" alt="GitHub Latest"></a>
</div>

<kbd>简体中文</kbd> <kbd>[English](README-en.md)</kbd>

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/SeaLantern-Studio/SeaLantern)

</div>

## 能干什么

- [x] 下载服务器核心
- [x] 客制化开服器体验
- [x] 方便直观的更改配置(目前仅支持原版的`server.properties`)
- [x] 快捷控制台命令
- [ ] JVM 预设与分享社区
- [ ] 在 Docker 容器化环境下运行

## 快速开始

| 我想要……               | 前往                                                |
| ---------------------- | --------------------------------------------------- |
| 下载并安装 Sea Lantern | [下载安装](https://docs.ideaflash.cn/zh/download)   |
| 第一次创建或导入服务器 | [使用教程](https://docs.ideaflash.cn/zh/tutorial)   |
| 不知道该选择哪种服务端 | [核心获取](https://docs.ideaflash.cn/zh/server-jar) |
| 遇到使用问题或异常情况 | [常见问题](https://docs.ideaflash.cn/zh/faq)        |

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust + Tauri 2（Desktop 宿主）+ Axum（Web 宿主）
- **通信**: Desktop 使用 Tauri IPC/Event，Web 使用 HTTP/WebSocket/SSE，插件使用宿主 Bridge
- **Docker**: itzg/minecraft-server

没有 Electron，没有 Node 后端，没有 Webpack。启动快，体积小，内存省。

`application` 提供宿主无关的公共业务编排，`src-tauri` 与 `server` 分别作为 Desktop、Web 宿主，`interface` 只定义两端真正共用的契约。可复用的前端业务通过 `src/api` 接入，宿主专用页面和功能可以独立演进。详见[项目架构与代码组织](docs/architecture.md)。

> 使用系统 Webview 渲染。

## 待开发功能

- 备份管理 - 世界存档的增量备份和还原
- 定时任务 - 自动重启、定时备份、定时执行命令
- 资源管理 - 从 Modrinth 和 CurseForge 搜索安装插件和模组
- 内网穿透 - 集成 FRP，为现有联机功能提供更稳定可靠的连接方案

## 给开发者

开发前需要准备：

| 依赖         | 版本   |
| ------------ | ------ |
| Node.js      | 24 LTS |
| Rust         | stable |
| pnpm         | 9.15.9 |
| Only（可选） | 0.0.7+ |

如果你还没有配置开发环境，可以先查看 [环境配置](https://docs.ideaflash.cn/zh/dev/environment)。

拉取项目：

```bash
git clone https://github.com/SeaLantern-Studio/SeaLantern.git
cd SeaLantern
```

安装依赖并启动桌面开发环境：

```bash
pnpm install
pnpm tauri dev
```

只预览前端页面：

```bash
pnpm dev
```

如果你在 Linux 上开发，可能需要先安装 Tauri 相关系统依赖。具体请看 [Tauri Linux 前置要求](https://tauri.app/zh-cn/start/prerequisites/#linux)。

仓库根目录提供了 [`Onlyfile`](Onlyfile)，用于统一常用的开发、构建和检查命令。可以按需安装 [Only](https://github.com/KercyDing/only)：

```bash
cargo install only
```

安装后，在项目根目录运行以下命令查看所有可用任务：

```bash
only
```

常用命令包括 `only dev`、`only build`、`only check`、`only test` 和 `only ci`。

### 代码检查

提交代码前，我们**建议**运行完整的本地 CI：

```bash
only ci
```

该命令会执行前后端静态检查和全部测试；如果没有安装 Only，也可以分别运行以下命令：

<details><summary>前端检查</summary>

```bash
# 代码质量检查
pnpm lint

# 类型检查并验证生产构建
pnpm build:check

# 自动修复可修复问题
pnpm lint:fix

# 格式化代码
pnpm fmt

# 检查代码格式
pnpm fmt:check
```

</details>

<details><summary>后端检查</summary>

```bash
# 格式化
cargo fmt --all

# 编译检查
cargo check --all-targets --workspace

# 运行 Clippy 检查
cargo clippy --all-targets --workspace -- -D warnings
```

</details>

项目已配置 CI，会在推送和提交 Pull Request 时自动检查代码质量。

## 参与开发

我们欢迎任何形式的贡献：代码、文档、翻译、问题反馈、功能建议，或者 UI 草图都可以。

1. Fork 仓库
2. 新建自己的开发分支
3. 完成修改并通过基本检查
4. 提交 Pull Request

涉及整体 UI、项目架构等较大改动时，请先在交流群或 GitHub Issues 中讨论；缺少充分理由的重大改动可能不会被合并。

## 社区与反馈

如果你在使用中遇到问题，或者想参与讨论，可以通过以下方式联系我们：

- QQ 一群：**293748695**
- QQ 二群：**1085823754**
- 问题反馈：[GitHub Issues](https://github.com/SeaLantern-Studio/SeaLantern/issues)

## 贡献者

感谢所有为 Sea Lantern 做出贡献的人！

[![Contributors](https://sealentern-contributors.sb4893.workers.dev/)](https://github.com/SeaLantern-Studio/SeaLantern/graphs/contributors)

## 许可证

[GNU Affero General Public License v3.0](LICENSE)

## 致谢

Minecraft 是 Mojang AB 的注册商标。本项目未经 Mojang 或 Microsoft 批准，也不与 Mojang 或 Microsoft 关联。

> 我们搭建了骨架，而灵魂，交给你们。
