## sl.ui

本文档说明插件运行时暴露的 [`sl.ui`](../../backend/tauri-host/src/plugins/runtime/ui/mod.rs) Lua 接口，用于注入 HTML / CSS、控制页面元素、显示反馈、注册侧边栏与上下文菜单，以及与宿主组件系统交互。

## APIs

- `sl.ui.set_error_mode(mode)`: 设置 UI 接口错误模式，支持 `compat` 与 `strict`
- `sl.ui.inject_html(elementId, html)`: 向指定元素注入 HTML 内容
- `sl.ui.remove_html(elementId)`: 移除指定元素已注入的 HTML 内容
- `sl.ui.update_html(elementId, html)`: 更新指定元素的 HTML 内容
- `sl.ui.query(selector)`: 向宿主发起 UI 查询请求
- `sl.ui.inject_css(styleId, css)`: 注入一段具名 CSS
- `sl.ui.remove_css(styleId)`: 移除此前注入的具名 CSS
- `sl.ui.hide(selector)`: 隐藏匹配选择器的元素
- `sl.ui.show(selector)`: 显示匹配选择器的元素
- `sl.ui.disable(selector)`: 禁用匹配选择器的元素
- `sl.ui.enable(selector)`: 启用匹配选择器的元素
- `sl.ui.insert(placement, selector, html)`: 按位置向目标元素前后或内部插入 HTML
- `sl.ui.remove(selector)`: 删除匹配选择器的元素
- `sl.ui.set_style(selector, styles)`: 批量设置内联样式
- `sl.ui.set_attribute(selector, attr, value)`: 设置元素属性
- `sl.ui.toast(type, message, duration?)`: 显示一条 Toast 提示
- `sl.ui.register_sidebar(config)`: 注册插件侧边栏入口
- `sl.ui.unregister_sidebar()`: 注销当前插件侧边栏入口
- `sl.ui.register_context_menu(context, items)`: 在指定上下文注册插件菜单项
- `sl.ui.unregister_context_menu(context)`: 注销指定上下文下的插件菜单项
- `sl.ui.on_context_menu_click(callback)`: 注册上下文菜单点击回调
- `sl.ui.on_context_menu_show(callback)`: 注册上下文菜单显示回调
- `sl.ui.on_context_menu_hide(callback)`: 注册上下文菜单隐藏回调
- `sl.ui.component.list(pageFilter?)`: 获取当前已镜像的宿主组件列表
- `sl.ui.component.get(componentId, prop)`: 请求读取组件属性
- `sl.ui.component.set(componentId, prop, value)`: 请求设置组件属性
- `sl.ui.component.call(componentId, method)`: 请求调用组件方法
- `sl.ui.component.on(componentId, event)`: 请求订阅组件事件
- `sl.ui.component.create(componentType, componentId, props)`: 请求创建宿主组件

## 错误处理与返回值语义

大多数 [`sl.ui`](../../backend/tauri-host/src/plugins/runtime/ui/mod.rs) 写操作接口都遵循以下约定：

| 模式     | 宿主事件发送成功 | 宿主事件发送失败    |
| -------- | ---------------- | ------------------- |
| `compat` | 返回 `true`      | 返回 `false`        |
| `strict` | 返回 `true`      | 抛出 Lua 运行时错误 |

对应实现见：

- [`set_error_mode()`](../../backend/tauri-host/src/plugins/runtime/ui/common.rs:92)
- [`emit_result()`](../../backend/tauri-host/src/plugins/runtime/ui/common.rs:112)

需要注意：

- 参数校验类错误不会被降级为 `false`，而是直接抛出运行时错误。
- 例如缺少 `label`、缺少菜单项 `id`、非法 `placement`、非法 `context` 等情况都会立即报错。

## 参数约束

| 限制项                  | 规则                                                                    | 对应实现                                                                                 |
| ----------------------- | ----------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| 错误模式                | 仅允许 `compat` 或 `strict`                                             | [`set_error_mode()`](../../backend/tauri-host/src/plugins/runtime/ui/common.rs:92)                |
| `insert` 的 `placement` | 仅允许 `before`、`after`、`prepend`、`append`                           | [`VALID_INSERT_PLACEMENTS`](../../backend/tauri-host/src/plugins/runtime/ui/common.rs:8)          |
| 上下文菜单 `context`    | 仅允许 `server-list`、`console`、`plugin-list`、`player-list`、`global` | [`validate_context_menu_context()`](../../backend/tauri-host/src/plugins/runtime/ui/common.rs:76) |
| 侧边栏配置              | 必须包含 `label` 字段                                                   | [`sidebar::register()`](../../backend/tauri-host/src/plugins/runtime/ui/sidebar.rs:11)            |
| 菜单项配置              | 必须包含 `id` 与 `label` 字段                                           | [`context_menu::register()`](../../backend/tauri-host/src/plugins/runtime/ui/context_menu.rs:22)  |

## 备注

- [`sl.ui.query()`](../../backend/tauri-host/src/plugins/runtime/ui/basic.rs:83) 当前仅负责向宿主发起查询事件，请求结果依赖宿主侧后续处理。
- [`sl.ui.component.get()`](../../backend/tauri-host/src/plugins/runtime/ui/component.rs:70)、[`sl.ui.component.set()`](../../backend/tauri-host/src/plugins/runtime/ui/component.rs:94)、[`sl.ui.component.call()`](../../backend/tauri-host/src/plugins/runtime/ui/component.rs:119)、[`sl.ui.component.on()`](../../backend/tauri-host/src/plugins/runtime/ui/component.rs:143)、[`sl.ui.component.create()`](../../backend/tauri-host/src/plugins/runtime/ui/component.rs:167) 当前均属于"事件请求型接口"，不是同步返回宿主执行结果的接口。
- [`sl.ui.on_context_menu_click()`](../../backend/tauri-host/src/plugins/runtime/ui/context_menu.rs:80) 这类回调注册接口当前只有注册能力，文档中未包含反注册接口，因为当前模块尚未暴露对应 Lua API。
