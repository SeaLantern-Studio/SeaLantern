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

```
sea-lantern/
│
├── src/                                前端代码（Vue 3 + TypeScript）
│   │
│   ├── api/                           与 Rust 后端通信的封装层
│   │   ├── tauri.ts                  基础 invoke 封装，所有 API 的入口
│   │   ├── server.ts                 服务器管理 API（创建、启动、停止、日志）
│   │   ├── java.ts                   Java 环境检测 API
│   │   ├── config.ts                 配置文件读写 API
│   │   ├── player.ts                 玩家管理 API（白名单、封禁、OP）
│   │   ├── settings.ts               应用设置 API
│   │   ├── system.ts                 系统信息、文件对话框 API
│   │   └── update.ts                 软件更新检查 API
│   │
│   ├── components/                    UI 组件
│   │   ├── common/                   通用组件（整个项目的积木块）
│   │   │   ├── SLButton.vue         按钮组件
│   │   │   ├── SLCard.vue           卡片容器
│   │   │   ├── SLInput.vue          输入框组件
│   │   │   ├── SLSelect.vue         下拉选择组件
│   │   │   ├── SLSwitch.vue         开关组件
│   │   │   ├── SLModal.vue          弹窗组件
│   │   │   ├── SLProgress.vue       进度条组件
│   │   │   └── SLBadge.vue          状态标签组件
│   │   │
│   │   ├── layout/                   页面布局组件
│   │   │   ├── AppLayout.vue        总布局（左侧栏 + 右侧内容区）
│   │   │   ├── AppSidebar.vue       侧边导航栏
│   │   │   └── AppHeader.vue        顶部标题栏
│   │   │
│   │   └── splash/                   启动画面
│   │       └── SplashScreen.vue     应用启动时的加载动画
│   │
│   ├── views/                         页面视图（每个路由对应一个）
│   │   ├── HomeView.vue              首页（服务器列表、系统状态）
│   │   ├── CreateServerView.vue     创建/导入服务器页面
│   │   ├── ConsoleView.vue          控制台页面（实时日志、命令输入）
│   │   ├── ConfigView.vue           配置编辑页面（server.properties）
│   │   ├── PlayerView.vue           玩家管理页面（白名单、封禁、OP）
│   │   ├── SettingsView.vue         应用设置页面
│   │   └── AboutView.vue            关于页面（贡献者墙、更新检查）
│   │
│   ├── stores/                        Pinia 状态管理
│   │   ├── index.ts                  Pinia 实例初始化
│   │   ├── serverStore.ts           服务器列表和运行状态
│   │   ├── consoleStore.ts          控制台日志（切换页面不丢失）
│   │   └── uiStore.ts               界面状态（侧栏折叠等）
│   │
│   ├── styles/                        全局样式
│   │   ├── variables.css            CSS 变量（颜色、间距、圆角、字体、阴影）
│   │   ├── reset.css                浏览器样式重置
│   │   ├── typography.css           排版样式
│   │   ├── animations.css           动画关键帧
│   │   └── glass.css                毛玻璃效果样式
│   │
│   ├── data/                          静态数据
│   │   └── contributors.ts          贡献者信息列表
│   │
│   ├── router/                        路由配置
│   │   └── index.ts                 路由表定义
│   │
│   ├── App.vue                        根组件
│   ├── main.ts                        应用入口（初始化 Vue、Pinia、Router）
│   └── style.css                      样式汇总导入
│
├── src-tauri/                         后端代码（Rust + Tauri 2）
│   │
│   ├── src/
│   │   │
│   │   ├── commands/                 Tauri 命令（前端 invoke 调用的 API）
│   │   │   ├── mod.rs               模块导出
│   │   │   ├── server.rs            服务器管理命令
│   │   │   ├── java.rs              Java 检测命令
│   │   │   ├── config.rs            配置文件读写命令
│   │   │   ├── player.rs            玩家管理命令
│   │   │   ├── settings.rs          应用设置命令
│   │   │   ├── system.rs            系统信息、文件对话框命令
│   │   │   └── update.rs            软件更新检查命令
│   │   │
│   │   ├── services/                业务逻辑层
│   │   │   ├── mod.rs               模块导出
│   │   │   ├── server_manager.rs   服务器进程管理、日志读取
│   │   │   ├── java_detector.rs    Java 环境扫描器
│   │   │   ├── config_parser.rs    .properties 文件解析器
│   │   │   ├── player_manager.rs   玩家数据文件读取
│   │   │   ├── settings_manager.rs 应用设置持久化
│   │   │   └── global.rs            全局单例管理器
│   │   │
│   │   ├── models/                  数据结构定义
│   │   │   ├── mod.rs               模块导出
│   │   │   ├── server.rs            服务器实例、状态数据结构
│   │   │   ├── config.rs            配置项数据结构
│   │   │   ├── settings.rs          应用设置数据结构
│   │   │   └── dev_config.rs        开发者配置数据结构
│   │   │
│   │   ├── utils/                   工具函数
│   │   │   └── mod.rs               工具模块
│   │   │
│   │   ├── lib.rs                   Tauri 库入口（插件注册、命令注册）
│   │   └── main.rs                  应用主函数
│   │
│   ├── capabilities/                 Tauri 权限配置
│   │   └── default.json             默认权限设置
│   │
│   ├── icons/                        应用图标
│   │   ├── 32x32.png
│   │   ├── 128x128.png
│   │   ├── icon.icns                macOS 图标
│   │   └── icon.ico                 Windows 图标
│   │
│   ├── Cargo.toml                    Rust 依赖配置
│   ├── Cargo.lock                    Rust 依赖锁定文件
│   ├── tauri.conf.json              Tauri 配置（窗口大小、标题、版本等）
│   └── build.rs                      构建脚本
│
├── public/                            静态资源
│
├── index.html                         HTML 入口文件
├── package.json                       Node.js 依赖配置
├── package-lock.json                  Node.js 依赖锁定文件
├── vite.config.ts                     Vite 构建配置
├── tsconfig.json                      TypeScript 配置
├── tsconfig.node.json                 Node.js 环境 TypeScript 配置
├── .gitignore                         Git 忽略文件配置
└── README.md                          项目说明文档（你正在看的这个）
```

