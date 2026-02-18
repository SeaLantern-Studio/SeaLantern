# AI 开发指南 - 海晶灯项目

> 本文档专为 AI 助手设计，帮助 AI 快速理解项目结构、代码规范和常见修改场景。不要用emoji表情，请使用用户发送的语言回复用户

## 项目概览

**项目名称**: 海晶灯 (Sea Lantern)
**项目类型**: Minecraft 服务器管理工具
**技术栈**: Tauri 2 + Rust + Vue 3 + TypeScript + Pinia
**当前版本**: 0.6.2
**Gitee仓库**: https://gitee.com/fps_z/SeaLantern (master 分支)
**Github仓库**: https://github.com/FPSZ/SeaLantern (main 分支)

### 核心特点

- 前端使用 Vue 3 Composition API + TypeScript
- 后端使用 Rust，通过 Tauri invoke 与前端通信
- 无 Electron，无 Node 后端，体积小，性能好
- 毛玻璃 UI 风格，纯 CSS 实现

---

## 关键文件速查表

### 版本号相关（修改版本时必须同步更新）

**核心版本文件**（必须修改）：
- `package.json` - 前端版本号，格式：`"version": "x.x.x"`
- `src-tauri/Cargo.toml` - Rust 后端版本号，格式：`version = "x.x.x"`
- `src-tauri/tauri.conf.json` - Tauri 配置版本号，格式：`"version": "x.x.x"`

**Arch Linux 打包文件**（如果发布到 AUR 需要修改）：
- `PKGBUILD` - Arch 打包脚本，格式：`pkgver=x.x.x`
- `.SRCINFO` - AUR 元数据文件，格式：`pkgver = x.x.x`

**自动生成文件**（运行命令后自动更新）：
- `Cargo.lock` - 运行 `cargo update -p sea-lantern` 自动更新
- `package-lock.json` - 运行 `npm install` 自动更新

**版本号读取方式**：
- 前端通过 `@tauri-apps/api/app` 的 `getVersion()` 从 Tauri 配置读取
- 显示在关于页面（AboutView.vue）

### 配置文件

- `vite.config.ts` - Vite 构建配置，开发服务器端口 5173
- `tsconfig.json` - TypeScript 编译配置
- `src-tauri/tauri.conf.json` - Tauri 应用配置（窗口大小、标题、权限等）
- `src-tauri/capabilities/default.json` - Tauri 权限配置

### 入口文件

- `src/main.ts` - 前端入口，初始化 Vue、Pinia、Router
- `src-tauri/src/main.rs` - Rust 应用入口
- `src-tauri/src/lib.rs` - Tauri 库入口，注册命令和插件

---

## 架构设计

### 前后端通信流程

```
前端 Vue 组件
    ↓ 调用
src/api/*.ts (封装层)
    ↓ invoke
src-tauri/src/commands/*.rs (命令层)
    ↓ 调用
src-tauri/src/services/*.rs (业务逻辑层)
    ↓ 返回
src-tauri/src/models/*.rs (数据结构)
```

**示例**：检查软件更新

1. 前端: `AboutView.vue` 调用 `checkUpdate()`
2. API 层: `src/api/update.ts` 的 `checkUpdate()` 调用 `tauriInvoke('check_update', ...)`
3. 命令层: `src-tauri/src/commands/update.rs` 的 `#[command] check_update()` 处理请求
4. 返回: 返回 `UpdateInfo` 结构体给前端

### 目录结构详解

#### 前端 (`src/`)

