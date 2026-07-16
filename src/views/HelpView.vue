<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from "vue";
import { i18n } from "@language";
import { marked } from "marked";
import {
  Home,
  Download,
  Rocket,
  Box,
  BookOpen,
  Layers,
  HelpCircle,
  Users,
  ChevronRight,
  Menu,
  X,
  ExternalLink,
} from "lucide-vue-next";

// 文档页面配置
const docPages = [
  { id: "intro", title: "项目简介", icon: Home, content: "" },
  { id: "download", title: "下载安装", icon: Download, content: "" },
  { id: "getting-started", title: "快速开始", icon: Rocket, content: "" },
  { id: "server-jar", title: "核心获取", icon: Box, content: "" },
  { id: "tutorial", title: "使用教程", icon: BookOpen, content: "" },
  { id: "features", title: "功能总览", icon: Layers, content: "" },
  { id: "faq", title: "常见问题", icon: HelpCircle, content: "" },
  { id: "contributor", title: "贡献者", icon: Users, content: "" },
];

// 文档内容（Markdown）
const docContents: Record<string, string> = {
  intro: `
# Sea Lantern（海晶灯）

Sea Lantern（海晶灯）是一个**轻量化**的 Minecraft 服务器管理工具。

基于 **Tauri 2 + Rust + Vue 3** 构建。

## 特性

- **实时控制台** - 实时查看服务器日志，直接输入命令，支持命令历史
- **图形化配置** - 可视化编辑 server.properties，无需手动改文件
- **玩家管理** - 白名单、封禁、OP 权限管理
- **插件系统** - 基于 Lua 的插件扩展机制
- **创建流程 2.0** - 引导式创建服务器，支持整合包自动安装
- **主题系统** - 5 套内置主题，支持明暗模式
- **多语言支持** - 内置 10 种语言
- **Java 管理** - 自动检测与一键下载
- **Mod 管理** - 查看已安装的 Mod/插件
- **安全退出** - 自动发送安全停止命令
- **自动更新** - 检测新版本并引导下载
- **跨平台** - Windows / macOS / Linux

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

[GNU General Public License v3.0](https://github.com/SeaLantern-Studio/SeaLantern/blob/main/LICENSE)
`,
  download: `
# 下载安装

## 最新版本

当前最新版本：**v1.3.0**

按你的系统选择对应安装包下载并安装。

## Windows

| 格式 | 说明 |
| --- | --- |
| [exe 安装包](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_x64-setup.exe) | 推荐，双击安装 |
| [msi 安装包](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_x64_zh-CN.msi) | Windows Installer 格式 |
| [exe 安装包 (ARM64)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_arm64-setup.exe) | 适用于 Windows on ARM |

## macOS

| 格式 | 说明 |
| --- | --- |
| [dmg (Apple Silicon)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_aarch64.dmg) | M1 / M2 / M3 / M4 |
| [dmg (Intel)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_x64.dmg) | x64 架构 |
| [app.tar.gz (Apple Silicon)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_aarch64.app.tar.gz) | 便携压缩包 |
| [app.tar.gz (Intel)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_x64.app.tar.gz) | 便携压缩包 |

::: tip macOS 提示
当前 Release 中的 dmg 与 app.tar.gz 均未进行 Apple 签名/公证，macOS 可能提示"已损坏，无法打开"或"无法验证开发者"。

在终端执行以下命令解除限制：

\`\`\`bash
xattr -dr com.apple.quarantine /Applications/Sea\\ Lantern.app
\`\`\`
:::

## Linux

| 格式 | 说明 |
| --- | --- |
| [deb](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_amd64.deb) | Debian / Ubuntu |
| [deb (ARM64)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_arm64.deb) | Debian / Ubuntu ARM64 |
| [rpm](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern-1.3.0-1.x86_64.rpm) | Fedora / RHEL (x86_64) |
| [rpm (ARM64)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern-1.3.0-1.aarch64.rpm) | Fedora / RHEL (aarch64) |
| [AppImage](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_amd64.AppImage) | 通用格式 |
| [AppImage (ARM64)](https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0/Sea.Lantern_1.3.0_aarch64.AppImage) | 通用格式 (ARM64) |

Arch Linux 用户可通过 AUR 安装：

\`\`\`bash
paru -S sealantern
\`\`\`

## Docker

\`\`\`bash
docker pull penetr4t10n/sealantern:latest
docker run -d --name sealantern -p 3000:3000 -p 25565:25565/tcp penetr4t10n/sealantern:latest
\`\`\`

::: warning Docker 注意事项
docker 内的网络环境是隔离的，因此需要暴露 web 端口和服务器端口
:::

## 系统要求

- Windows 10+ / macOS 10.15+ / Linux (glibc 2.31+)
- Java 8 及以上（用于运行 Minecraft 服务端）
`,
  "getting-started": `
# 快速开始

## 系统要求

| 项目 | 最低要求 |
| --- | --- |
| 操作系统 | Windows 10+、macOS 10.15+、Linux (glibc 2.31+) |
| Java | Java 8+（用于运行 Minecraft 服务端） |
| 内存 | 建议 4GB 以上（Minecraft 服务端需要额外内存） |

::: info Windows 版本说明
Sea Lantern 使用 WebView2 运行时，**要求 Windows 10（版本 1909 及以上）或 Windows 11**。
:::

## 1. 下载安装

前往 [下载页面](https://docs.ideaflash.cn/zh/download) 获取适合你操作系统的安装包，双击运行即可完成安装。

## 2. 获取服务端核心

你可以使用 Minecraft 服务端 JAR 创建新服务器，也可以直接导入已有服务器目录或启动脚本。

::: tip 推荐
对于大多数玩家，推荐使用 [Paper](https://papermc.io/downloads/paper) — 性能优秀、插件生态丰富。
:::

## 3. 配置 Java

启动 Sea Lantern 后，软件会**自动检测**系统中已安装的 Java。如果没有合适的版本，可以使用内置的 Java 下载器一键安装。

Minecraft 不同版本对 Java 的要求不同：

| Minecraft 版本 | 推荐 Java |
| --- | --- |
| 1.16.5 及以下 | Java 8 / 11 |
| 1.17 ~ 1.20.4 | Java 17 |
| 1.20.5+ | Java 21 |

## 4. 创建服务器

1. 点击「创建服务器」按钮
2. 选择导入来源（JAR、已有服务器或启动脚本）
3. 按向导完成启动方式检测；如有需要可自定义开服命令
4. 选择 Java 运行时版本（缺失时可一键安装）
5. 为你的服务器命名并确认参数
6. 首次启动时需要同意 Minecraft EULA

## 5. 启动服务器

点击启动按钮，等待控制台显示 \`Done!\` 即表示服务器启动成功。此时你可以在 Minecraft 中通过 \`localhost\` 连接到你的服务器。
`,
  "server-jar": `
# 核心获取

Minecraft 服务器需要一个服务端核心（JAR 文件）才能运行。本页介绍常见的服务端类型及其获取方式。

## 原版服务端（Vanilla）

Mojang 官方提供的原版服务端，不支持任何插件或 Mod。

- 下载地址：[Minecraft 官网](https://www.minecraft.net/zh-hans/download/server)
- 适用场景：纯原版体验，不需要任何扩展

::: warning 性能提示
原版服务端性能较低，建议仅用于少人数本地联机。
:::

## Paper（推荐）

基于 Spigot 的高性能服务端，是目前最流行的选择。

- 下载地址：[papermc.io](https://papermc.io/downloads/paper)
- 适用场景：需要插件、追求性能的服务器
- 插件兼容：支持 Bukkit / Spigot / Paper 插件

**为什么推荐 Paper？**

- 性能远超原版和 Spigot，对红石和生物 AI 有优化
- 社区活跃，插件兼容性好
- 自带反作弊和安全修补
- 更新速度快，新版本支持及时

## Spigot

老牌的插件服务端，Paper 的上游项目。

- 获取方式：通过 [BuildTools](https://www.spigotmc.org/wiki/buildtools/) 自行编译
- 适用场景：需要 Spigot 专属插件，或与 Paper 有兼容性问题时

::: info 提示
Spigot 不提供直接下载的 JAR 文件，需要使用 BuildTools 编译。对新手不太友好，建议优先选择 Paper。
:::

## Forge

老牌 Mod 服务端，用于运行 Java 版的 Forge 端 Mod。

- 下载地址：[Forge 官网](https://files.minecraftforge.net/)
- 适用场景：需要运行 Forge Mod 的模组服务器
- 注意：**不兼容 Bukkit/Spigot 插件**

## Fabric

轻量级 Mod 加载器，启动速度快。

- 下载地址：[Fabric 官网](https://fabricmc.net/use/installer/)
- 适用场景：需要运行 Fabric Mod 的模组服务器
- 注意：**不兼容 Bukkit/Spigot 插件**

## Purpur

基于 Paper 的分支，提供更多配置选项。

- 下载地址：[Purpur 官网](https://purpurmc.org/downloads)
- 适用场景：需要更多自定义配置选项的服务器
- 插件兼容：支持 Bukkit / Spigot / Paper 插件

## 如何选择？

| 类型 | 插件支持 | Mod 支持 | 性能 | 推荐度 |
| --- | --- | --- | --- | --- |
| Paper | ✅ Bukkit/Spigot/Paper | ❌ | ⭐⭐⭐ | ⭐⭐⭐ |
| Purpur | ✅ Bukkit/Spigot/Paper | ❌ | ⭐⭐⭐ | ⭐⭐ |
| Spigot | ✅ Bukkit/Spigot | ❌ | ⭐⭐ | ⭐⭐ |
| Vanilla | ❌ | ❌ | ⭐ | ⭐ |
| Forge | ❌ | ✅ Forge Mod | ⭐⭐ | ⭐⭐ |
| Fabric | ❌ | ✅ Fabric Mod | ⭐⭐⭐ | ⭐⭐ |

## 注意事项

### 版本兼容性

服务端版本必须与客户端版本一致，否则玩家无法加入。

### Minecraft EULA

首次启动任何 Minecraft 服务端时，你需要同意 [Minecraft EULA](https://www.minecraft.net/eula)。Sea Lantern 会在创建服务器时引导你完成这一步。
`,
  tutorial: `
# 使用教程

本页详细介绍 Sea Lantern 的各项功能使用方法。

## 创建第一个服务器

### 导入来源（JAR / 已有服务器 / 启动脚本）

1. 点击主界面的「创建服务器」按钮
2. 在引导界面选择导入来源：服务端 JAR、已有服务器目录或启动脚本（\`bat\` / \`sh\`）
3. 如果还没有服务端，请参考 [核心获取](/zh/server-jar)

### 选择 Java 版本

| Minecraft 版本 | 推荐 Java |
| --- | --- |
| 1.16.5 及以下 | Java 8 / 11 |
| 1.17 ~ 1.20.4 | Java 17 |
| 1.20.5+ | Java 21 |

::: tip 提示
如果列表中没有合适的 Java，点击「下载 Java」按钮，Sea Lantern 会自动下载并安装所需版本。
:::

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

## server.properties 配置

Sea Lantern 提供了图形化的 server.properties 编辑器，让你无需手动修改文件。

### 常用配置项

| 配置项 | 说明 | 默认值 |
| --- | --- | --- |
| \`server-port\` | 服务器端口 | 25565 |
| \`max-players\` | 最大玩家数 | 20 |
| \`difficulty\` | 游戏难度 | easy |
| \`gamemode\` | 默认游戏模式 | survival |
| \`motd\` | 服务器简介（显示在服务器列表） | A Minecraft Server |
| \`online-mode\` | 正版验证 | true |
| \`pvp\` | 是否允许 PVP | true |
| \`white-list\` | 是否启用白名单 | false |

::: warning 注意
修改配置后需要**重启服务器**才能生效。
:::

## 玩家管理

### 白名单

启用白名单后，只有白名单中的玩家才能加入服务器。

### 封禁管理

- 封禁玩家：在玩家列表中点击封禁，或执行 \`ban 玩家名\`
- 解除封禁：在封禁列表中点击解封，或执行 \`pardon 玩家名\`

### OP 权限

OP（管理员）拥有服务器的最高权限。谨慎给予 OP 权限。

## 高级设置

### 内存分配

| 玩家数 | 推荐内存 |
| --- | --- |
| 1-5 人 | 2-4 GB |
| 5-15 人 | 4-6 GB |
| 15-30 人 | 6-8 GB |
| 30+ 人 | 8 GB+ |

### 安全退出

当你关闭 Sea Lantern 时，程序会自动向服务器发送 \`stop\` 命令，等待服务器安全关闭后再退出。这可以防止存档损坏。

::: danger 危险操作
**不要** 直接通过任务管理器强制关闭正在运行的服务器，这可能导致存档丢失或损坏。
:::
`,
  features: `
# 功能总览

本页用于快速了解 Sea Lantern 现阶段可用能力与适用场景。

::: info 提示
如果你是首次使用，建议先看「服务器创建」「Java 管理」「控制台」这三部分。
:::

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
`,
  faq: `
# 常见问题

## SeaLantern 无法启动

### 窗口不出现或空白，只有任务栏图标

可能为代理设置冲突。

请检查是否设置了 \`HTTP_PROXY\`、\`HTTPS_PROXY\`、\`ALL_PROXY\` 等环境变量。如有设置，请将其置空。

::: info 说明
SeaLantern 基于 Tauri 实现，本质上是一个访问 localhost 的浏览器，因此涉及网络栈的系统配置（如代理服务器、防火墙设置）均可能影响 SeaLantern 的正常运行。
:::

## 无法启动服务器

### 提示「Java 未找到」

Sea Lantern 需要 Java 来运行 Minecraft 服务端。解决方法：

1. 在 Sea Lantern 中使用内置 Java 下载器安装 Java
2. 或手动安装 Java 后重启 Sea Lantern，程序会自动检测

### 启动后立即崩溃

常见原因：

- **Java 版本不匹配**：检查你的 Java 版本是否符合要求
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

::: warning 安全提示
将服务器暴露到公网时，建议：

- 启用 \`online-mode\`（正版验证）
- 启用白名单
- 定期备份存档
:::

## 旧版 Windows 运行方案

Sea Lantern 基于 Tauri 2，要求 Windows 10（版本 1909 及以上）或 Windows 11。**不支持** Windows 7、Windows 8、Windows 8.1。

对于仍在使用旧版系统的用户，可以通过安装 VxKex 扩展内核来补全所需运行库。

::: danger 重要警告
- 使用 VxKex 等非官方方法在 Windows 7/8/8.1 上运行 Sea Lantern **并不保证能正常工作**
- **开发者不会接受任何通过此方法运行软件所产生的错误反馈**
- 使用此类方法可能存在系统安全风险，用户需自行承担相关责任
- 强烈建议升级到 Windows 10/11 以获得最佳体验
:::

## 其他常见问题

### 如何备份服务器？

重点备份以下内容：

- \`world/\` 目录（主世界存档）
- \`world_nether/\` 和 \`world_the_end/\`
- \`plugins/\` 目录
- \`server.properties\`

### 服务器安全性建议

- 启用 \`online-mode=true\` 开启正版验证
- 使用白名单限制玩家加入
- 谨慎给予 OP 权限
- 定期更新服务端版本
- 不要安装来源不明的插件
`,
  contributor: `
# 贡献者名单

感谢所有为 Sea Lantern 做出贡献的开发者。

## 核心贡献者

| 贡献者 | 贡献次数 |
| --- | --- |
| [FPSZ](https://github.com/FPSZ) | 94 次 |
| [xingwangzhe](https://github.com/xingwangzhe) | 79 次 |
| [CmzYa](https://github.com/CmzYa) | 65 次 |
| [KercyDing](https://github.com/KercyDing) | 36 次 |
| [Sjshi763](https://github.com/Sjshi763) | 18 次 |
| [zhuxiaojt](https://github.com/zhuxiaojt) | 17 次 |
| [Nanaloveyuki](https://github.com/Nanaloveyuki) | 13 次 |
| [LingyeNBird](https://github.com/LingyeNBird) | 13 次 |

## 参与贡献

- **写代码** — 提交 PR，修 Bug 或加新功能
- **做设计** — 设计 UI、图标、主题皮肤
- **提建议** — 在 Issues 里提出你的想法
- **写文档** — 完善教程和使用说明
- **翻译** — 帮助翻译成其他语言
- **推广** — 分享给更多 MC 服主

详见 [GitHub Contributors](https://github.com/SeaLantern-Studio/SeaLantern/graphs/contributors)。
`,
};

