# UI Lua 接口说明

本文档说明插件运行时暴露的 [`sl.ui`](./mod.rs) Lua 接口，用于注入 HTML / CSS、控制页面元素、显示反馈、注册侧边栏与上下文菜单，以及与宿主组件系统交互。

## 接口总览

| Lua 接口                                                    | 参数                                                           | 返回值                 | 说明                                            | 对应实现                                             |
| ----------------------------------------------------------- | -------------------------------------------------------------- | ---------------------- | ----------------------------------------------- | ---------------------------------------------------- |
| `sl.ui.set_error_mode(mode)`                                | `mode: string`                                                 | `boolean`              | 设置 UI 接口错误模式，支持 `compat` 与 `strict` | [`config::register()`](./config.rs:4)                |
| `sl.ui.inject_html(elementId, html)`                        | `elementId: string`，`html: string`                            | `boolean`              | 向指定元素注入 HTML 内容                        | [`basic::register()`](./basic.rs:5)                  |
| `sl.ui.remove_html(elementId)`                              | `elementId: string`                                            | `boolean`              | 移除指定元素已注入的 HTML 内容                  | [`basic::register()`](./basic.rs:32)                 |
| `sl.ui.update_html(elementId, html)`                        | `elementId: string`，`html: string`                            | `boolean`              | 更新指定元素的 HTML 内容                        | [`basic::register()`](./basic.rs:57)                 |
| `sl.ui.query(selector)`                                     | `selector: string`                                             | `boolean`              | 向宿主发起 UI 查询请求                          | [`basic::register()`](./basic.rs:83)                 |
| `sl.ui.inject_css(styleId, css)`                            | `styleId: string`，`css: string`                               | `boolean`              | 注入一段具名 CSS                                | [`style::register()`](./style.rs:8)                  |
| `sl.ui.remove_css(styleId)`                                 | `styleId: string`                                              | `boolean`              | 移除此前注入的具名 CSS                          | [`style::register()`](./style.rs:34)                 |
| `sl.ui.hide(selector)`                                      | `selector: string`                                             | `boolean`              | 隐藏匹配选择器的元素                            | [`style::register()`](./style.rs:58)                 |
| `sl.ui.show(selector)`                                      | `selector: string`                                             | `boolean`              | 显示匹配选择器的元素                            | [`style::register()`](./style.rs:82)                 |
| `sl.ui.disable(selector)`                                   | `selector: string`                                             | `boolean`              | 禁用匹配选择器的元素                            | [`style::register()`](./style.rs:106)                |
| `sl.ui.enable(selector)`                                    | `selector: string`                                             | `boolean`              | 启用匹配选择器的元素                            | [`style::register()`](./style.rs:130)                |
| `sl.ui.insert(placement, selector, html)`                   | `placement: string`，`selector: string`，`html: string`        | `boolean`              | 按位置向目标元素前后或内部插入 HTML             | [`style::register()`](./style.rs:154)                |
| `sl.ui.remove(selector)`                                    | `selector: string`                                             | `boolean`              | 删除匹配选择器的元素                            | [`style::register()`](./style.rs:190)                |
| `sl.ui.set_style(selector, styles)`                         | `selector: string`，`styles: table<string, string>`            | `boolean`              | 批量设置内联样式                                | [`style::register()`](./style.rs:214)                |
| `sl.ui.set_attribute(selector, attr, value)`                | `selector: string`，`attr: string`，`value: string`            | `boolean`              | 设置元素属性                                    | [`style::register()`](./style.rs:246)                |
| `sl.ui.toast(type, message, duration?)`                     | `type: string`，`message: string`，`duration?: integer`        | `boolean`              | 显示一条 Toast 提示                             | [`feedback::register()`](./feedback.rs:6)            |
| `sl.ui.register_sidebar(config)`                            | `config: table`，其中 `label: string`，`icon?: string`         | `boolean`              | 注册插件侧边栏入口                              | [`sidebar::register()`](./sidebar.rs:6)              |
| `sl.ui.unregister_sidebar()`                                | 无                                                             | `boolean`              | 注销当前插件侧边栏入口                          | [`sidebar::register()`](./sidebar.rs:28)             |
| `sl.ui.register_context_menu(context, items)`               | `context: string`，`items: table<number, table>`               | `boolean`              | 在指定上下文注册插件菜单项                      | [`context_menu::register()`](./context_menu.rs:8)    |
| `sl.ui.unregister_context_menu(context)`                    | `context: string`                                              | `boolean`              | 注销指定上下文下的插件菜单项                    | [`context_menu::register()`](./context_menu.rs:60)   |
| `sl.ui.on_context_menu_click(callback)`                     | `callback: function`                                           | `boolean`              | 注册上下文菜单点击回调                          | [`context_menu::register()`](./context_menu.rs:80)   |
| `sl.ui.on_context_menu_show(callback)`                      | `callback: function`                                           | `boolean`              | 注册上下文菜单显示回调                          | [`context_menu::register()`](./context_menu.rs:95)   |
| `sl.ui.on_context_menu_hide(callback)`                      | `callback: function`                                           | `boolean`              | 注册上下文菜单隐藏回调                          | [`context_menu::register()`](./context_menu.rs:110)  |
| `sl.ui.component.list(pageFilter?)`                         | `pageFilter?: string`                                          | `table<number, table>` | 获取当前已镜像的宿主组件列表                    | [`component::register_list()`](./component.rs:46)    |
| `sl.ui.component.get(componentId, prop)`                    | `componentId: string`，`prop: string`                          | `boolean`              | 请求读取组件属性                                | [`component::register_get()`](./component.rs:70)     |
| `sl.ui.component.set(componentId, prop, value)`             | `componentId: string`，`prop: string`，`value: any`            | `boolean`              | 请求设置组件属性                                | [`component::register_set()`](./component.rs:94)     |
| `sl.ui.component.call(componentId, method)`                 | `componentId: string`，`method: string`                        | `boolean`              | 请求调用组件方法                                | [`component::register_call()`](./component.rs:119)   |
| `sl.ui.component.on(componentId, event)`                    | `componentId: string`，`event: string`                         | `boolean`              | 请求订阅组件事件                                | [`component::register_on()`](./component.rs:143)     |
| `sl.ui.component.create(componentType, componentId, props)` | `componentType: string`，`componentId: string`，`props: table` | `boolean`              | 请求创建宿主组件                                | [`component::register_create()`](./component.rs:167) |

