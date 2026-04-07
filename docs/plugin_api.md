# 插件 API 文档

本文档基于当前插件运行时实现，汇总 Sea Lantern 提供给 Lua 插件的公开 API、权限门槛与行为限制，便于按照现状开发与排错。

## 目录

- [插件 API 文档](#插件-api-文档)
  - [目录](#目录)
  - [运行时与权限模型](#运行时与权限模型)
  - [Element API](#element-api)
  - [Server API](#server-api)
  - [Console API](#console-api)
  - [Plugins API](#plugins-api)
    - [额外限制](#额外限制)
  - [FileSystem API](#filesystem-api)
    - [行为说明](#行为说明)
  - [前端插件管理命令](#前端插件管理命令)
  - [MCS 插件管理命令](#mcs-插件管理命令)
  - [权限说明](#权限说明)
    - [运行时模块权限](#运行时模块权限)
    - [当前权限元数据](#当前权限元数据)
  - [示例](#示例)
    - [发送命令到服务器](#发送命令到服务器)
    - [获取服务器状态](#获取服务器状态)
    - [列出其他插件文件](#列出其他插件文件)
    - [文件系统操作](#文件系统操作)

## 运行时与权限模型

- 插件运行时会创建全局 `sl` 命名空间，并按清单中的 `permissions` 决定是否挂载对应模块。
- 旧权限 `fs` 会在运行时自动归一化为 `fs.data`，因此仅代表“插件私有数据目录”访问权。
- 文件系统权限支持两级控制：
  - scope 级：`fs.data`、`fs.server`、`fs.global`
  - action 级：如 `fs.data.read`、`fs.server.write`、`fs.global.transfer`
- 未授权模块不会直接消失，而是被替换为权限拒绝模块，调用时会抛出权限错误。
- 所有 `sl.plugins.*` 与 `sl.fs.*` 调用都会写入权限日志，可通过前端命令查询指定插件的调用记录。

## Element API

| 方法名                                     | 参数                                       | 返回值            | 描述                           |
| ------------------------------------------ | ------------------------------------------ | ----------------- | ------------------------------ | -------- |
| `sl.element.get_text(selector)`            | `selector: string` - CSS 选择器            | `string` 或 `nil` | 获取指定元素文本               |
| `sl.element.get_value(selector)`           | `selector: string` - CSS 选择器            | `string` 或 `nil` | 获取指定元素值                 |
| `sl.element.get_attribute(selector, attr)` | `selector: string`<br>`attr: string`       | `string` 或 `nil` | 获取指定属性值                 |
| `sl.element.get_attributes(selector)`      | `selector: string`                         | `object` 或 `nil` | 获取元素全部属性               |
| `sl.element.click(selector)`               | `selector: string`                         | `boolean`         | 点击元素                       |
| `sl.element.set_value(selector, value)`    | `selector: string`<br>`value: string`      | `boolean`         | 设置元素值                     |
| `sl.element.check(selector, checked)`      | `selector: string`<br>`checked: boolean`   | `boolean`         | 切换勾选状态                   |
| `sl.element.select(selector, value)`       | `selector: string`<br>`value: string`      | `boolean`         | 选择下拉项                     |
| `sl.element.focus(selector)`               | `selector: string`                         | `boolean`         | 聚焦元素                       |
| `sl.element.blur(selector)`                | `selector: string`                         | `boolean`         | `boolean`                      | 取消聚焦 |
| `sl.element.on_change(selector, callback)` | `selector: string`<br>`callback: function` | `function`        | 监听元素变化，返回取消监听函数 |

## Server API

| 方法名                                                    | 参数                                                                | 返回值    | 描述                     |
| --------------------------------------------------------- | ------------------------------------------------------------------- | --------- | ------------------------ |
| `sl.server.list()`                                        | 无                                                                  | `table`   | 获取服务器列表           |
| `sl.server.get_path(server_id)`                           | `server_id: string`                                                 | `string`  | 获取服务器路径           |
| `sl.server.read_file(server_id, relative_path)`           | `server_id: string`<br>`relative_path: string`                      | `string`  | 读取服务器文件           |
| `sl.server.write_file(server_id, relative_path, content)` | `server_id: string`<br>`relative_path: string`<br>`content: string` | `boolean` | 写入服务器文件           |
| `sl.server.list_dir(server_id, relative_path)`            | `server_id: string`<br>`relative_path: string`                      | `table`   | 列出服务器目录           |
| `sl.server.exists(server_id, relative_path)`              | `server_id: string`<br>`relative_path: string`                      | `boolean` | 检查文件是否存在         |
| `sl.server.logs.get(server_id, count)`                    | `server_id: string`<br>`count?: number`                             | `table`   | 获取指定服务器日志       |
| `sl.server.logs.getAll(count)`                            | `count?: number`                                                    | `table`   | 获取所有运行中服务器日志 |

## Console API

| 方法名                                  | 参数                                     | 返回值    | 描述           |
| --------------------------------------- | ---------------------------------------- | --------- | -------------- |
| `sl.console.send(server_id, command)`   | `server_id: string`<br>`command: string` | `boolean` | 发送控制台命令 |
| `sl.console.get_logs(server_id, count)` | `server_id: string`<br>`count?: number`  | `table`   | 获取控制台日志 |
| `sl.console.get_status(server_id)`      | `server_id: string`                      | `string`  | 获取服务器状态 |

## Plugins API

`sl.plugins` 命名空间只会在插件声明 `plugin_folder_access` 权限时注册，用于访问“其他插件目录”。插件根目录会先定位到当前插件目录的父目录，然后对目标插件 ID 与相对路径做路径校验。

| 方法名                                                     | 参数                                                                | 返回值            | 描述                                                               |
| ---------------------------------------------------------- | ------------------------------------------------------------------- | ----------------- | ------------------------------------------------------------------ |
| `sl.plugins.list()`                                        | 无                                                                  | `table`           | 扫描插件根目录下所有包含 `manifest.json` 的插件目录                |
| `sl.plugins.get_manifest(target_id)`                       | `target_id: string`                                                 | `table` 或 `nil`  | 读取目标插件 `manifest.json` 并转换为 Lua 表                       |
| `sl.plugins.read_file(target_id, relative_path)`           | `target_id: string`<br>`relative_path: string`                      | `string` 或 `nil` | 读取目标插件文本文件；路径非法、文件不存在或目标为目录时返回 `nil` |
| `sl.plugins.file_exists(target_id, relative_path)`         | `target_id: string`<br>`relative_path: string`                      | `boolean`         | 检查目标插件路径是否存在；路径非法时返回 `false`                   |
| `sl.plugins.list_files(target_id, relative_path)`          | `target_id: string`<br>`relative_path: string`                      | `table`           | 列出目录项，字段包含 `name`、`is_dir`、`size`、`modified_at`       |
| `sl.plugins.write_file(target_id, relative_path, content)` | `target_id: string`<br>`relative_path: string`<br>`content: string` | `boolean`         | 写入目标插件文本文件，成功时返回 `true`                            |

### 额外限制

- `sl.plugins.read_file()` 与 `sl.plugins.write_file()` 的内容大小上限均为 10 MiB。
- `sl.plugins.list()` 会跳过没有 `manifest.json`、清单无法读取或 JSON 解析失败的目录。
- `sl.plugins.get_manifest()` 在目标插件缺少 `manifest.json` 时返回 `nil`。
- `sl.plugins.list_files()` 输出按“目录优先、名称不区分大小写排序、最后按原名称排序”。
- `sl.plugins.write_file()` 禁止写入 `manifest.json`。
- `sl.plugins.write_file()` 禁止写入符号链接、禁止穿过符号链接父目录、禁止将目录当成文件写入。
- `sl.plugins.write_file()` 在父目录不存在时会自动创建目录。
- 所有调用都会记录为权限日志，日志类型为 `api_call`。

## FileSystem API

`sl.fs` 命名空间在插件拥有任意文件系统权限时启用。scope 固定为 `data`、`server`、`global`：

- `data`：插件私有数据目录
- `server`：服务器目录
- `global`：全局目录

| 方法名                                    | 参数                                                        | 返回值    | 描述                               |
| ----------------------------------------- | ----------------------------------------------------------- | --------- | ---------------------------------- |
| `sl.fs.read(scope, path)`                 | `scope: string`<br>`path: string`                           | `string`  | 读取文本文件                       |
| `sl.fs.read_binary(scope, path)`          | `scope: string`<br>`path: string`                           | `string`  | 读取二进制文件并返回 Base64 字符串 |
| `sl.fs.write(scope, path, content)`       | `scope: string`<br>`path: string`<br>`content: string`      | `nil`     | 写入文件                           |
| `sl.fs.exists(scope, path)`               | `scope: string`<br>`path: string`                           | `boolean` | 检查文件是否存在                   |
| `sl.fs.list(scope, path)`                 | `scope: string`<br>`path: string`                           | `table`   | 列出目录内容                       |
| `sl.fs.mkdir(scope, path)`                | `scope: string`<br>`path: string`                           | `nil`     | 递归创建目录                       |
| `sl.fs.remove(scope, path)`               | `scope: string`<br>`path: string`                           | `nil`     | 删除文件或空目录                   |
| `sl.fs.info(scope, path)`                 | `scope: string`<br>`path: string`                           | `table`   | 获取文件元数据                     |
| `sl.fs.copy(scope, src, dst)`             | `scope: string`<br>`src: string`<br>`dst: string`           | `nil`     | 复制文件或目录                     |
| `sl.fs.move(scope, src, dst)`             | `scope: string`<br>`src: string`<br>`dst: string`           | `nil`     | 移动文件或目录                     |
| `sl.fs.rename(scope, old_path, new_path)` | `scope: string`<br>`old_path: string`<br>`new_path: string` | `nil`     | 重命名文件或目录                   |
| `sl.fs.get_path(scope)`                   | `scope: string`                                             | `string`  | 获取当前 scope 对应的真实根路径    |

### 行为说明

- scope 必须是 `data`、`server`、`global` 之一，否则会报错。
- 每次调用都要求拥有对应的 scope 权限或 action 权限，例如：
  - 读 `data` 目录：`fs.data` 或 `fs.data.read`
  - 写 `server` 目录：`fs.server` 或 `fs.server.write`
  - 复制 `global` 目录：`fs.global` 或 `fs.global.transfer`
- 所有路径都会经过沙箱校验，禁止越出对应根目录。
- 文件系统沙箱显式拒绝符号链接与重解析点访问。
- `sl.fs.remove()` 会拒绝删除 sandbox 根目录本身。
- `sl.fs.copy()` 复制目录时会递归复制，并在目标已存在时拒绝覆盖。
- 所有 `sl.fs.*` 调用都会写入权限日志。

## 前端插件管理命令

以下命令由前端通过 Tauri invoke 调用，用于管理 Sea Lantern 自身的 Lua 插件系统：

| 前端封装                                                     | 对应命令                           | 说明                                      |
| ------------------------------------------------------------ | ---------------------------------- | ----------------------------------------- |
| `listPlugins()`                                              | `list_plugins`                     | 获取已扫描插件列表                        |
| `scanPlugins()`                                              | `scan_plugins`                     | 重新扫描插件目录                          |
| `enablePlugin(pluginId)`                                     | `enable_plugin`                    | 启用插件                                  |
| `disablePlugin(pluginId)`                                    | `disable_plugin`                   | 禁用插件，并返回受影响依赖                |
| `getPluginNavItems()`                                        | `get_plugin_nav_items`             | 获取插件注册的导航项                      |
| `installPlugin(path)`                                        | `install_plugin`                   | 从 `.zip`、`manifest.json` 或插件目录安装 |
| `installPluginsBatch(paths)`                                 | `install_plugins_batch`            | 批量安装插件                              |
| `getPluginIcon(pluginId)`                                    | `get_plugin_icon`                  | 获取插件图标                              |
| `getPluginSettings(pluginId)`                                | `get_plugin_settings`              | 获取插件设置                              |
| `setPluginSettings(pluginId, settings)`                      | `set_plugin_settings`              | 保存插件设置                              |
| `getPluginCss(pluginId)`                                     | `get_plugin_css`                   | 获取单个插件 CSS                          |
| `getAllPluginCss()`                                          | `get_all_plugin_css`               | 获取所有插件 CSS                          |
| `deletePlugin(pluginId, deleteData?)`                        | `delete_plugin`                    | 删除插件，可选删除数据目录                |
| `deletePlugins(pluginIds, deleteData?)`                      | `delete_plugins`                   | 批量删除插件                              |
| `checkPluginUpdate(pluginId)`                                | `check_plugin_update`              | 检查单个插件更新                          |
| `checkAllPluginUpdates()`                                    | `check_all_plugin_updates`         | 检查全部插件更新                          |
| `fetchMarketPlugins(marketUrl?)`                             | `fetch_market_plugins`             | 拉取插件市场列表                          |
| `fetchMarketPluginDetail(pluginPath, marketUrl?)`            | `fetch_market_plugin_detail`       | 拉取插件市场详情                          |
| `fetchMarketCategories(marketUrl?)`                          | `fetch_market_categories`          | 拉取插件分类                              |
| `installFromMarket(options)`                                 | `install_from_market`              | 从插件市场安装                            |
| `onLocaleChanged(locale)`                                    | `on_locale_changed`                | 通知插件语言切换                          |
| `onPageChanged(path)`                                        | `on_page_changed`                  | 通知插件当前页面切换                      |
| `componentMirrorClear()`                                     | `component_mirror_clear`           | 清空组件镜像注册表                        |
| `contextMenuCallback(pluginId, context, itemId, targetData)` | `context_menu_callback`            | 回调插件上下文菜单事件                    |
| `getPluginUiSnapshot()`                                      | `get_plugin_ui_snapshot`           | 读取并消费插件 UI 事件快照                |
| `getPluginSidebarSnapshot()`                                 | `get_plugin_sidebar_snapshot`      | 读取并消费插件侧边栏事件快照              |
| `getPluginContextMenuSnapshot()`                             | `get_plugin_context_menu_snapshot` | 读取并消费插件右键菜单事件快照            |
| `getPluginComponentSnapshot()`                               | `get_plugin_component_snapshot`    | 读取并消费插件组件事件快照                |
| `getPluginPermissionLogs(pluginId)`                          | `get_plugin_permission_logs`       | 查询指定插件权限日志                      |

## MCS 插件管理命令

`src/api/mcs_plugins.ts` 这组封装对应的是“Minecraft 服务端插件文件管理”，与上文 Lua 插件系统不同：

| 前端封装                                                             | 对应命令                    | 说明                                 |
| -------------------------------------------------------------------- | --------------------------- | ------------------------------------ |
| `m_pluginApi.m_getPlugins(serverId)`                                 | `m_get_plugins`             | 扫描指定服务端的插件文件             |
| `m_pluginApi.m_getPluginConfigFiles(serverId, fileName, pluginName)` | `m_get_plugin_config_files` | 读取插件配置文件                     |
| `m_pluginApi.m_togglePlugin(serverId, fileName, enabled)`            | `m_toggle_plugin`           | 通过重命名切换启用状态               |
| `m_pluginApi.m_deletePlugin(serverId, fileName)`                     | `m_delete_plugin`           | 删除服务端插件文件                   |
| `m_pluginApi.m_installPlugin(serverId, fileData, fileName)`          | `m_install_plugin`          | 安装上传的插件文件                   |
| `m_pluginApi.m_reloadPlugins(serverId)`                              | `send_command`              | 实际上直接向服务端发送 `reload` 命令 |

## 权限说明

### 运行时模块权限

- `sl.element`：需要 `element`
- `sl.server`：需要 `server`
- `sl.console`：需要 `console`
- `sl.plugins`：需要 `plugin_folder_access`
- `sl.fs`：需要任意文件系统权限：`fs.data`、`fs.server`、`fs.global` 或更细粒度的 action 权限
- `sl.storage`：需要 `storage`
- `sl.ui`：需要 `ui`
- `sl.http` / 网络模块：需要 `network`
- `sl.system`：需要 `system`
- 进程执行模块：需要 `execute_program`
- 插件间 API：需要 `api`

### 当前权限元数据

当前实现中登记的权限包括：

- `log`
- `storage`
- `ui`
- `element`
- `fs`
- `api`
- `network`
- `system`
- `server`
- `console`
- `execute_program`
- `plugin_folder_access`
- `ui.component.read`
- `ui.component.write`
- `ui.component.proxy`
- `ui.component.create`

其中高风险权限包括：

- Critical：`execute_program`、`plugin_folder_access`、`ui.component.proxy`
- Dangerous：`fs`、`network`、`server`、`console`、`ui.component.write`、`ui.component.create`

## 示例

### 发送命令到服务器

```lua
local success = sl.console.send("server1", "say Hello from plugin!")
if success then
    print("Command sent successfully")
else
    print("Failed to send command")
end
```

### 获取服务器状态

```lua
local status = sl.console.get_status("server1")
print("Server status: " .. status)
```

### 列出其他插件文件

```lua
local plugins = sl.plugins.list()
for i, plugin in ipairs(plugins) do
    print(plugin.id, plugin.name)
end

local files = sl.plugins.list_files("example-plugin", ".")
for i, item in ipairs(files) do
    print(item.name, item.is_dir, item.size)
end
```

### 文件系统操作

```lua
-- 读取文件
local content = sl.fs.read("data", "config.txt")
print("File content: " .. content)

-- 写入文件
sl.fs.write("data", "output.txt", "Hello, world!")

-- 检查文件是否存在
local exists = sl.fs.exists("data", "config.txt")
print("File exists: " .. tostring(exists))

-- 列出目录内容
local files = sl.fs.list("data", "")
for i, file in ipairs(files) do
    print("File " .. i .. ": " .. file)
end
```