## 已实现功能

### 服务器管理

- 导入 JAR 文件创建服务器，一键启动和停止
- 数据保存到 JSON，重启软件不丢失

### 实时控制台

- 后端用独立线程读 stdout 和 stderr
- 前端每 800ms 轮询拉新日志
- 支持命令输入、Tab 补全、上下键历史、快捷指令按钮
- 日志存在全局 store 里，切页面不丢

### Java 检测

- 启动时扫描 A 到 Z 所有盘符
- 递归搜索常见安装路径，包括 .minecraft/runtime 里 MC 自带的 Java
- 按版本号排序，标记推荐

### 配置编辑

- 读取 server.properties，解析成带描述和分类的结构化数据
- 布尔值用开关，枚举用下拉，数字和字符串用输入框
- 改完直接写回文件

### 玩家管理

- 读取 whitelist.json / banned-players.json / ops.json 显示列表
- 添加和移除通过向运行中的服务器发送 MC 命令实现
- 解析日志判断在线玩家

### 应用设置

- 关闭软件时自动停止所有服务器（默认开启）
- 自动同意 EULA
- 默认内存、端口、JVM 参数，全部可配

### 软件更新

- 检查来自 Gitee（或Github） 的发行版，获取最新版本信息
- 显示更新日志，一键打开下载链接
- 版本号自动比较，提示用户更新

## 待开发功能

这些功能的位置都预留好了，代码骨架是现成的，等你来写：

- 下载中心 - 下载服务端核心，插件，模组
- 备份管理 - 世界存档的增量备份和还原
- 内网穿透 - 集成 FRP
- 定时任务 - 自动重启、定时备份、定时执行命令
- 资源管理 - 从 Modrinth / CurseForge 搜索安装插件和 Mod
- 暗色主题 - CSS 变量都准备好了，加一套 dark 的值就行
- 国际化 - 目前全是中文硬编码，可以抽成语言文件

## 参与开发

欢迎贡献代码！在开始之前，请阅读 [贡献指南](CONTRIBUTING.md) 了解代码规范和开发流程。

界面也是。颜色在 CSS 变量里，组件是独立的，不喜欢就换。
想做个主题皮肤？做。想把整个布局推翻重来？也行。

### 怎么贡献

1. Fork 这个仓库
2. 建分支写代码（遵循 [贡献指南](CONTRIBUTING.md)）
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

## License

GPLv3

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=FPSZ/SeaLantern&type=Date)](https://star-history.com/#FPSZ/SeaLantern&Date)

## 贡献者

感谢所有为 Sea Lantern 做出贡献的人！

[![Contributors](https://contrib.rocks/image?repo=FPSZ/SeaLantern)](https://github.com/FPSZ/SeaLantern/graphs/contributors)

## 致谢

Sea Lantern 是一个开源项目，遵循 GPLv3 协议。

Minecraft 是 Mojang Studios 的注册商标。本项目与 Mojang 无关。

"我们搭建了骨架，而灵魂，交给你们。"