## 使用说明

### 1. 设置错误模式

```lua
sl.ui.set_error_mode("compat")
```

说明：

- `compat` 模式下，宿主事件发送失败时多数 UI 接口返回 `false`。
- `strict` 模式下，宿主事件发送失败会直接抛出 Lua 运行时错误。
- 该行为由 [`set_error_mode()`](./common.rs:92) 与 [`emit_result()`](./common.rs:112) 控制。

### 2. 注入与更新 HTML

```lua
sl.ui.inject_html("plugin-root", [[
  <div class="plugin-panel">Hello UI</div>
]])

sl.ui.update_html("plugin-root", [[
  <div class="plugin-panel">Updated</div>
]])
```

移除：

```lua
sl.ui.remove_html("plugin-root")
```

### 3. 注入与操作 CSS / DOM

```lua
sl.ui.inject_css("my-style", [[
  .plugin-panel {
    color: #42b883;
  }
]])

sl.ui.hide(".debug-only")
sl.ui.show(".plugin-panel")
sl.ui.disable("#submit")
sl.ui.enable("#submit")
```

删除样式：

```lua
sl.ui.remove_css("my-style")
```

### 4. 插入 HTML 片段

```lua
sl.ui.insert("append", ".plugin-panel", [[
  <button id="plugin-action">Click</button>
]])
```

允许的 `placement` 值：

- `before`
- `after`
- `prepend`
- `append`

对应校验常量见 [`VALID_INSERT_PLACEMENTS`](./common.rs:8)。

### 5. 设置样式与属性

```lua
sl.ui.set_style(".plugin-panel", {
  color = "#fff",
  backgroundColor = "#1e1e1e",
  padding = "12px"
})

sl.ui.set_attribute("#plugin-action", "data-plugin", "example")
```

删除元素：

```lua
sl.ui.remove("#plugin-action")
```

### 6. 显示 Toast 提示

```lua
sl.ui.toast("success", "保存成功", 2500)
sl.ui.toast("error", "操作失败")
```

说明：

- `duration` 可选，默认值为 `3000` 毫秒，见 [`feedback::register()`](./feedback.rs:20)。

### 7. 注册侧边栏入口

```lua
sl.ui.register_sidebar({
  label = "我的插件",
  icon = "puzzle"
})
```

注销：

```lua
sl.ui.unregister_sidebar()
```

说明：

- `label` 为必填项，缺失时会抛出运行时错误，见 [`sidebar::register()`](./sidebar.rs:11)。
- `icon` 为可选项，不传时默认为空字符串。

### 8. 注册上下文菜单

```lua
sl.ui.register_context_menu("server-list", {
  { id = "open", label = "打开" },
  { id = "delete", label = "删除", icon = "trash" }
})
```

