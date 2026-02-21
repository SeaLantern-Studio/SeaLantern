# Sea Lantern - AI 代理上下文指南

## 项目概述

**Sea Lantern（海晶灯）** 是一个轻量化的 Minecraft 服务器管理工具，基于 **Tauri 2 + Rust + Vue 3** 技术栈构建。

### 核心功能
- 控制台实时查看日志，直接输入命令
- `server.properties` 图形化编辑
- 白名单、封禁、OP 一键管理
- 关闭软件时自动停止服务器，防止丢存档
- 检查更新，一键下载新版本

### 技术栈
- **前端**: Vue 3 + TypeScript + Vite + Pinia
- **后端**: Rust + Tauri 2
- **样式**: 纯 CSS（CSS 变量驱动）
- **通信**: Tauri invoke（前端直接调用 Rust 函数）

---

## 项目结构

```
sea-lantern/
├── src/                          # 前端代码（Vue 3 + TypeScript）
│   ├── api/                      # 与 Rust 后端通信的封装层
│   │   ├── tauri.ts              # 基础 invoke 封装
│   │   ├── server.ts             # 服务器管理 API
│   │   ├── java.ts               # Java 环境检测 API
│   │   ├── config.ts             # 配置文件读写 API
│   │   ├── player.ts             # 玩家管理 API
│   │   ├── settings.ts           # 应用设置 API
│   │   └── ...
│   ├── components/               # UI 组件
│   │   ├── common/               # 通用组件（SLButton, SLInput, SLModal 等）
│   │   ├── config/               # 配置相关组件
│   │   ├── console/              # 控制台相关组件
│   │   ├── layout/               # 页面布局组件
│   │   └── splash/               # 启动画面
│   ├── composables/              # 组合式函数
│   ├── language/                 # 国际化资源（10+ 语言）
│   ├── router/                   # 路由配置
│   ├── stores/                   # Pinia 状态管理
│   ├── styles/                   # 全局样式（CSS 变量、动画、毛玻璃）
│   ├── types/                    # TypeScript 类型定义
│   ├── utils/                    # 工具函数
│   ├── views/                    # 页面视图
│   ├── App.vue                   # 根组件
│   └── main.ts                   # 应用入口
│
├── src-tauri/                    # 后端代码（Rust + Tauri 2）
│   ├── src/
│   │   ├── commands/             # Tauri 命令（前端可调用的 API）
│   │   ├── services/             # 业务逻辑层
│   │   ├── models/               # 数据结构定义
│   │   └── utils/                # 工具函数
│   ├── icons/                    # 应用图标
│   └── tauri.conf.json           # Tauri 配置
│
└── docs/                         # 项目文档
```

---

## 常用命令

### 开发环境

```bash
# 安装依赖
npm install

# 启动开发服务器（需要 Rust 1.70+ 和 Node.js 20+）
npm run tauri dev
```

### 构建发布版

```bash
# 构建生产版本
npm run tauri build

# 产物位置
src-tauri/target/release/bundle/
```

### 代码质量检查

**前端检查：**
```bash
# 代码质量检查
npm run lint

# 自动修复可修复问题
npm run lint:fix

# 格式化代码
npm run fmt

# 检查代码格式
npm run fmt:check
```

**后端检查：**
```bash
# 检查代码格式
cargo fmt --all -- --check

# 运行 Clippy 检查
cargo clippy --workspace -- -D warnings

# 格式化代码
cargo fmt --all
```

---

## 开发规范

### 前端规范

1. **Vue 组件**
   - 组件名使用 `PascalCase`（如 `ServerCard.vue`）
   - 使用 `<script setup>` 语法
   - Props 和 emits 必须定义类型

2. **TypeScript**
   - 启用严格模式
   - 避免使用 `any`，优先使用具体类型
   - 接口名使用 `PascalCase`

3. **样式**
   - 使用 CSS 变量（`var(--sl-*)`）
   - 避免硬编码颜色值
   - 使用 scoped 样式

4. **UI 组件与图标**
   - 优先使用 Headless UI（Vue v1）提供无样式、可访问性的交互组件
   - 图标使用 Lucide Vue Next（`lucide-vue-next`），按需导入
   - 不要硬编码 SVG 路径

### 后端规范

1. **命名规范**
   - 文件名：使用 `snake_case`（如 `server_manager.rs`）
   - 函数名：使用 `snake_case`（如 `get_server_list`）
   - 结构体：使用 `PascalCase`（如 `ServerInstance`）
   - 常量：使用 `SCREAMING_SNAKE_CASE`（如 `MAX_MEMORY`）

2. **错误处理**
   - 使用 `Result<T, String>` 返回错误
   - 错误信息要清晰、用户友好
   - 避免使用 `unwrap()`，优先使用 `?` 或 `unwrap_or`

3. **注释规范**
   - 公共 API 必须有文档注释（`///`）
   - 复杂逻辑需要添加行内注释（`//`）

### Git 工作流

**分支命名：**
- `feature/功能名` - 新功能
- `fix/问题描述` - Bug 修复
- `chore/任务描述` - 杂项任务
- `docs/文档说明` - 文档更新

**Commit 规范（约定式提交）：**
```
<类型>: <简短描述>

<详细描述>（可选）
```

**类型：**
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建/工具链相关

---

## 添加新功能指南

### 后端开发流程

1. 在 `src-tauri/src/services/` 下创建新的服务文件（业务逻辑）
2. 在 `src-tauri/src/commands/` 下创建新的命令文件（Tauri 命令）
3. 在 `commands/mod.rs` 中添加 `pub mod 模块名`
4. 在 `lib.rs` 的 `generate_handler!` 宏中注册命令

### 前端开发流程

