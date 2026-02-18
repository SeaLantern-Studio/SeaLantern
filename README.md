<div align="center">
<img src="https://gitee.com/fps_z/SeaLantern/raw/master/src/assets/logo.svg" alt="logo" width="200" height="200">

# 海晶灯(Sea Lantern)

一个轻量化的 Minecraft 服务器管理工具 ，基于 Tauri 2 + Rust + Vue 3

[![star](https://gitee.com/fps_z/SeaLantern/badge/star.svg?theme=dark)](https://gitee.com/fps_z/SeaLantern/stargazers)[![fork](https://gitee.com/fps_z/SeaLantern/badge/fork.svg?theme=dark)](https://gitee.com/fps_z/SeaLantern/members)
[![GitHub Repo stars](https://img.shields.io/github/stars/FPSZ/SeaLantern?style=flat&logo=github&label=stars)](https://github.com/FPSZ/SeaLantern)[![GitHub forks](https://img.shields.io/github/forks/FPSZ/SeaLantern?style=flat&logo=github&label=forks)](https://github.com/FPSZ/SeaLantern/network/members)
[![最新版本](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fgitee.com%2Fapi%2Fv5%2Frepos%2FFPS_Z%2FSeaLantern%2Freleases%2Flatest&query=%24.tag_name&label=最新版本&color=brightgreen&logo=gitee&style=flat)](https://gitee.com/FPS_Z/SeaLantern/releases)[![GitHub release](https://img.shields.io/github/v/release/FPSZ/SeaLantern?style=flat&logo=github&label=latest)](https://github.com/FPSZ/SeaLantern/releases)
====

</div>

> 您正在浏览简体中文版的readme，点击[此处](README-en.md)前往英文版

> You are browsing the Simplified Chinese version of the readme. Click [here](README-en.md) to go to the English version

![img](https://gitee.com/fps_z/markdown/raw/master/img/about2.png)

## 能干什么

- 控制台实时看日志，直接输命令
- server.properties 图形化编辑，不用手改文件
- 白名单、封禁、OP 一键管理
- 关软件的时候自动帮你停服务器，不会丢存档
- 检查更新，一键下载新版本

## 快速开始

- 如果你是使用者，在右侧下载Release版本，导入一个服务端 JAR 文件，选一个 Java，点启动。就这么简单。

- 如果你是开发者，需要 Node.js 20+ 和 Rust 1.70+。

```bash
git clone https://github.com/FPSZ/SeaLantern.git
cd SeaLantern
npm install
npm run tauri dev
```

构建发布版：

```bash
npm run tauri build
```

产物在 `src-tauri/target/release/bundle/` 里。

### 代码质量检查

提交代码前，建议运行以下命令检查代码质量：

前端检查：

```bash
# 代码质量检查
npm run lint

# 自动修复可修复问题
npm run lint:fix

# 格式化代码
npm run fmt

# 检查格式（不改动文件）
npm run fmt:check
```

后端检查：

```bash
# 检查代码格式
cargo fmt --all -- --check

# 运行 Clippy 检查
cargo clippy --workspace -- -D warnings

# 自动格式化代码
cargo fmt --all
```

项目已配置 CI 自动检查，确保所有提交的代码都符合规范。

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite + Pinia
- **后端**: Rust + Tauri 2
- **样式**: 纯 CSS
- **通信**: Tauri invoke（前端调 Rust 函数，直接拿返回值）

没有 Electron，没有 Node 后端，没有 Webpack。启动快，体积小，内存省。

## 项目结构
 - 详见 [项目结构](docs/项目结构.md)

## 待开发功能

这些功能的位置都预留好了，代码骨架是现成的，等你来写：

- 下载中心 - 下载服务端核心，插件，模组
- 备份管理 - 世界存档的增量备份和还原
- 内网穿透 - 集成 FRP
- 定时任务 - 自动重启、定时备份、定时执行命令
- 资源管理 - 从 Modrinth / CurseForge 搜索安装插件和 Mod

## 交流群

QQ 交流群：**293748695**，欢迎加入讨论！

## 参与开发

欢迎贡献代码！在开始之前，请阅读 [贡献指南](docs/CONTRIBUTING.md) 了解代码规范和开发流程。

界面也是。颜色在 CSS 变量里，组件是独立的，不喜欢就换。
想做个主题皮肤？做。想把整个布局推翻重来？也行。

### 怎么贡献

1. Fork 这个仓库
2. 建分支写代码（遵循 [贡献指南](docs/CONTRIBUTING.md)）
3. 提 Pull Request
4. 你的名字会出现在关于页面的贡献者墙上

不会写代码也行。说你想要什么功能，或者画个 UI 草图发出来，都算贡献。

### 添加新功能

假设你要加一个「备份管理」功能：

**后端**：

1. `src-tauri/src/services/` 下建 `backup_manager.rs`，写逻辑
2. `src-tauri/src/commands/` 下建 `backup.rs`，写 Tauri 命令
3. 在 `commands/mod.rs` 里加 `pub mod backup`
4. 在 `lib.rs` 的 `generate_handler!` 宏里注册命令

**前端**：

1. `src/api/` 下建 `backup.ts`，封装 invoke 调用
2. `src/views/` 下建 `BackupView.vue`，画页面
3. `src/router/index.ts` 里加路由
4. `AppSidebar.vue` 的 `navItems` 数组里加一项

前后端各三个文件，路由和侧栏各改一行。

### i18n 国际化支持指南

Sea Lantern 支持多语言国际化，包括简体中文、繁体中文和英文。如果你需要添加新的翻译或语言支持，请参考 [i18n 国际化指南](docs/i18n-help.md)。

## License

GPLv3

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=FPSZ/SeaLantern&type=Date)](https://star-history.com/#FPSZ/SeaLantern&Date)

## 贡献者

感谢所有为 Sea Lantern 做出贡献的人！

[![Contributors](https://sealentern-contributors.sb4893.workers.dev/)](https://github.com/FPSZ/SeaLantern/graphs/contributors)

## 致谢

Sea Lantern 是一个开源项目，遵循 GPLv3 协议。

Minecraft 是 Mojang Studios 的注册商标。本项目与 Mojang 无关。

"我们搭建了骨架，而灵魂，交给你们。"