注销：

```lua
sl.ui.unregister_context_menu("server-list")
```

允许的 `context` 值：

- `server-list`
- `console`
- `plugin-list`
- `player-list`
- `global`

对应校验逻辑见 [`validate_context_menu_context()`](./common.rs:76)。

每个菜单项 table 至少需要：

- `id: string`
- `label: string`

可选：

- `icon: string`

### 9. 注册上下文菜单回调

```lua
sl.ui.on_context_menu_click(function()
  print("menu clicked")
end)

sl.ui.on_context_menu_show(function()
  print("menu show")
end)

sl.ui.on_context_menu_hide(function()
  print("menu hide")
end)
```

说明：

- 当前 Rust 层会把回调保存进 Lua registry，见 [`register_callback()`](./common.rs:87)。
- 这些接口当前返回 `true` 表示注册成功。

### 10. 查询组件镜像列表

```lua
local components = sl.ui.component.list()
for i, item in ipairs(components) do
  print(item.id, item.type)
end
```

按页面过滤：

```lua
local components = sl.ui.component.list("home")
```

返回项结构示例：

```lua
{
  { id = "header-1", type = "card" },
  { id = "panel-2", type = "button" }
}
```

### 11. 读取 / 设置 / 调用组件

```lua
sl.ui.component.get("header-1", "title")
sl.ui.component.set("header-1", "title", "新的标题")
sl.ui.component.call("dialog-1", "open")
sl.ui.component.on("dialog-1", "confirm")
```

说明：

- 这些接口本质上是向宿主发送组件事件请求，当前统一通过 [`emit_component_action()`](./common.rs:45) 分发。
- 宿主如何回传结果，取决于上层事件系统；当前 Lua 接口本身主要负责发起请求。

### 12. 创建宿主组件

```lua
sl.ui.component.create("card", "plugin-card", {
  title = "示例卡片",
  collapsible = true,
  order = 10
})
```

说明：

- `props` 会被转换为 JSON 对象，基础支持 `string`、`boolean`、`integer`、`number`、`table` 等值类型，见 [`lua_value_to_json()`](./common.rs:20)。

## 错误处理与返回值语义

大多数 [`sl.ui`](./mod.rs) 写操作接口都遵循以下约定：

| 模式     | 宿主事件发送成功 | 宿主事件发送失败    |
| -------- | ---------------- | ------------------- |
| `compat` | 返回 `true`      | 返回 `false`        |
| `strict` | 返回 `true`      | 抛出 Lua 运行时错误 |

对应实现见：

- [`set_error_mode()`](./common.rs:92)
- [`emit_result()`](./common.rs:112)

需要注意：

- 参数校验类错误不会被降级为 `false`，而是直接抛出运行时错误。
- 例如缺少 `label`、缺少菜单项 `id`、非法 `placement`、非法 `context` 等情况都会立即报错。

## 参数约束

| 限制项                  | 规则                                                                    | 对应实现                                            |
| ----------------------- | ----------------------------------------------------------------------- | --------------------------------------------------- |
| 错误模式                | 仅允许 `compat` 或 `strict`                                             | [`set_error_mode()`](./common.rs:92)                |
| `insert` 的 `placement` | 仅允许 `before`、`after`、`prepend`、`append`                           | [`VALID_INSERT_PLACEMENTS`](./common.rs:8)          |
| 上下文菜单 `context`    | 仅允许 `server-list`、`console`、`plugin-list`、`player-list`、`global` | [`validate_context_menu_context()`](./common.rs:76) |
| 侧边栏配置              | 必须包含 `label` 字段                                                   | [`sidebar::register()`](./sidebar.rs:11)            |
| 菜单项配置              | 必须包含 `id` 与 `label` 字段                                           | [`context_menu::register()`](./context_menu.rs:22)  |

## 备注

- [`sl.ui.query()`](./basic.rs:83) 当前仅负责向宿主发起查询事件，请求结果依赖宿主侧后续处理。
- [`sl.ui.component.get()`](./component.rs:70)、[`sl.ui.component.set()`](./component.rs:94)、[`sl.ui.component.call()`](./component.rs:119)、[`sl.ui.component.on()`](./component.rs:143)、[`sl.ui.component.create()`](./component.rs:167) 当前均属于“事件请求型接口”，不是同步返回宿主执行结果的接口。
- [`sl.ui.on_context_menu_click()`](./context_menu.rs:80) 这类回调注册接口当前只有注册能力，文档中未包含反注册接口，因为当前模块尚未暴露对应 Lua API。
