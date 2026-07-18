/**
 * 帮助文档静态数据
 * 从远程 GitHub 拉取并清理 Vue 语法后写死的 Markdown 内容
 * 避免了运行时网络请求和 Vue 模板污染
 */

const DOCS_BASE = "https://docs.ideaflash.cn";

/** 项目简介 - 特性部分之上的内容 */
export const introTop = `# 项目简介
Sea Lantern（海晶灯）是一个**轻量化**的 Minecraft 服务器管理工具。  
基于 **Tauri 2 + Rust + Vue 3** 构建。`;

/** 项目简介 - 特性部分之下的内容 */
export const introFooter = `## 特性导读
> 以上是 Sea Lantern 的主要功能概览，更多详细信息请查看各功能页面。
## 技术栈
- **前端**: Vue 3 + TypeScript + Vite + Pinia
- **后端**: Rust + Tauri 2
- **样式**: CSS Variables 设计系统 + 主题引擎
- **图表**: ECharts
- **包管理**: pnpm
- **代码检查**: oxlint + oxfmt
没有 Electron，没有 Node 后端，没有 Webpack。启动快，体积小，内存省。
## 交流
QQ 交流群：**293748695**，欢迎加入讨论！
## 开源协议
[GNU General Public License v3.0](https://github.com/SeaLantern-Studio/SeaLantern/blob/main/LICENSE)`;

/** 特性卡片数据 */
export interface IntroFeature {
  title: string;
  desc: string;
  note: string;
}

export const introFeatures: IntroFeature[] = [
  {
    title: "实时控制台",
    desc: "实时查看服务器日志，直接输入命令，支持命令历史。",
    note: "适合排查启动报错和日常运维，不必频繁切换外部终端。",
  },
  {
    title: "图形化配置",
    desc: "server.properties 可视化编辑，按分类组织，减少手改文件。",
    note: "常见参数可快速定位；涉及重启生效的项仍遵循服务端规则。",
  },
  {
    title: "玩家管理",
    desc: "白名单、封禁、OP 一键操作，常用管理集中在一个入口。",
    note: "在线状态与权限操作集中展示，减少手动记忆复杂指令。",
  },
  {
    title: "插件系统",
    desc: "基于 Lua 脚本扩展，支持自定义 UI 组件、右键菜单与插件市场。",
    note: "插件权限面板与市场体验在 v1.0.0 持续优化，状态保留更稳定。",
  },
  {
    title: "创建流程 2.0",
    desc: "支持 JAR/脚本/已有服务器导入，智能识别开服方式并可自定义命令。",
    note: "可在创建阶段自动安装整合包，减少首次开服的手动配置量。",
  },
  {
    title: "主题系统",
    desc: "内置 5 套主题，支持明暗模式；Windows 支持亚克力效果。",
    note: "主题切换不影响功能布局，主要改变视觉风格与可读性。",
  },
  {
    title: "多语言支持",
    desc: "内置 10 种语言，覆盖中文、英语、日语、韩语等。支持运行时切换。",
    note: "支持运行时切换语言，常用术语在界面与文档中尽量保持一致。",
  },
  {
    title: "Java 管理",
    desc: "自动检测已安装 Java，并支持一键下载安装。",
    note: "扫描路径在 v1.0.0 扩展，常见安装目录的识别率更高。",
  },
  {
    title: "Mod 管理",
    desc: "查看服务端已安装的 Mod/插件，并提供基础管理能力。",
    note: "当前以本地查看与基础维护为主，便于先排查冲突再调整。",
  },
  {
    title: "安全退出",
    desc: "关闭软件时自动停止服务器，降低存档损坏风险。",
    note: "避免直接强制结束进程，减少世界与玩家数据异常概率。",
  },
  {
    title: "自动更新",
    desc: "检查新版本并跳转下载页面（Arch Linux 使用 AUR 更新）。",
    note: "桌面端以下载引导为主，不会在后台静默覆盖你的安装。",
  },
  {
    title: "跨平台",
    desc: "支持 Windows、macOS、Linux（含 Arch Linux AUR）。",
    note: "各平台能力存在差异，文档会单独标注平台特定行为。",
  },
];