const currentSection = ref("intro");
const sidebarOpen = ref(true);
const contentRef = ref<HTMLElement | null>(null);
const isMobile = ref(false);

// 配置 marked
marked.setOptions({
  breaks: true,
  gfm: true,
});

// 计算当前内容
const currentContent = computed(() => {
  return docContents[currentSection.value] || "";
});

// 计算当前标题
const currentTitle = computed(() => {
  const page = docPages.find((p) => p.id === currentSection.value);
  return page?.title || "帮助文档";
});

// 渲染 Markdown
const renderedContent = computed(() => {
  return marked.parse(currentContent.value) as string;
});

// 切换页面
function switchPage(id: string) {
  if (currentSection.value === id) return;
  currentSection.value = id;
  if (isMobile.value) {
    sidebarOpen.value = false;
  }
  nextTick(() => {
    contentRef.value?.scrollTo(0, 0);
  });
}

// 在浏览器打开
function openInBrowser() {
  window.open("https://docs.ideaflash.cn/zh/intro", "_blank");
}

// 检测移动端
function checkMobile() {
  isMobile.value = window.innerWidth < 768;
  if (!isMobile.value) {
    sidebarOpen.value = true;
  }
}

onMounted(() => {
  checkMobile();
  window.addEventListener("resize", checkMobile);
});

