# 前端说明

`frontend/` 是项目前端（辛苦前端仙人们），负责界面、状态、语言、主题，以及对 Tauri 后端命令的调用封装。

## 目录结构

```text
frontend/                   frontend/
├── index.html              └── HTML 入口
├── package.json                依赖和脚本
├── scripts/                    辅助脚本
└── src/                        主代码
    ├── main.ts                 └── 应用入口
    ├── main.next.ts                next 入口
    ├── NextRoot.vue                next 根组件
    ├── api/                        Tauri 调用
    ├── router/                     路由和元数据
    ├── pages/                      页面入口
    ├── views/                      业务视图
    ├── layout/                     布局
    ├── shell/                      应用外壳
    ├── composables/                组合式逻辑
    ├── services/                   前端服务
    ├── host/                       宿主运行时
    ├── launcher/                   启动适配
    ├── data/                       静态数据
    ├── contracts/                  契约类型
    ├── types/                      通用类型
    ├── assets/                     静态资源
    ├── utils/                      工具函数
    ├── styles/                     样式
    ├── themes/                     主题
    ├── language/                   语言资源
    ├── components/                 组件
    │   ├── common/                 └── 基础组件
    │   ├── plugin/                     插件运行时组件
    │   ├── plugins/                    插件页面组件
    │   ├── servers/                    服务器列表组件
    │   ├── server-instance/            服务器实例组件
    │   ├── host/                       宿主插槽组件
    │   ├── shell/                      桌面外壳组件
    │   └── startup/                    启动期组件
    ├── features/                   业务流程
    │   └── config-editor/          └── 配置编辑
    └── stores/                     Pinia 状态
        └── plugin/                 └── 插件状态拆分
```

## 表格对照

| 改动内容 | 优先查看 |
| --- | --- |
| Tauri 命令调用 | `src/api/` |
| 新增或调整页面 | `src/router/`、`src/pages/`、`src/views/`、`src/router/pageMeta.ts` |
| 页面内组件 | `src/components/`、对应页面所在目录 |
| 应用外壳、导航、布局 | `src/layout/`、`src/shell/`、`src/components/shell/`、`src/NextRoot.vue` |
| 宿主插槽、宿主能力 | `src/host/`、`src/components/host/`、`src/services/hostCapabilities.ts` |
| 桌面 shell 启动适配 | `src/launcher/desktopShell.ts` |
| 跨页面业务流程 | `src/features/`、`src/services/` |
| 可复用 Vue 逻辑 | `src/composables/` |
| 全局状态 | `src/stores/` |
| 类型定义 | `src/types/`、`src/contracts/` |
| 样式 | `src/styles/` |
| 主题 | `src/themes/` |
| 多语言文案 | `src/language/` |
| Sea Lantern Lua 插件界面 | `src/api/plugin.ts`、`src/pages/plugins/`、`src/components/plugin/`、`src/components/plugins/`、`src/stores/plugin/` |
| Minecraft 服务端插件文件管理 | `src/api/mcs_plugins.ts`、对应插件管理页面 |
| 更新检查和更新界面 | `src/api/update.ts`、`src/stores/updateStore.ts`、更新相关视图 |
| 服务器列表、创建、导入、启动流程 | `src/api/server.ts`、`src/stores/serverStore.ts`、`src/pages/servers/`、`src/pages/server-instance/` |
| 服务端配置编辑 | `src/api/config.ts`、`src/features/config-editor/`、`src/pages/server-instance/config/` |
| 控制台 | 控制台视图、`src/stores/consoleStore.ts`、后端日志/命令 API |
| 玩家、白名单、封禁、OP | `src/api/player.ts`、`src/pages/server-instance/players/` |
| 联机、隧道、Join ID | `src/api/tunnel.ts`、`src/pages/tunnel/` |
| 下载任务、Java、服务端核心下载 | `src/api/downloader.ts`、`src/api/java.ts`、`src/pages/downloads/` |
| 设置、数据目录、个性化导入导出 | `src/api/settings.ts`、`src/pages/settings/`、`src/pages/paint/` |
| 认证入口 | `src/services/auth*.ts`、`src/stores/auth*.ts`、`src/router/authRoute.ts`、`src/views/AuthEntryView.vue` |

## 开发命令

安装依赖：

```bash
pnpm --dir frontend install
```

启动 Tauri 桌面开发环境：

```bash
pnpm --dir frontend run tauri:dev
```

只启动前端页面：

```bash
pnpm --dir frontend run dev
```

只启动 next shell 指定页面：

```bash
pnpm --dir frontend run next
pnpm --dir frontend run next:servers
pnpm --dir frontend run next:plugins
pnpm --dir frontend run next:settings
```

只启动 HTTP / Docker 后端：

```bash
pnpm --dir frontend run dev:http:backend
```

提交前检查：

```bash
pnpm --dir frontend run lint
pnpm --dir frontend run build:check
pnpm --dir frontend run fmt:check
```

检查语言资源：

```bash
pnpm --dir frontend run i18n:check
```

同步语言资源：

```bash
pnpm --dir frontend run i18n:sync
```

## 注意事项

### API 调用

- `src/api/*.ts` 是前后端契约入口。
- 改命令名、参数、返回值时，要同时检查后端命令注册和实现。
- 后端命令注册在 `backend/tauri-host/src/runtime/command_catalog.rs`。
- 后端命令实现一般在 `backend/tauri-host/src/commands/`。
- `src/api/tauri.ts` 是统一 invoke 封装，改错误处理或调用行为时要检查所有 API 调用方。

### 状态

- 多页面共享状态放 `src/stores/`。
- 单页面临时输入状态优先留在组件或页面内。
- 不要把后端规则复制成前端状态规则。
- store 里保存的数据要注意刷新、失效和错误状态。
- 插件相关状态已经拆到 `src/stores/plugin/`，不要再把大块插件运行时逻辑塞回 `pluginStore.ts`。

### 插件

- Sea Lantern Lua 插件系统和 Minecraft 服务端插件文件管理不是一回事。
- Lua 插件相关 API 通常在 `src/api/plugin.ts`。
- Minecraft 服务端插件文件管理通常在 `src/api/mcs_plugins.ts`。
- 插件 UI snapshot、sidebar、context menu、component mirror、permission log 都是插件可见行为，改动前要确认兼容性。
- 插件页面在 `src/pages/plugins/`，单个服务器的插件文件管理在 server instance 相关页面里，不要混用状态和文案。

### 语言和主题

- 新增用户可见文案时，放进 `src/language/`。
- 不要在新页面里硬编码中文或英文。
- 新增颜色和视觉状态时，优先使用已有 CSS 变量和 `src/themes/`。
- 不要为了单个页面新建一套独立颜色系统。

### 界面

- Sea Lantern 是桌面工具，页面应优先清楚、稳定、可重复使用。
- 改现有页面时，先沿用已有组件、store 和样式结构。
- 只有在旧结构确实挡住修改时，再拆组件或移动逻辑。
- 涉及 Tauri、文件系统、服务器启动、插件运行时或更新流程时，最好跑一次 `tauri:dev` 验证真实行为。
- `src/main.next.ts` 和 next shell 是当前活跃路径，不要假设旧的 `next-src` 目录存在。

## To Agent

- Treat `src/api/*.ts` as frontend-backend contract code.
- Check frontend callers, backend command catalog, and backend command implementations together.
- Do not mix Sea Lantern Lua plugin behavior with Minecraft server plugin file management.
- Put user-facing strings in language resources.
- Use existing CSS variables, themes, stores, and component patterns before adding new structure.
- Run frontend lint, type/build check, and format check when practical.