/** 帮助文档内容映射 */
export const helpDocs: Record<string, string> = {
  intro: introTop + "\n" + introFooter,
  download: `# 下载安装
## 最新版本
当前最新版本：
<strong>v1.3.0</strong>
按你的系统选择对应安装包下载并安装。  
- 建议优先使用安装包格式（Windows 选 exe，macOS 选 dmg）
- 较为特殊的是Docker
  - 此版本无头，可以在服务器运行
  - 此版本跨平台，可以在支持docker的所有平台运行
## Windows
| 格式 | 说明 |
|------|------|
| [exe 安装包](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_x64-setup.exe) | 推荐，双击安装 |
| [msi 安装包](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_x64_zh-CN.msi) | Windows Installer 格式 |
| [exe 安装包 (ARM64)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_arm64-setup.exe) | 适用于 Windows on ARM |
| [msi 安装包 (ARM64)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_arm64_zh-CN.msi) | Windows Installer (ARM64) |
## macOS
| 格式 | 说明 |
|------|------|
| [dmg (Apple Silicon)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_aarch64.dmg) | M1 / M2 / M3 / M4 |
| [dmg (Intel)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_x64.dmg) | x64 架构 |
| [app.tar.gz (Apple Silicon)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_aarch64.app.tar.gz) | 便携压缩包 |
| [app.tar.gz (Intel)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_x64.app.tar.gz) | 便携压缩包 |
当前 Release 中的 \`dmg\` 与 \`app.tar.gz\` 均未进行 Apple 签名/公证，macOS 可能提示"已损坏，无法打开"或"无法验证开发者"。
- \`dmg\`：先双击打开 DMG，并将 \`Sea Lantern.app\` 拖到 \`/Applications\`，再在终端执行：
\`\`\`bash
xattr -dr com.apple.quarantine /Applications/Sea\\ Lantern.app
\`\`\`
- \`app.tar.gz\`：先解压，再在解压后的当前目录执行：
\`\`\`bash
xattr -dr com.apple.quarantine ./Sea\\ Lantern.app
\`\`\`
若仍被拦截，可在"系统设置 -> 隐私与安全性"中点击"仍要打开"，或右键应用后选择"打开"。
## Linux
| 格式 | 说明 |
|------|------|
| [deb](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_amd64.deb) | Debian / Ubuntu |
| [deb (ARM64)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_arm64.deb) | Debian / Ubuntu ARM64 |
| [rpm](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern-1.3.0-1.x86_64.rpm) | Fedora / RHEL (x86_64) |
| [rpm (ARM64)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern-1.3.0-1.aarch64.rpm) | Fedora / RHEL (aarch64) |
| [AppImage](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_amd64.AppImage) | 通用格式 |
| [AppImage (ARM64)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_aarch64.AppImage) | 通用格式 (ARM64) |
| [pkg.tar.zst](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/sealantern-1.3.0-1-x86_64.pkg.tar.zst) | Arch Linux 可安装包 (x86_64) |
Arch Linux 用户可通过 AUR 安装，例如：
\`\`\`bash
paru -S sealantern
\`\`\`
### Ubuntu PPA（社区维护）
Ubuntu 用户可通过 PPA 快速安装：
\`\`\`bash
sudo add-apt-repository ppa:brianeee7878/sealantern
sudo apt update
sudo apt install sea-lantern-ppa-updater
\`\`\`
该 PPA 为社区维护渠道，不属于官方发布渠道；如遇问题请改用上方 deb 安装包。
## Docker
- 安装docker。详见 [安装docker](https://www.bilibili.com/video/BV1vm421T7Kw/?share_source=copy_web&vd_source=67ab86499fd78344263cc23e969c3fe4)
- 拉取最新的海晶灯docker镜像
  \`\`\`bash
  docker pull penetr4t10n/sealantern:latest
  \`\`\`
- 运行最新的海晶灯录像
  \`\`\`
  docker run -d \\
    --name sealantern \\
    -p 3000:3000 \\
    -p 25565:25565/tcp \\
    penetr4t10n/sealantern:latest
  \`\`\`
:::tip 注意
docker内的网络环境是隔离的，因此需要暴露web端口和服务器端口
:::
## 系统要求
- Windows 10+ / macOS 10.15+ / Linux (glibc 2.31+)
- Java 8及以上（用于运行 Minecraft 服务端）
## Java要求
- 来自Minecraft Wiki的Minecraft对Java的运行标准：https://zh.minecraft.wiki/w/Java%E7%89%88
- Java配置
  * 自1.12（17w13a）起，运行Minecraft的最低要求为Java 8。若玩家不确定设备是否已安装Java 8，安装程序会默认自动安装并使用支持Minecraft运行的Java版本。
  * 自1.17（21w19a）起，运行Minecraft的最低要求为Java 16。
  * 自1.18（1.18-pre2）起，运行Minecraft的最低要求为Java 17。
  * 自1.20.5（24w14a）起，运行Minecraft的最低要求为Java 21，且操作系统须为64位。
  * 自26.1（26.1-snapshot-1）起，运行Minecraft的最低要求为Java 25。
- 需要注意的是，若使用的Java版本与操作系统位数不匹配（32位或64位）、使用部分Java 7版本，或安装多个Java版本来运行Minecraft Java版，可能会出现一些运行问题。
- 使用Java运行时需要稳定的互联网连接，用于下载游戏文件、验证用户名以及连接多人游戏服务器。至少需要连接一次互联网以完成Java运行时的下载和运行；之后可在断开互联网连接的情况下游玩，但接收更新或联机仍需在联网状态下进行。
- Java运行时无法在Windows RT平板上运行。
- Java运行时不在Chromebook上原生支持，但在设置Crouton后可运行（即Java版Minecraft）。`,
  "getting-started": `# 快速开始
## 系统要求
| 项目 | 最低要求 |
|------|---------|
| 操作系统 | Windows 10+、macOS 10.15+、Linux (glibc 2.31+) |
| Java | Java 8+（用于运行 Minecraft 服务端） |
| 内存 | 建议 4GB 以上（Minecraft 服务端需要额外内存） |
::: tip Windows 版本说明
Sea Lantern 使用 WebView2 运行时，**要求 Windows 10（版本 1909 及以上）或 Windows 11**。Windows 7/8/8.1 用户请参考 [常见问题 - 旧版 Windows](${DOCS_BASE}/zh/faq#旧版-windows-运行方案) 中的替代方案。
:::
## 1. 下载安装
前往 [下载页面](${DOCS_BASE}/zh/download) 获取适合你操作系统的安装包，双击运行即可完成安装。
Ubuntu 用户也可以使用 PPA 安装（见下载页 Linux 小节）。
Arch Linux 用户可通过 AUR 安装（见下载页 Linux 小节）。
## 2. 获取服务端核心
你可以使用 Minecraft 服务端 JAR 创建新服务器，也可以直接导入已有服务器目录或启动脚本。如果你还没有核心文件，请参考 [核心获取](${DOCS_BASE}/zh/server-jar) 页面下载。
::: tip 推荐
对于大多数玩家，推荐使用 [Paper](https://papermc.io/downloads/paper) — 性能优秀、插件生态丰富。
:::
## 3. 配置 Java
启动 Sea Lantern 后，软件会**自动检测**系统中已安装的 Java。如果没有合适的版本，可以使用内置的 Java 下载器一键安装。
::: info
Minecraft 不同版本对 Java 的要求不同：
- 1.16.5 及以下 → Java 8 或 Java 11
- 1.17 ~ 1.20.4 → Java 17
- 1.20.5+ → Java 21
:::
## 4. 创建服务器
1. 点击「创建服务器」按钮
2. 选择导入来源（JAR、已有服务器或启动脚本）
3. 按向导完成启动方式检测；如有需要可自定义开服命令
4. 选择 Java 运行时版本（缺失时可一键安装）
5. 为你的服务器命名并确认参数
6. 首次启动时需要同意 Minecraft EULA
## 5. 启动服务器
点击启动按钮，等待控制台显示 \`Done!\` 即表示服务器启动成功。此时你可以在 Minecraft 中通过 \`localhost\` 连接到你的服务器。
---
接下来，查看 [使用教程](${DOCS_BASE}/zh/tutorial) 了解更多操作细节，或浏览 [功能总览](${DOCS_BASE}/zh/features) 了解所有功能。`,
  "server-jar": `# 核心获取
Minecraft 服务器需要一个服务端核心（JAR 文件）才能运行。本页介绍常见的服务端类型及其获取方式。
## 原版服务端（Vanilla）
Mojang 官方提供的原版服务端，不支持任何插件或 Mod。
- 下载地址：[Minecraft 官网](https://www.minecraft.net/zh-hans/download/server)
- 适用场景：纯原版体验，不需要任何扩展
::: warning
原版服务端性能较低，建议仅用于少人数本地联机。
:::
## Paper（推荐）
基于 Spigot 的高性能服务端，是目前最流行的选择。
- 下载地址：[papermc.io](https://papermc.io/downloads/paper)
- 适用场景：需要插件、追求性能的服务器
- 插件兼容：支持 Bukkit / Spigot / Paper 插件
::: tip 为什么推荐 Paper？
- 性能远超原版和 Spigot，对红石和生物 AI 有优化
- 社区活跃，插件兼容性好
- 自带反作弊和安全修补
- 更新速度快，新版本支持及时
:::
## Spigot
老牌的插件服务端，Paper 的上游项目。
- 获取方式：通过 [BuildTools](https://www.spigotmc.org/wiki/buildtools/) 自行编译
- 适用场景：需要 Spigot 专属插件，或与 Paper 有兼容性问题时
::: info
Spigot 不提供直接下载的 JAR 文件，需要使用 BuildTools 编译。对新手不太友好，建议优先选择 Paper。
:::
## Forge
老牌 Mod 服务端，用于运行 Java 版的Forge端Mod。
- 下载地址：[Forge 官网](https://files.minecraftforge.net/)
- 适用场景：需要运行 Forge Mod 的模组服务器
- 注意：**不兼容 Bukkit/Spigot 插件**
## Fabric
轻量级 Mod 加载器，启动速度快。
- 下载地址：[Fabric 官网](https://fabricmc.net/use/installer/)
- 适用场景：需要运行 Fabric Mod 的模组服务器
- 注意：**不兼容 Bukkit/Spigot 插件，需要使用 Fabric Mod**大部分 Fabric Mod 还需要您将 Fabric API 安装到 mods 文件夹中。来源：[Fabic 中文网](https://fabricmc.net.cn/use/server/)
## Purpur
基于 Paper 的分支，提供更多配置选项。
- 下载地址：[Purpur 官网](https://purpurmc.org/downloads)
- 适用场景：需要更多自定义配置选项的服务器
- 插件兼容：支持 Bukkit / Spigot / Paper 插件
## Pumpkin（实验性）
基于 Rust 的新型服务器项目。
- 官网：[pumpkinmc.org](https://pumpkinmc.org/)
- 下载页：[Pumpkin Download](https://pumpkinmc.org/download/)
- 文档：[docs.pumpkinmc.org](https://docs.pumpkinmc.org/)
- 适用场景：技术尝鲜、测试 Rust 服务器方案
- 发布形式：常见为可执行文件（不是 \`.jar\`）
- 性能：整体表现很强，当前主要短板是生态兼容性
::: warning
Pumpkin 与传统 Bukkit/Paper 生态不同，当前主要风险是生态兼容性。建议先在测试环境验证，再用于正式服。
如果下载的是可执行文件，请在 Sea Lantern 中使用脚本启动模式（\`bat\` / \`sh\`）调用该可执行文件，而不是按 JAR 导入。
:::
## 如何选择？
| 类型 | 插件支持 | Mod 支持 | 性能 | 推荐度 |
|------|---------|---------|------|-------|
| Paper | Bukkit/Spigot/Paper | ❌ | ⭐⭐⭐ | ⭐⭐⭐ |
| Purpur | Bukkit/Spigot/Paper | ❌ | ⭐⭐⭐ | ⭐⭐ |
| Spigot | Bukkit/Spigot | ❌ | ⭐⭐ | ⭐⭐ |
| Vanilla | ❌ | ❌ | ⭐ | ⭐ |
| Forge | ❌ | Forge Mod | ⭐⭐ | ⭐⭐ |
| Fabric | ❌ | Fabric Mod | ⭐⭐⭐ | ⭐⭐ |
| Pumpkin | ⚠️ 与 Bukkit/Paper 生态不同 | ❌ | ⭐⭐⭐⭐ | ⭐⭐ |
## 注意事项
### 版本兼容性
服务端版本必须与客户端版本一致，否则玩家无法加入。例如：
- 服务端 1.21.4 → 客户端也必须是 1.21.4
- 部分模组服务端支持向下兼容，但建议保持一致
### Minecraft EULA
首次启动任何 Minecraft 服务端时，你需要同意 [Minecraft EULA](https://www.minecraft.net/eula)。Sea Lantern 会在创建服务器时引导你完成这一步。
### 下载后如何使用？
1. 大多数核心下载得到 \`.jar\` 文件（Pumpkin 等项目可能提供可执行文件）
2. 在 Sea Lantern 中点击「创建服务器」，选择导入来源（JAR / 既有服务器 / 启动脚本）
3. 若是 \`.jar\` 可直接导入；若是可执行文件，请使用脚本模式（\`bat\` / \`sh\`）
4. 创建向导会尝试自动识别开服方式；必要时可手动调整并填写自定义开服命令
5. 详细步骤请参考 [快速开始](${DOCS_BASE}/zh/getting-started)`,
  tutorial: `# 使用教程
本页详细介绍 Sea Lantern 的各项功能使用方法。
## 创建第一个服务器
### 导入来源（JAR / 已有服务器 / 启动脚本）
1. 点击主界面的「创建服务器」按钮
2. 在引导界面选择导入来源：服务端 JAR、已有服务器目录或启动脚本（\`bat\` / \`sh\`）
3. 如果还没有服务端，请参考 [核心获取](${DOCS_BASE}/zh/server-jar)
### 启动方式识别与自定义命令
- Sea Lantern 会根据导入内容智能识别启动方式
- 当识别结果不符合预期时，你可以手动切换并填写自定义开服命令
- 创建流程支持整合包自动安装，减少首次开服的手动步骤
### 选择 Java 版本
完成导入后，Sea Lantern 会列出系统中已检测到的 Java 版本。选择与你的 Minecraft 版本匹配的 Java：
| Minecraft 版本 | 推荐 Java |
|---------------|----------|
| 1.16.5 及以下 | Java 8 / 11 |
| 1.17 ~ 1.20.4 | Java 17 |
| 1.20.5+ | Java 21 |
::: tip
如果列表中没有合适的 Java，点击「下载 Java」按钮，Sea Lantern 会自动下载并安装所需版本。
:::
### 命名和配置
为你的服务器取一个名字。你也可以在这里调整初始内存分配等参数，但这些都可以稍后修改。
### 同意 EULA
首次启动时，Sea Lantern 会提示你同意 Minecraft EULA。确认后服务器才能正常运行。
[官方EULA解释](https://www.minecraft.net/zh-hans/eula)
## 控制台
控制台是 Sea Lantern 的核心界面，提供以下功能：
### 查看日志
服务器运行时的所有输出都会实时显示在控制台中，包括：
- 服务器启动/关闭信息
- 玩家加入/离开
- 聊天消息
- 错误和警告信息
### 输入命令
在控制台底部的输入框中输入 Minecraft 服务器命令（无需 \`/\` 前缀），按回车执行。
常用命令示例：
\`\`\`
say 欢迎来到服务器！
op Steve
whitelist add Alex
stop
\`\`\`
### 命令历史
按 \`↑\` / \`↓\` 箭头键可以浏览之前执行过的命令，方便重复操作。
### 快捷命令面板
控制台工具栏提供常用操作的快捷按钮，如安全停止服务器、重启等。
## server.properties 配置
Sea Lantern 提供了图形化的 server.properties 编辑器，让你无需手动修改文件。
### 常用配置项
| 配置项 | 说明 | 默认值 |
|-------|------|-------|
| \`server-port\` | 服务器端口 | 25565 |
| \`max-players\` | 最大玩家数 | 20 |
| \`difficulty\` | 游戏难度 | easy |
| \`gamemode\` | 默认游戏模式 | survival |
| \`motd\` | 服务器简介（显示在服务器列表） | A Minecraft Server |
| \`online-mode\` | 正版验证 | true |
| \`pvp\` | 是否允许 PVP | true |
| \`white-list\` | 是否启用白名单 | false |
### 搜索和筛选
配置项支持分类浏览和关键词搜索，可以快速定位你要修改的选项。
::: warning
修改配置后需要**重启服务器**才能生效。
:::
## 玩家管理
### 白名单
启用白名单后，只有白名单中的玩家才能加入服务器。
- 添加玩家：输入玩家名点击添加，或在控制台执行 \`whitelist add 玩家名\`
- 移除玩家：在列表中点击移除按钮
- 开关白名单：在 server.properties 中设置 \`white-list\` 为 \`true\` 或 \`false\`
### 封禁管理
- 封禁玩家：在玩家列表中点击封禁，或执行 \`ban 玩家名\`
- 解除封禁：在封禁列表中点击解封，或执行 \`pardon 玩家名\`
- IP 封禁：执行 \`ban-ip IP地址\`
### OP 权限
OP（管理员）拥有服务器的最高权限。
- 给予 OP：执行 \`op 玩家名\`
- 移除 OP：执行 \`deop 玩家名\`
::: warning
谨慎给予 OP 权限，拥有 OP 的玩家可以执行任何服务器命令。
:::
## 插件安装
::: info
插件功能仅适用于 Paper / Spigot 等支持插件的服务端。Vanilla、Forge 和 Fabric 服务端不支持 Bukkit 插件。
:::
### 获取插件
常见的插件下载渠道：
- [SpigotMC](https://www.spigotmc.org/resources/) — 最大的 Spigot 插件社区，需要注册账户（免费），注意检查插件的兼容性版本和评分
- [Modrinth](https://modrinth.com/plugins) — 现代化的插件/Mod 平台
- [Hangar](https://hangar.papermc.io/) — PaperMC 官方插件平台
- [Polymart](https://polymart.org/) — 类似 SpigotMC，界面更现代化
- [Bukkit](https://dev.bukkit.org/) — 老牌插件平台
- [GitHub](https://github.com) — 搜索开源插件项目，通常在 Releases 部分提供下载
### 安装步骤
1. **准备工作**
   - 确保服务器已停止（如果正在运行）
   - 找到服务器目录中的 \`plugins\` 文件夹
   - 如果没有 \`plugins\` 文件夹，可手动创建一个
2. **放置插件文件**
   - 将下载的插件 \`.jar\` 文件复制到 \`plugins\` 文件夹中
   - 确保插件文件名不包含特殊字符或空格
   - 建议每次只安装一个插件以避免冲突
3. **配置插件**
   - 启动服务器，插件会自动生成配置文件
   - 在 \`plugins\` 文件夹中找到插件的配置文件夹
   - 根据需要编辑配置文件（通常是 \`config.yml\`）
4. **重启服务器**
   - 重启服务器以使插件生效
   - 检查控制台日志确认插件加载成功
   - 使用 \`plugins\` 命令查看已加载的插件列表
### 常用插件推荐
**管理类插件：**
| 插件 | 功能 | 链接 |
|------|------|------|
| EssentialsX | 基础指令集（传送、家、经济等） | <https://essentialsx.net/downloads> | 
| LuckPerms | 权限管理 |<https://luckperms.net/download>|
| WorldEdit | 强大的地图编辑工具 |<https://modrinth.com/plugin/worldedit>|
| WorldGuard | 区域保护和权限管理 |<https://modrinth.com/plugin/worldguard/version/7.0.11>|
**服务器安全插件：**
| 插件 | 功能 | 链接 |
|------|------|------|
| Authme | 提供服务器登录密码保护（离线服务器必加） |<https://www.spigotmc.org/resources/authmereloaded.6269/>|
**游戏增强类插件：**
| 插件 | 功能 | 链接 |
|------|------|------|
| CoreProtect | 方块日志/回滚 |<https://modrinth.com/plugin/coreprotect>|
| Multiverse-Core | 多世界管理 |<https://modrinth.com/plugin/multiverse-core>|
| DiscordSRV | 服务器与 Discord 集成 |<https://www.spigotmc.org/resources/discordsrv.18494/>|
**经济系统插件：**
| 插件 | 功能 | 链接 |
|------|------|------|
| Vault | 经济/权限 API（需要配合经济插件使用） |<https://www.spigotmc.org/resources/vault.34315/>|
| Shopkeepers | 商人 NPC 系统 |<https://www.spigotmc.org/resources/shopkeepers.80756/>|
### 插件兼容性注意事项
- 确保插件与你的服务端和 Minecraft 版本兼容
- 检查插件之间的依赖关系
- 避免安装功能重复的插件
- 定期更新插件以获得新功能和安全修复
- 备份服务器数据后再安装新插件
## 高级设置
### 内存分配
Minecraft 服务器对内存需求较大，建议根据玩家数量合理分配：
| 玩家数 | 推荐内存 |
|-------|---------|
| 1-5 人 | 2-4 GB |
| 5-15 人 | 4-6 GB |
| 15-30 人 | 6-8 GB |
| 30+ 人 | 8 GB+ |
在 Sea Lantern 的服务器设置中可以调整内存分配。
### JVM 参数
Sea Lantern 允许你自定义 JVM 启动参数。对于 Paper 服务端，推荐使用 Aikar's Flags：
\`\`\`
-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200
-XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC
-XX:+AlwaysPreTouch -XX:G1NewSizePercent=30
-XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M
-XX:G1ReservePercent=20 -XX:G1HeapWastePercent=5
-XX:G1MixedGCCountTarget=4 -XX:InitiatingHeapOccupancyPercent=15
-XX:G1MixedGCLiveThresholdPercent=90
-XX:G1RSetUpdatingPauseTimePercent=5 -XX:SurvivorRatio=32
-XX:+PerfDisableSharedMem -XX:MaxTenuringThreshold=1
\`\`\`
::: warning
如果你不了解 JVM 参数的作用，建议使用默认配置。错误的参数可能导致服务器不稳定。
:::
### 安全退出
当你关闭 Sea Lantern 时，程序会自动向服务器发送 \`stop\` 命令，等待服务器安全关闭后再退出。这可以防止存档损坏。
::: danger
**不要**直接通过任务管理器强制关闭正在运行的服务器，这可能导致存档丢失或损坏。
:::
`,
  features: `# 功能总览
本页用于快速了解 Sea Lantern 现阶段可用能力与适用场景。  
如果你是首次使用，建议先看「服务器创建」「Java 管理」「控制台」这三部分。
## 控制台
- 实时查看服务器日志输出，覆盖启动阶段与运行阶段
- 直接输入命令，支持命令历史记录和快捷命令面板
- 工具栏提供清空显示、导出日志等常用操作
## server.properties 编辑器
- 图形化界面编辑配置项，不需要手动改文件
- 支持分类浏览和搜索筛选，定位配置更快
- 适合日常参数调整；部分参数仍需重启服务器生效
## 玩家管理
- 在线玩家状态查看
- 白名单管理
- 封禁管理
- OP 权限管理
- 支持踢出玩家等常见管理操作
## 插件系统
基于 Lua 脚本的插件扩展机制：
- 插件可注册自定义 UI 组件和页面
- 支持右键菜单扩展
- 插件权限管理面板
- 插件运行时沙箱隔离
- 支持插件安装、启用/禁用与删除
- 提供插件市场入口，支持下载失败提示和状态保留
## 主题系统
- 5 套内置主题：默认、午夜、海洋、玫瑰、日落
- 支持明暗模式切换
- 亚克力/毛玻璃效果（Windows）
- 基于 CSS Variables 的完整设计令牌体系，支持运行时切换
## 多语言支持
内置 10 种语言：
- 简体中文、繁体中文
- English、日本語、한국어
- Français、Deutsch、Español
- Русский、Tiếng Việt
- 支持运行时切换语言，无需重启应用
## Java 管理
- 自动检测系统中已安装的 Java 运行时
- 内置 Java 下载器，支持一键安装所需版本
- 支持手动指定 Java 路径并进行可用性校验
- 扩展常见安装目录扫描路径，降低"已安装但未识别"的情况
## Mod 管理
- 查看服务端目录中已安装的 Mod/插件文件
- 便于排查版本冲突、重复安装和目录问题
- 当前以本地管理为主，在线检索与一键安装能力仍在持续完善
## 服务器创建
- 引导式流程创建新服务器，降低新手配置门槛
- 支持 \`jar\` / \`bat\` / \`sh\` 启动模式
- 支持导入已有服务器或启动文件
- 智能检测开服方式，并支持自定义开服命令
- 可在创建流程中自动安装整合包
- 首次启动可引导完成 EULA 相关步骤
## 安全退出
- 关闭 Sea Lantern 时自动发送安全停止流程
- 尽量避免强制退出导致的存档损坏风险
## 自动更新
- 检查新版本并展示当前版本与最新版本信息
- 提供发布说明查看入口与下载跳转
- Arch Linux 使用 AUR 方式更新，其它平台走各自安装流程
## 系统托盘
- 最小化到系统托盘后可保持后台运行
- 支持从托盘快速恢复窗口或执行退出
## 使用建议
- 新服首次上线：先用「服务器创建」完成初始化，再用「图形化配置」调整关键参数
- 日常运维：优先通过「控制台」和「玩家管理」完成高频操作，减少手写命令
- 长时间挂服：配合「系统托盘」和「安全退出」降低误操作与强退风险
## 功能边界说明
- 不同服务端核心、不同操作系统的行为可能存在差异，建议按各页说明进行配置
- 自动更新以"检测版本 + 引导下载"为主，是否覆盖安装由你自行确认`,
  faq: `# 常见问题
## SeaLantern 无法启动
### 窗口不出现或空白，只有任务栏图标
可能为代理设置冲突。
请检查是否设置了 \`HTTP_PROXY\`、\`HTTPS_PROXY\`、\`ALL_PROXY\` 等环境变量。如有设置，请将其置空。
SeaLantern 基于 Tauri 实现，本质上是一个访问 localhost 的浏览器，因此涉及网络栈的系统配置（如代理服务器、防火墙设置）均可能影响 SeaLantern 的正常运行。
如果遇到其他类似的 SeaLantern 问题，例如 \`Connection Refused\`，请优先检查相关配置。
## 无法启动服务器
### 提示「Java 未找到」
Sea Lantern 需要 Java 来运行 Minecraft 服务端。解决方法：
1. 在 Sea Lantern 中使用内置 Java 下载器安装 Java
2. 或手动安装 Java 后重启 Sea Lantern，程序会自动检测
### 启动后立即崩溃
常见原因：
- **Java 版本不匹配**：检查你的 Java 版本是否符合要求（参考 [快速开始](${DOCS_BASE}/zh/getting-started#_3-配置-java)）
- **内存不足**：尝试增加分配的内存
- **JAR 文件损坏**：重新下载服务端 JAR
- **端口被占用**：检查 25565 端口是否被其他程序占用，或在 server.properties 中更换端口
### 提示「EULA 未同意」
首次启动服务器需要同意 Minecraft EULA。Sea Lantern 会弹出提示，点击同意即可。
## 控制台无响应
### 服务器似乎卡住了
- 检查控制台最后的日志输出，可能正在加载世界或插件
- 对于大型世界，首次加载可能需要较长时间
- 如果确实卡死，使用 Sea Lantern 的安全停止按钮关闭服务器，然后重新启动
### 命令没有反应
- 确认命令格式正确（控制台中不需要 \`/\` 前缀）
- 确认服务器已完全启动（控制台显示 \`Done!\`）
## 无法连接服务器
### 本地连接
在 Minecraft 中添加服务器，地址填写 \`localhost\` 或 \`127.0.0.1\`。
- 确认服务器已启动且控制台显示 \`Done!\`
- 确认客户端版本与服务端版本一致
- 如果修改过端口，地址需要加上端口号，如 \`localhost:25566\`
### 局域网连接
其他玩家使用你的**局域网 IP** 连接（如 \`192.168.1.100:25565\`）。
- 确保你和其他玩家在同一个局域网中
- 检查防火墙是否放行了服务器端口
### 外网连接
外网玩家需要你的**公网 IP** 或域名。
- 需要在路由器上配置端口转发（将外部 25565 转发到你的内网 IP）
- 或使用内网穿透工具（如 frp、ngrok）
- 检查防火墙设置
::: warning
将服务器暴露到公网时，建议：
- 启用 \`online-mode\`（正版验证）
- 启用白名单
- 定期备份存档
:::
## 旧版 Windows 运行方案
Sea Lantern 基于 Tauri 2，要求 Windows 10（版本 1909 及以上）或 Windows 11。**不支持** Windows 7、Windows 8、Windows 8.1。
对于仍在使用旧版系统的用户，可以通过安装 VxKex 扩展内核来补全所需运行库，使 Sea Lantern 在旧版 Windows 上运行。此方式同样适用于一些无法在老系统上正常运行的软件。
### 具体步骤
1. **获取 Sea Lantern 软件压缩包**
   - 前往 [下载页面](${DOCS_BASE}/zh/download) 下载 Windows 安装包
   - 安装完成后**先不要打开** Sea Lantern
2. **下载并安装 VxKex 扩展内核**
   - 此处提供的是国内的一个分支
   - VxKex-NEXT：[github.com/YuZhouRen86/VxKex-NEXT](https://github.com/YuZhouRen86/VxKex-NEXT)
   - 蓝奏云：[geekerz.lanzoue.com/b0ra5olfi](https://geekerz.lanzoue.com/b0ra5olfi)
   - 百度网盘：[pan.baidu.com/s/1w0JUk4JpoKiZQVi2LMoqwA?pwd=0000](https://pan.baidu.com/s/1w0JUk4JpoKiZQVi2LMoqwA?pwd=0000)（提取码：0000）
   - 下载完成后直接安装即可
3. **给 Sea Lantern 启用 VxKex**
   - 右键 Sea Lantern 主程序，选择「属性」，选择「VxKex」选项卡
   - 勾选「为此程序启用 VxKex NEXT」和「报告其他版本的 Windows」
   - 点击应用，确定
   - 此时 Sea Lantern 即可在 Windows 7 上正常运行
   - 除联机功能外，其他功能均正常
### 视频参考
如果不会操作请参考以下视频：[【让 Win7 强行兼容 Win10 应用？VxKex 扩展内核安装体验】](https://www.bilibili.com/video/BV1SedHYsE6b/)
::: danger 重要警告与免责声明
- 使用 VxKex 等非官方方法在 Windows 7/8/8.1 上运行 Sea Lantern **并不保证能正常工作**，可能会出现各种错误和兼容性问题
- **开发者不会接受任何通过此方法运行软件所产生的错误反馈，也不会对由此产生的问题进行修复**
- 使用此类方法可能存在系统安全风险，用户需自行承担相关责任
- 强烈建议升级到 Windows 10/11 以获得最佳体验和完整功能支持
- Sea Lantern 的开发和测试均基于现代 Windows 系统，对旧版系统的兼容性不在官方支持范围内
:::
## ArchLinux 运行方案
### ArchLinux 用户可以通过 AUR 安装 Sea Lantern，安装包会自动处理依赖关系，确保软件正常运行。
### Arch上的更新问题
由于 ArchLinux 的滚动更新机制，某些依赖库可能会发生重大更新，导致 Sea Lantern 无法正常运行。所以如果你的文件系统是BTRFS,建议在更新前使用快照功能备份系统，以便在出现问题时快速回滚到之前的状态。
1. **安装 Sea Lantern**
   - 使用 AUR 包管理器（如 paru）安装：
     \`\`\`bash
     paru -S sealantern
     \`\`\`
     ***注意这里不是 sea-lantern，而是 sealantern***
   - 安装完成后即可在应用菜单中找到 Sea Lantern 并启动，或者是使用命令行：
     \`\`\`bash
     sea-lantern
     \`\`\`
2. **更新 Sea Lantern**
   - AUR 包会随着官方版本更新而更新，使用 AUR 包管理器检查更新：
     \`\`\`bash
     paru -Syu sealantern
     \`\`\`
   - 如果更新后出现问题，可以回滚到之前的版本：
     \`\`\`bash
     paru -S sealantern-<previous_version>
     \`\`\`
3. **更新失败的解决步骤**
   - 如果更新后 Sea Lantern 无法启动，首先不要慌张，尝试回滚到之前的版本。
   - 之后可以在 GitHub 上查看是否有其他用户报告了类似的问题，或者是等待开发者发布修复版本。
   - 在此期间，建议不要删除旧版本的安装包，以便随时回滚。
   - 如果你有能力，可以尝试自己修复问题并提交 Pull Request，帮助社区更快地解决问题。
   如果是发现使用命令行进行更新的时候发现依旧是旧版本，可以尝试使用命令更新库：
   \`\`\`bash
   paru -Syu --devel sealantern
   \`\`\`
   如果还是不行可以尝试删除缓存：
   \`\`\`bash
   paru -Sc sealantern
   \`\`\`
## 其他常见问题
### Sea Lantern 需要什么权限？
Sea Lantern 需要访问文件系统、网络和执行程序的权限，这些是管理 Minecraft 服务器所必需的。
### Sea Lantern 是否安全？
Sea Lantern 是开源软件，代码完全透明。它不会连接到任何第三方服务器，所有数据都保存在本地。
### 支持哪些 Minecraft 版本？
Sea Lantern 支持所有标准的 Minecraft 服务器版本，只需使用对应版本的服务器 JAR 文件。
### 如何备份服务器？
服务器数据存放在服务器目录中，重点备份以下内容：
- \`world/\` 目录（主世界存档）
- \`world_nether/\` 和 \`world_the_end/\`（下界和末地）
- \`plugins/\` 目录（插件及其配置）
- \`server.properties\`（服务器配置）
::: tip
建议定期手动备份，或使用自动备份插件。
:::
### 服务器安全性
- 启用 \`online-mode=true\` 开启正版验证
- 使用白名单限制玩家加入
- 谨慎给予 OP 权限
- 定期更新服务端版本以修复安全漏洞
- 不要安装来源不明的插件
### 版本更新后不兼容
- 更新服务端前**务必备份**
- 确认所有插件都支持新版本
- 不建议跨多个大版本直接升级（如从 1.16 直接升到 1.21）
- 如果出现问题，可以回退到备份的版本`,
  contributor: `# 贡献者名单
感谢所有为 Sea Lantern 做出贡献的开发者。
详见 [GitHub Contributors](https://github.com/SeaLantern-Studio/SeaLantern/graphs/contributors).`,
};
