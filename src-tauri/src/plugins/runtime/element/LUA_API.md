# element Lua 接口说明

本文档说明插件运行时暴露的 [`sl.element`](../element.rs) Lua 接口，用于按 CSS 选择器查询页面元素状态、触发常见交互操作，以及订阅元素变化事件。

## 接口总览

| Lua 接口                                          | 参数                                                                            | 返回值          | 说明                                                                  | 对应实现                        |
| ------------------------------------------------- | ------------------------------------------------------------------------------- | --------------- | --------------------------------------------------------------------- | ------------------------------- |
| `sl.element.get_text(selector)`                   | `selector: string`                                                              | `string \| nil` | 获取匹配元素的文本内容；查询失败、超时或未找到时返回 `nil`            | [`register()`](./query.rs:82)   |
| `sl.element.get_value(selector)`                  | `selector: string`                                                              | `string \| nil` | 获取输入类元素的当前值；查询失败、超时或未找到时返回 `nil`            | [`register()`](./query.rs:82)   |
| `sl.element.exists(selector)`                     | `selector: string`                                                              | `string \| nil` | 判断元素是否存在，返回字符串 `"true"` / `"false"`，超时返回 `nil`     | [`register()`](./query.rs:82)   |
| `sl.element.is_visible(selector)`                 | `selector: string`                                                              | `string \| nil` | 判断元素是否可见，返回字符串 `"true"` / `"false"`，超时返回 `nil`     | [`register()`](./query.rs:82)   |
| `sl.element.is_enabled(selector)`                 | `selector: string`                                                              | `string \| nil` | 判断元素是否启用，返回字符串 `"true"` / `"false"`，超时返回 `nil`     | [`register()`](./query.rs:82)   |
| `sl.element.get_attribute(selector, attr)`        | `selector: string`，`attr: string`                                              | `string \| nil` | 获取元素指定属性值；查询失败、超时或未找到时返回 `nil`                | [`register()`](./query.rs:150)  |
| `sl.element.get_attributes(selector)`             | `selector: string`                                                              | `table \| nil`  | 获取元素全部属性，返回 Lua table；查询失败或超时时返回 `nil`          | [`register()`](./query.rs:178)  |
| `sl.element.click(selector)`                      | `selector: string`                                                              | `boolean`       | 触发元素点击行为                                                      | [`register()`](./mutate.rs:39)  |
| `sl.element.set_value(selector, value)`           | `selector: string`，`value: string`                                             | `boolean`       | 设置输入类元素的值，并派发 `input` / `change` 事件                    | [`register()`](./mutate.rs:39)  |
| `sl.element.check(selector, checked)`             | `selector: string`，`checked: boolean`                                          | `boolean`       | 设置复选框 / 单选框选中状态，并派发 `change` 事件                     | [`register()`](./mutate.rs:39)  |
| `sl.element.select(selector, value)`              | `selector: string`，`value: string`                                             | `boolean`       | 设置下拉框选中值，并派发 `change` 事件                                | [`register()`](./mutate.rs:39)  |
| `sl.element.set_attribute(selector, attr, value)` | `selector: string`，`attr: string`，`value: string \| number \| boolean \| nil` | `boolean`       | 批量设置匹配元素的单个属性，`nil` 会被写为空字符串                    | [`register()`](./mutate.rs:114) |
| `sl.element.set_style(selector, styles)`          | `selector: string`，`styles: table`                                             | `boolean`       | 批量设置匹配元素样式，支持 Lua table 转 JSON 后下发前端               | [`register()`](./mutate.rs:141) |
| `sl.element.form_fill(selector, fields)`          | `selector: string`，`fields: table`                                             | `boolean`       | 按字段名批量填写表单，支持 `input` / `textarea` / `select` / 复选框组 | [`register()`](./mutate.rs:162) |
| `sl.element.focus(selector)`                      | `selector: string`                                                              | `boolean`       | 让元素获得焦点                                                        | [`register()`](./mutate.rs:182) |
| `sl.element.blur(selector)`                       | `selector: string`                                                              | `boolean`       | 让元素失去焦点                                                        | [`register()`](./mutate.rs:200) |
| `sl.element.on_change(selector, callback)`        | `selector: string`，`callback: function`                                        | `function`      | 监听元素 `change` 事件，返回 cleanup 函数用于解除监听                 | [`register()`](./watch.rs:7)    |

## 使用说明

### 1. 读取元素文本

```lua
local text = sl.element.get_text("#server-name")
if text ~= nil then
  print("server name:", text)
end
```

同步等待逻辑由 [`wait_for_element_response()`](./common.rs:48) 处理，超时时间由 [`ELEMENT_GET_TIMEOUT_MS`](./common.rs:6) 控制，当前为 `500ms`。

### 2. 读取输入框值

```lua
local value = sl.element.get_value("input[name='javaPath']")
print("current value:", value)
```

### 3. 判断元素是否存在 / 可见 / 可用

```lua
print(sl.element.exists("#server-name"))
print(sl.element.is_visible("#server-name"))
print(sl.element.is_enabled("#server-name"))
```

这些接口当前返回字符串 `"true"` / `"false"`，与 [`wait_for_element_response()`](./common.rs:44) 的现有返回协议保持一致。

### 4. 读取单个属性

```lua
local placeholder = sl.element.get_attribute("#download-url", "placeholder")
print("placeholder:", placeholder)
```

### 5. 读取全部属性

```lua
local attrs = sl.element.get_attributes("#download-url")
if attrs ~= nil then
  print(attrs.id)
  print(attrs.placeholder)
end
```