onUnmounted(() => {
  window.removeEventListener("resize", checkMobile);
});
</script>

<template>
  <div class="help-view">
    <!-- 移动端侧栏切换按钮 -->
    <button v-if="isMobile" class="sidebar-toggle" @click="sidebarOpen = !sidebarOpen">
      <Menu v-if="!sidebarOpen" :size="20" />
      <X v-else :size="20" />
    </button>

    <!-- 侧栏导航 -->
    <aside class="help-sidebar" :class="{ open: sidebarOpen }">
      <div class="sidebar-header">
        <BookOpen class="sidebar-icon" :size="18" />
        <span class="sidebar-title">{{ i18n.t("help.title") }}</span>
      </div>
      <nav class="sidebar-nav">
        <div
          v-for="page in docPages"
          :key="page.id"
          class="nav-item"
          :class="{ active: currentSection === page.id }"
          @click="switchPage(page.id)"
        >
          <component :is="page.icon" class="nav-icon" :size="16" :stroke-width="1.8" />
          <span class="nav-label">{{ page.title }}</span>
          <ChevronRight v-if="currentSection === page.id" class="nav-arrow" :size="14" />
        </div>
      </nav>

      <!-- 在浏览器打开按钮 -->
      <div class="sidebar-footer">
        <button class="open-browser-btn" @click="openInBrowser">
          <ExternalLink :size="16" />
          <span>在浏览器打开</span>
        </button>
      </div>
    </aside>

    <!-- 内容区域 -->
    <main class="help-content" ref="contentRef">
      <article class="help-article">
        <div class="article-body" v-html="renderedContent"></div>
      </article>
    </main>
  </div>
</template>

<style src="@styles/views/HelpView.css" scoped></style>
