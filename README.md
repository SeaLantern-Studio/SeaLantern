<div align="center">
  
<img src="src/assets/logo.svg" alt="logo" width="200" height="200">

# 海晶灯（Sea Lantern）

一个轻量化的 Minecraft 服务器管理工具，基于 Tauri 2 + Rust + Vue 3

|[![github-stars](https://img.shields.io/github/stars/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Stars)](https://github.com/SeaLantern-Studio/SeaLantern/stargazers)|[![github-forks](https://img.shields.io/github/forks/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Forks)](https://github.com/SeaLantern-Studio/SeaLantern/network/members)|[![github-latest](https://img.shields.io/github/v/release/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=%E6%9C%80%E6%96%B0%E7%89%88%E6%9C%AC)](https://github.com/SeaLantern-Studio/SeaLantern/releases/latest)|
|:--|:--|:--|
|[![gitee-stars](https://gitee.com/fps_z/SeaLantern/badge/star.svg?theme=dark)](https://gitee.com/fps_z/SeaLantern/stargazers)|[![gitee-forks](https://gitee.com/fps_z/SeaLantern/badge/fork.svg?theme=dark)](https://gitee.com/fps_z/SeaLantern/members)|[![gitee-latest](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fgitee.com%2Fapi%2Fv5%2Frepos%2FFPS_Z%2FSeaLantern%2Freleases%2Flatest&query=%24.tag_name&label=%E6%9C%80%E6%96%B0%E7%89%88%E6%9C%AC&color=brightgreen&logo=gitee&style=flat)](https://gitee.com/FPS_Z/SeaLantern/releases)|

<kbd>简体中文</kbd> <kbd>[English](README-en.md)</kbd>

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

您将会需要 Node.js 20+ 和 Rust 1.70+。

```bash
git clone https://github.com/SeaLantern-Studio/SeaLantern.git
cd SeaLantern
npm install
npm run tauri dev
```

部分 Linux 发行版，例如 Arch 直接使用 `npm run tauri dev` 可能不会编译成功，请检查您的依赖库是否完全，建议您在运行上述命令时使用包管理器提前安装 `Tauri` 的依赖以避免出现依赖不存在问题。[点击前往"Tauri | 前置要求"](https://tauri.app/zh-cn/start/prerequisites/#linux)

构建发布版：

```bash
npm run tauri build
```

产物在 `src-tauri/target/release/bundle/` 里。

### 代码质量检查

提交代码前，我们建议运行以下命令来检查代码质量：

- 前端检查

> ```bash
> # 代码质量检查
> npm run lint
> 
> # 自动修复可修复问题
> npm run lint:fix
> 
> # 格式化代码
> npm run fmt
> 
> # 检查代码格式
> npm run fmt:check
> ```

- 后端检查

> ```bash
> # 检查代码格式
> cargo fmt --all -- --check
> 
> # 运行 Clippy 检查
> cargo clippy --workspace -- -D warnings
> 
> # 格式化代码
> cargo fmt --all
> ```

项目已配置 CI 自动检查，确保所有提交的代码都符合规范。

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite + Pinia
- **后端**: Rust + Tauri 2
- **样式**: 纯 CSS
- **通信**: Tauri invoke（前端调 Rust 函数，直接拿返回值）

没有 Electron，没有 Node 后端，没有 Webpack。启动快，体积小，内存省。

### 项目结构

详见 [项目结构](docs/项目结构.md)。

## 待开发功能

这些功能的位置都预留好了，代码骨架是现成的，等你来写：

- 下载中心 - 下载服务端核心，插件，模组
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

### 怎么贡献

1. Fork 这个仓库
2. 建分支写代码
3. 提 Pull Request
4. 你的名字会出现在关于页面的贡献者墙上

不会写代码也行。说你想要什么功能，或者画个 UI 草图发出来，都算贡献。

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