1. 在 `src/api/` 下创建新的 API 封装文件（调用 Rust 命令）
2. 在 `src/views/` 下创建新的页面组件
3. 在 `src/router/index.ts` 中添加路由
4. 在 `src/components/` 下创建相关组件（如需要）
5. 在 `src/stores/` 下创建相关状态管理（如需要）

---

## 国际化（i18n）

项目支持多语言国际化，语言文件位于 `src/language/` 目录：

- `zh-CN.json` - 简体中文
- `zh-TW.json` - 繁体中文
- `en-US.json` - 英语
- `ja-JP.json` - 日语
- `ko-KR.json` - 韩语
- `de-DE.json` - 德语
- `es-ES.json` - 西班牙语
- `ru-RU.json` - 俄语
- `vi-VN.json` - 越南语

添加新翻译时，需要同时更新所有语言文件。

---

## 主题系统

项目使用 CSS 变量驱动主题，支持：
- `default` - 默认主题
- `midnight` - 午夜主题
- `ocean` - 海洋主题
- `rose` - 玫瑰主题
- `sunset` - 日落主题

主题文件位于 `src/themes/` 目录。

---

## 注意事项

1. **数据存储**：服务器数据存储在可执行文件所在目录的 `sea_lantern_servers.json` 文件中
2. **日志管理**：服务器日志存储在服务器目录的 `latest.log` 文件中
3. **权限管理**：确保应用具有足够的文件系统权限
4. **性能优化**：避免在主线程中执行耗时操作
5. **错误处理**：确保所有错误都有适当的处理和提示

---

## 待开发功能

以下功能已预留位置，等待实现：

- **下载中心** - 下载服务端核心、插件、模组
- **备份管理** - 世界存档的增量备份和还原
- **内网穿透** - 集成 FRP
- **定时任务** - 自动重启、定时备份、定时执行命令
- **资源管理** - 从 Modrinth 和 CurseForge 搜索安装插件和模组

---

## 相关文档

- [项目结构详解](docs/STRUCTURE.md)
- [贡献指南](docs/CONTRIBUTING.md)
- [i18n 国际化指南](src/language/README.md)
- [AI 使用指南](docs/AI_GUIDE.md)

---

## 许可证

[GNU General Public License v3.0](LICENSE)

---

## PPA 测试流程（ppa-deploy 分支）

### 概述

`ppa-deploy` 分支专门用于构建和发布 Sea Lantern 到 Ubuntu PPA。PPA 地址：`ppa:brianeee7878/sealantern`

### 工作流程

1. **构建流程** (`.github/workflows/build-and-upload-ppa.yml`)
   - 从 GitHub Releases 下载 Sea Lantern deb 包
   - 创建嵌套的 PPA 安装包 (`sea-lantern-ppa-updater`)
   - 构建源代码包并签名
   - 上传到 Launchpad PPA

2. **测试流程** (`.github/workflows/ppa-test.yml`)
   - 定期测试 PPA 包安装
   - 验证菜单项和命令是否正确配置
   - 监控 PPA 构建状态

### 本地测试

```bash
# 运行本地安装测试
./test-ppa-install.sh

# 手动构建 PPA 包（单版本）
./create-ppa-deb.sh

# 手动构建 PPA 包（多版本）
./create-ppa-deb-multi.sh
```

### gh CLI 自动测试命令

```bash
# 手动触发 PPA 测试工作流
gh workflow run ppa-test.yml -f version=0.6.5 -f distro=noble

# 查看工作流运行状态
gh run list --workflow=ppa-test.yml

# 查看最近运行的日志
gh run view $(gh run list --workflow=ppa-test.yml -L 1 --json databaseId -q '.[0].databaseId') --log

# 触发 PPA 构建和上传
gh workflow run build-and-upload-ppa.yml -f version=0.6.5
```

### Launchpad API 查询

```bash
# 获取 PPA 基本信息
curl -s "https://api.launchpad.net/1.0/~brianeee7878/+archive/ubuntu/sealantern"

# 获取已发布的源码包
curl -s "https://api.launchpad.net/1.0/~brianeee7878/+archive/ubuntu/sealantern?ws.op=getPublishedSources"

# 获取构建记录
curl -s "https://api.launchpad.net/1.0/~brianeee7878/+archive/ubuntu/sealantern?ws.op=getBuildRecords"

# 获取已发布的二进制包
curl -s "https://api.launchpad.net/1.0/~brianeee7878/+archive/ubuntu/sealantern?ws.op=getPublishedBinaries"
```

### 已知问题和解决方案

1. **postinst 脚本版本号获取问题**
   - 问题：`dpkg -s` 在 postinst 执行时可能无法正确获取版本
   - 解决：使用 `dpkg-query -W -f='${Version}'` 替代

2. **桌面文件 Exec 路径**
   - 原始 deb 包中的 Exec 为 `sea-lantern`（相对路径）
   - 实际安装后命令位于 `/usr/bin/sea-lantern`
   - 已在系统中创建符号链接

3. **图标文件位置**
   - 图标安装到 `/usr/share/icons/hicolor/*/apps/sealantern.png`
   - 需要运行 `gtk-update-icon-cache` 更新图标缓存

### 测试检查清单

- [ ] deb 包可以正常安装
- [ ] postinst 脚本正确执行
- [ ] `/usr/bin/sea-lantern` 命令可用
- [ ] 桌面文件 `/usr/share/applications/Sea Lantern.desktop` 存在
- [ ] 图标文件已安装
- [ ] 嵌入的 deb 包已被清理
- [ ] 卸载时文件被正确删除

---

*本文档由 AI 代理自动生成，用于提供项目上下文信息。*
*最后更新：2026-02-21*