**api/** - 与 Rust 后端通信的封装层

- `tauri.ts` - 基础 invoke 封装，导出 `tauriInvoke` 函数
- `server.ts` - 服务器管理 API（创建、启动、停止、日志）
- `java.ts` - Java 环境检测 API
- `config.ts` - 配置文件读写 API
- `player.ts` - 玩家管理 API（白名单、封禁、OP）
- `settings.ts` - 应用设置 API
- `system.ts` - 系统信息、文件对话框 API
- `update.ts` - 软件更新检查 API

**components/** - UI 组件

- `common/` - 通用组件（SLButton、SLCard、SLInput、SLSelect、SLSwitch、SLModal、SLProgress、SLBadge）
- `layout/` - 布局组件（AppLayout、AppSidebar、AppHeader）
- `splash/` - 启动画面（SplashScreen）

**views/** - 页面视图

- `HomeView.vue` - 首页（服务器列表）
- `CreateServerView.vue` - 创建/导入服务器
- `ConsoleView.vue` - 控制台（实时日志、命令输入）
- `ConfigView.vue` - 配置编辑（server.properties）
- `PlayerView.vue` - 玩家管理（白名单、封禁、OP）
- `SettingsView.vue` - 应用设置
- `AboutView.vue` - 关于页面（贡献者墙、更新检查）

**stores/** - Pinia 状态管理

- `serverStore.ts` - 服务器列表和运行状态
- `consoleStore.ts` - 控制台日志（切换页面不丢失）
- `uiStore.ts` - 界面状态（侧栏折叠等）

**styles/** - 全局样式

- `variables.css` - CSS 变量（颜色、间距、圆角、字体、阴影）
- `reset.css` - 浏览器样式重置
- `typography.css` - 排版样式
- `animations.css` - 动画关键帧
- `glass.css` - 毛玻璃效果样式

#### 后端 (`src-tauri/src/`)

**commands/** - Tauri 命令（前端 invoke 调用的 API）

- `server.rs` - 服务器管理命令
- `java.rs` - Java 检测命令
- `config.rs` - 配置文件读写命令
- `player.rs` - 玩家管理命令
- `settings.rs` - 应用设置命令
- `system.rs` - 系统信息、文件对话框命令
- `update.rs` - 软件更新检查命令

**services/** - 业务逻辑层

- `server_manager.rs` - 服务器进程管理、日志读取
- `java_detector.rs` - Java 环境扫描器
- `config_parser.rs` - .properties 文件解析器
- `player_manager.rs` - 玩家数据文件读取
- `settings_manager.rs` - 应用设置持久化
- `global.rs` - 全局单例管理器

**models/** - 数据结构定义

- `server.rs` - 服务器实例、状态数据结构
- `config.rs` - 配置项数据结构
- `settings.rs` - 应用设置数据结构
- `dev_config.rs` - 开发者配置数据结构

---

## 代码规范和约定

### 前端规范

#### 1. API 调用规范

```typescript
// src/api/example.ts
import { tauriInvoke } from "./tauri";

export async function someFunction(param: string): Promise<ReturnType> {
  return tauriInvoke<ReturnType>("command_name", { param });
}
```

#### 2. Vue 组件规范

- 使用 Composition API (`<script setup lang="ts">`)
- 使用 TypeScript
- Props 使用 `defineProps<{ ... }>()`
- Emits 使用 `defineEmits<{ ... }>()`

#### 3. 样式规范

- 使用 scoped CSS
- 使用 CSS 变量（定义在 `src/styles/variables.css`）
- 毛玻璃效果使用 `backdrop-filter: blur(10px)`

#### 4. 路由规范

路由定义在 `src/router/index.ts`，格式：

```typescript
{
  path: '/path',
  name: 'RouteName',
  component: () => import('../views/ViewName.vue')
}
```

添加新页面后，需要在 `src/components/layout/AppSidebar.vue` 的 `navItems` 数组中添加导航项。

### 后端规范

#### 1. 命令定义规范

```rust
// src-tauri/src/commands/example.rs
use tauri::command;

#[command]
pub async fn command_name(param: String) -> Result<ReturnType, String> {
    // 实现逻辑
    Ok(result)
}
```

#### 2. 命令注册流程

1. 在 `src-tauri/src/commands/mod.rs` 中添加 `pub mod example;`
2. 在 `src-tauri/src/lib.rs` 的 `generate_handler!` 宏中添加命令名

#### 3. 错误处理规范

- 命令返回 `Result<T, String>`
- 使用 `.map_err(|e| e.to_string())` 转换错误

#### 4. 异步规范

- 需要异步操作的命令使用 `async fn`
- 使用 `tokio` 运行时

---

## 常见修改场景

### 场景 1: 添加新的 Tauri 命令

**步骤**：

1. 在 `src-tauri/src/commands/` 创建或修改对应的 `.rs` 文件
2. 定义命令函数，添加 `#[command]` 宏
3. 在 `src-tauri/src/commands/mod.rs` 中导出模块
4. 在 `src-tauri/src/lib.rs` 的 `generate_handler!` 中注册命令
5. 在 `src/api/` 创建或修改对应的 `.ts` 文件，封装 invoke 调用
6. 在 Vue 组件中调用 API 函数

**示例**：添加 `get_system_info` 命令

```rust
// src-tauri/src/commands/system.rs
#[command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    // 实现
}
```

```typescript
// src/api/system.ts
export async function getSystemInfo(): Promise<SystemInfo> {
  return tauriInvoke<SystemInfo>("get_system_info");
}
```

### 场景 2: 添加新页面

**步骤**：

1. 在 `src/views/` 创建新的 `.vue` 文件
2. 在 `src/router/index.ts` 添加路由
3. 在 `src/components/layout/AppSidebar.vue` 的 `navItems` 添加导航项
4. 如果需要新的 API，在 `src/api/` 创建对应文件

### 场景 3: 修改版本号

**必须同步修改的核心文件**（3 个）：

1. `package.json` - `"version": "x.x.x"`
2. `src-tauri/Cargo.toml` - `version = "x.x.x"`
3. `src-tauri/tauri.conf.json` - `"version": "x.x.x"`

**如果发布到 AUR，还需要修改**（2 个）：

4. `PKGBUILD` - `pkgver=x.x.x`
5. `.SRCINFO` - `pkgver = x.x.x`（注意有空格）

**更新依赖锁定文件**：

```bash
# 更新 Cargo.lock
cargo update -p sea-lantern

# 更新 package-lock.json
npm install
```

**完整版本更新流程**：

```bash
# 1. 手动修改上述 5 个文件的版本号

# 2. 更新依赖锁定文件
cd src-tauri
cargo update -p sea-lantern
cd ..
npm install

# 3. 提交更改
git add .
git commit -m "chore: bump version to x.x.x"

# 4. 创建标签（用于 GitHub Release）
git tag sea-lantern-vx.x.x
git push origin main
git push origin sea-lantern-vx.x.x
```

**注意事项**：
- 版本号格式遵循语义化版本（Semantic Versioning）：`major.minor.patch`
- 标签格式：`sea-lantern-vx.x.x`（与 GitHub Release 保持一致）
- PKGBUILD 中的 `source` URL 也会引用版本号，确保 Release 已发布

### 场景 4: 添加 Tauri 插件权限

在 `src-tauri/capabilities/default.json` 的 `permissions` 数组中添加权限。

**示例**：添加 opener 插件的 openUrl 权限

```json
{
  "permissions": ["opener:allow-open-url"]
}
```

**注意**：权限名称格式为 `plugin-name:allow-function-name`

### 场景 5: 修改窗口配置

在 `src-tauri/tauri.conf.json` 的 `windows` 数组中修改：

- `width` / `height` - 窗口大小
- `title` - 窗口标题
- `resizable` - 是否可调整大小
- `fullscreen` - 是否全屏
- `decorations` - 是否显示标题栏

### 场景 6: 添加新的 CSS 变量

在 `src/styles/variables.css` 的 `:root` 中添加变量，然后在组件中使用 `var(--variable-name)`。

---

## 重要技术细节

### 1. Tauri 插件使用

#### opener 插件（打开外部链接）

```typescript
import { openUrl } from "@tauri-apps/plugin-opener";

await openUrl("https://example.com");
```

**权限**：`opener:allow-open-url`

#### dialog 插件（文件对话框）

```typescript
import { open } from "@tauri-apps/plugin-dialog";

const file = await open({
  multiple: false,
  filters: [{ name: "JAR", extensions: ["jar"] }],
});
```

**权限**：`dialog:allow-open`

### 2. 版本比较逻辑

在 `src-tauri/src/commands/update.rs` 中实现了语义化版本比较：

```rust
fn compare_versions(v1: &str, v2: &str) -> std::cmp::Ordering {
    // 比较 major.minor.patch
}
```

### 3. 控制台日志轮询

前端每 800ms 轮询一次后端获取新日志：

```typescript
// src/stores/consoleStore.ts
setInterval(async () => {
  const logs = await getServerLogs(serverId);
  // 更新 store
}, 800);
```

### 4. 毛玻璃效果实现

```css
.glass {
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.2);
}
```

### 5. 全局状态管理

使用 Pinia，状态定义在 `src/stores/` 中。

**示例**：访问 serverStore

```typescript
import { useServerStore } from "@/stores/serverStore";

const serverStore = useServerStore();
const servers = serverStore.servers;
```

---

## 常见问题和解决方案

### 问题 1: AboutView 页面打不开

**原因**：opener 插件导入错误或权限配置错误

**解决方案**：

1. 确保导入正确：`import { openUrl } from '@tauri-apps/plugin-opener'`
2. 确保权限正确：`opener:allow-open-url`（不是 `opener:allow-open`）

### 问题 2: 命令调用失败

**检查清单**：

1. 命令是否在 `lib.rs` 的 `generate_handler!` 中注册
2. 命令函数是否添加了 `#[command]` 宏
3. 参数类型是否匹配
4. 是否需要权限配置

### 问题 3: 端口冲突

**默认端口**：

- 海晶灯主项目：1420
- 开发者工具：1430

**修改位置**：

- `vite.config.ts` - `server.port`
- `src-tauri/tauri.conf.json` - `devUrl`

### 问题 4: 构建失败

**常见原因**：

1. Rust 依赖未安装：运行 `cargo build`
2. Node 依赖未安装：运行 `npm install`
3. Tauri CLI 未安装：运行 `npm install -g @tauri-apps/cli`

---

## 开发命令

```bash
# 开发模式（热重载）
npm run tauri dev

# 构建发布版
npm run tauri build

# 仅构建前端
npm run build

# 类型检查 + 前端构建检查
npm run build:check

# Rust 编译检查
cd src-tauri && cargo check

# Rust 测试
cd src-tauri && cargo test
```

---

## Git 工作流

### 发布新版本流程

1. 修改版本号（3 个文件）
2. 提交代码：`git add . && git commit -m "chore: bump version to x.x.x"`
3. 创建标签：`git tag vx.x.x`
4. 推送代码：`git push origin main`
5. 推送标签：`git push origin vx.x.x`
6. 构建发布版：`npm run tauri build`
7. 在 Gitee 创建 Release，上传构建产物

---

## 项目依赖

### 前端依赖

- `vue` - Vue 3 框架
- `vue-router` - 路由管理
- `pinia` - 状态管理
- `@tauri-apps/api` - Tauri API
- `@tauri-apps/plugin-opener` - 打开外部链接
- `@tauri-apps/plugin-dialog` - 文件对话框
- `typescript` - TypeScript 支持
- `vite` - 构建工具

### 后端依赖（Cargo.toml）

- `tauri` - Tauri 框架
- `serde` - 序列化/反序列化
- `serde_json` - JSON 处理
- `tokio` - 异步运行时
- `reqwest` - HTTP 客户端（用于更新检查）

---

## 注意事项

### 1. 不要修改的文件

- `node_modules/` - 自动生成
- `src-tauri/target/` - 构建产物
- `dist/` - 前端构建产物
- `package-lock.json` - 自动生成
- `Cargo.lock` - 自动生成

### 2. 修改时需要同步的文件

**版本号更新**（5 个文件 + 2 个自动生成）：
- 核心：`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`
- AUR：`PKGBUILD`, `.SRCINFO`
- 自动：`Cargo.lock`（cargo update）, `package-lock.json`（npm install）

**添加 Tauri 命令**（4 个文件）：
- `src-tauri/src/commands/*.rs` - 定义命令函数
- `src-tauri/src/commands/mod.rs` - 导出模块
- `src-tauri/src/lib.rs` - 注册到 `generate_handler!`
- `src/api/*.ts` - 前端 API 封装

**添加新页面**（2 个文件）：
- `src/router/index.ts` - 添加路由
- `src/components/layout/AppSidebar.vue` - 添加导航项

### 3. 代码风格

- 前端：使用 2 空格缩进
- 后端：使用 4 空格缩进（Rust 标准）
- 使用有意义的变量名和函数名
- 添加必要的注释
- 参考check.yml

### 4. 性能优化建议

- 避免在循环中调用 Tauri 命令
- 大量数据传输时考虑分页
- 使用 Pinia store 缓存数据，减少重复请求

---

## 相关项目

### sea-lantern-dev-tools

独立的开发者工具项目，用于可视化编辑海晶灯的配置文件。

**位置**：`D:\Game\minecraft\server\Sea Lantern\sea-lantern-dev-tools\`
**端口**：1430
**功能**：编辑应用信息（版本、名称、作者、构建年份）

---

## 联系方式

- Gitee: https://gitee.com/fps_z/SeaLantern
- B站: https://space.bilibili.com/1192466596

---

**最后更新**: 2026-02-17
**文档版本**: 1.1
**当前项目版本**: 0.6.2