该接口会把前端返回的 JSON 对象转换为 Lua table，转换逻辑见 [`json_string_to_lua_value()`](./common.rs:72) 与 [`json_to_lua_value()`](./common.rs:77)。

### 6. 点击元素

```lua
local ok = sl.element.click("button.submit")
print("click result:", ok)
```

### 7. 设置输入框内容

```lua
sl.element.set_value("#server-name", "Fabric 1.21")
```

前端侧会同步派发 `input` 与 `change` 事件，具体处理见 [`pluginStore.ts`](../../../src/stores/pluginStore.ts:1022)。

### 8. 设置勾选状态

```lua
sl.element.check("#agree-eula", true)
```

### 9. 设置下拉框值

```lua
sl.element.select("#server-core", "paper")
```

### 10. 设置属性与样式

```lua
sl.element.set_attribute("#server-name", "data-plugin-mark", true)
sl.element.set_style("#server-name", {
  borderColor = "#42b883",
  backgroundColor = "rgba(66, 184, 131, 0.12)",
})
```

其中 [`set_attribute()`](./mutate.rs:115) 支持标量值；[`set_style()`](./mutate.rs:142) 会将 Lua table 转为 JSON 后由前端批量写入样式。

### 11. 高阶表单填写

```lua
sl.element.form_fill("form#server-form", {
  serverName = "Paper 1.21",
  javaPath = "C:/Java/bin/java.exe",
  agreeEula = true,
  serverType = "paper",
})
```

高阶填表由前端 [`"element_form_fill"`](../../../src/stores/pluginStore.ts:1220) 处理，按字段 `name` 匹配元素，支持：

- 普通 `input`
- `textarea`
- `select`
- `checkbox`
- `radio`
- checkbox 多选数组

### 12. 焦点控制

```lua
sl.element.focus("#server-name")
sl.element.blur("#server-name")
```

### 13. 监听元素变化

```lua
local cleanup = sl.element.on_change("#server-name", function(value)
  print("new value:", value)
end)

-- 不再需要时释放监听
cleanup()
```

监听回调会存入 Lua registry，key 生成逻辑见 [`element_watch_registry_key()`](./common.rs:13)。cleanup 函数除了移除 Lua 侧回调外，还会通知前端执行 [`"element_off_change"`](../../../src/stores/pluginStore.ts:1115) 解除 DOM 监听。

## 权限模型

所有 [`sl.element`](../element.rs) 接口都要求插件拥有 `element` 权限，装配逻辑见 [`setup_sl_namespace()`](../core/setup.rs:57)。

- 拥有 `element` 权限时，会注册真实的 [`sl.element`](../element.rs) 接口
- 缺少权限时，会回退到权限拒绝模块，具体逻辑见 [`setup_permission_denied_module()`](../core/permissions.rs:17)
- 所有已注册接口都会记录权限日志，调用点见 [`emit_permission_log()`](../../../src-tauri/src/plugins/api.rs:407)

## 安全限制

当前 [`sl.element`](../element.rs) 主要通过权限门禁、同步等待上限和事件桥接边界进行约束：

| 限制项       | 规则                                                       | 对应实现                                                                                  |
| ------------ | ---------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| 权限校验     | 仅允许拥有 `element` 权限的插件调用                        | [`setup_sl_namespace()`](../core/setup.rs:57)                                             |
| 查询超时     | 查询类接口最长同步等待 `500ms`，超时返回 `nil`             | [`ELEMENT_GET_TIMEOUT_MS`](./common.rs:6)                                                 |
| 查询返回桥接 | 查询通过请求 ID + channel 等待前端返回                     | [`emit_query()`](./query.rs:20)                                                           |
| 值转换限制   | `get_attributes` 仅支持 JSON 可表示的数据类型转换为 Lua 值 | [`json_to_lua_value()`](./common.rs:77)                                                   |
| 事件边界     | 实际 DOM 访问与操作都通过 UI 事件下发到前端处理            | [`emit_ui_event()`](../../../src-tauri/src/plugins/api.rs:220)                            |
| 监听清理     | cleanup 时同时清理 Lua registry 与前端 DOM `change` 监听   | [`register()`](./watch.rs:7)、[`pluginStore.ts`](../../../src/stores/pluginStore.ts:1115) |

## 备注

- [`sl.element.get_text()`](./query.rs:85) / [`sl.element.get_value()`](./query.rs:98) / [`sl.element.exists()`](./query.rs:111) / [`sl.element.is_visible()`](./query.rs:124) / [`sl.element.is_enabled()`](./query.rs:137) / [`sl.element.get_attribute()`](./query.rs:150) 在失败、超时、前端未响应等场景下统一返回 `nil`。
- [`sl.element.exists()`](./query.rs:111) / [`sl.element.is_visible()`](./query.rs:124) / [`sl.element.is_enabled()`](./query.rs:137) 当前返回的是字符串 `"true"` / `"false"`，这是为了兼容现有查询响应桥接协议。
- [`sl.element.click()`](./mutate.rs:41) 等修改类接口不会抛出 Lua 异常，而是记录错误后返回 `false`。
- [`sl.element.form_fill()`](./mutate.rs:163) 当前属于高阶 helper，仍挂载在 [`sl.element`](../element.rs) 下，而不是独立 [`sl.form`](../core/setup.rs:57) 命名空间。
- [`sl.element.on_change()`](./watch.rs:13) 当前监听的是 DOM `change` 事件，不是更高频的 `input` 事件。
- 当前选择器语义完全依赖前端 [`document.querySelector`](../../../src/stores/pluginStore.ts:964) / [`document.querySelectorAll`](../../../src/stores/pluginStore.ts:924) 的行为。
