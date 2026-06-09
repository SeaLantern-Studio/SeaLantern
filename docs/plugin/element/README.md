---
title: sl.element Namespace
description: "本文档说明插件运行时暴露的 [`sl.element`](../../backend/tauri-host/src/plugins/runtime/element.rs) Lua 接口，用于按 CSS 选择器查询页面元素状态、触发常见交互操作，以及订阅元素变化事件。"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "element"]
author: Codex
---

## sl.element

本文档说明插件运行时暴露的 [`sl.element`](../../../backend/tauri-host/src/plugins/runtime/element.rs) Lua 接口，用于按 CSS 选择器查询页面元素状态、触发常见交互操作，以及订阅元素变化事件。

## APIs

- [sl.element.get_text(selector)](./get-text.md): 获取匹配元素的文本内容；查询失败、超时或未找到时返回 `nil`
- [sl.element.get_value(selector)](./get-value.md): 获取输入类元素的当前值；查询失败、超时或未找到时返回 `nil`
- [sl.element.exists(selector)](./exists.md): 判断元素是否存在，返回字符串 `"true"` / `"false"`，超时返回 `nil`
- [sl.element.is_visible(selector)](./is-visible.md): 判断元素是否可见，返回字符串 `"true"` / `"false"`，超时返回 `nil`
- [sl.element.is_enabled(selector)](./is-enabled.md): 判断元素是否启用，返回字符串 `"true"` / `"false"`，超时返回 `nil`
- [sl.element.get_attribute(selector, attr)](./get-attribute.md): 获取元素指定属性值；查询失败、超时或未找到时返回 `nil`
- [sl.element.get_attributes(selector)](./get-attributes.md): 获取元素全部属性，返回 Lua table；查询失败或超时时返回 `nil`
- [sl.element.click(selector)](./click.md): 触发元素点击行为
- [sl.element.set_value(selector, value)](./set-value.md): 设置输入类元素的值，并派发 `input` / `change` 事件
- [sl.element.check(selector, checked)](./check.md): 设置复选框 / 单选框选中状态，并派发 `change` 事件
- [sl.element.select(selector, value)](./select.md): 设置下拉框选中值，并派发 `change` 事件
- [sl.element.set_attribute(selector, attr, value)](./set-attribute.md): 批量设置匹配元素的单个属性，`nil` 会被写为空字符串
- [sl.element.set_style(selector, styles)](./set-style.md): 批量设置匹配元素样式，支持 Lua table 转 JSON 后下发前端
- [sl.element.form_fill(selector, fields)](./form-fill.md): 按字段名批量填写表单，支持 `input` / `textarea` / `select` / 复选框组
- [sl.element.focus(selector)](./focus.md): 让元素获得焦点
- [sl.element.blur(selector)](./blur.md): 让元素失去焦点
- [sl.element.on_change(selector, callback)](./on-change.md): 监听元素 `change` 事件，返回 cleanup 函数用于解除监听

## 权限模型

所有 [`sl.element`](../../../backend/tauri-host/src/plugins/runtime/element.rs) 接口都要求插件拥有 `element` 权限，装配逻辑见 [`setup_sl_namespace()`](../../../backend/tauri-host/src/plugins/runtime/core/setup.rs:57)。

- 拥有 `element` 权限时，会注册真实的 [`sl.element`](../../../backend/tauri-host/src/plugins/runtime/element.rs) 接口
- 缺少权限时，会回退到权限拒绝模块，具体逻辑见 [`setup_permission_denied_module()`](../../../backend/tauri-host/src/plugins/runtime/core/permissions.rs:17)
- 所有已注册接口都会记录权限日志，调用点见 [`emit_permission_log()`](../../../backend/tauri-host/src/plugins/api.rs:407)

## 安全限制

当前 [`sl.element`](../../../backend/tauri-host/src/plugins/runtime/element.rs) 主要通过权限门禁、同步等待上限和事件桥接边界进行约束：

| 限制项       | 规则                                                       | 对应实现                                                                                                                         |
| ------------ | ---------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| 权限校验     | 仅允许拥有 `element` 权限的插件调用                        | [`setup_sl_namespace()`](../../../backend/tauri-host/src/plugins/runtime/core/setup.rs:57)                                                   |
| 查询超时     | 查询类接口最长同步等待 `500ms`，超时返回 `nil`             | [`ELEMENT_GET_TIMEOUT_MS`](../../../backend/tauri-host/src/plugins/runtime/element/common.rs:6)                                              |
| 查询返回桥接 | 查询通过请求 ID + channel 等待前端返回                     | [`emit_query()`](../../../backend/tauri-host/src/plugins/runtime/element/query.rs:20)                                                        |
| 值转换限制   | `get_attributes` 仅支持 JSON 可表示的数据类型转换为 Lua 值 | [`json_to_lua_value()`](../../../backend/tauri-host/src/plugins/runtime/element/common.rs:77)                                                |
| 事件边界     | 实际 DOM 访问与操作都通过 UI 事件下发到前端处理            | [`emit_ui_event()`](../../../backend/tauri-host/src/plugins/api.rs:220)                                                                      |
| 监听清理     | cleanup 时同时清理 Lua registry 与前端 DOM `change` 监听   | [`register()`](../../../backend/tauri-host/src/plugins/runtime/element/watch.rs:7)、[`pluginStore.ts`](../../../frontend/src/stores/pluginStore.ts:1115) |

## 备注

- [`sl.element.get_text()`](../../../backend/tauri-host/src/plugins/runtime/element/query.rs:85) / [`sl.element.get_value()`](../../../backend/tauri-host/src/plugins/runtime/element/query.rs:98) / [`sl.element.exists()`](../../../backend/tauri-host/src/plugins/runtime/element/query.rs:111) / [`sl.element.is_visible()`](../../../backend/tauri-host/src/plugins/runtime/element/query.rs:124) / [`sl.element.is_enabled()`](../../../backend/tauri-host/src/plugins/runtime/element/query.rs:137) / [`sl.element.get_attribute()`](../../../backend/tauri-host/src/plugins/runtime/element/query.rs:150) 在失败、超时、前端未响应等场景下统一返回 `nil`。
- [`sl.element.exists()`](../../../backend/tauri-host/src/plugins/runtime/element/query.rs:111) / [`sl.element.is_visible()`](../../../backend/tauri-host/src/plugins/runtime/element/query.rs:124) / [`sl.element.is_enabled()`](../../../backend/tauri-host/src/plugins/runtime/element/query.rs:137) 当前返回的是字符串 `"true"` / `"false"`，这是为了兼容现有查询响应桥接协议。
- [`sl.element.click()`](../../../backend/tauri-host/src/plugins/runtime/element/mutate.rs:41) 等修改类接口不会抛出 Lua 异常，而是记录错误后返回 `false`。
- [`sl.element.form_fill()`](../../../backend/tauri-host/src/plugins/runtime/element/mutate.rs:163) 当前属于高阶 helper，仍挂载在 [`sl.element`](../../../backend/tauri-host/src/plugins/runtime/element.rs) 下，而不是独立 [`sl.form`](../../../backend/tauri-host/src/plugins/runtime/core/setup.rs:57) 命名空间。
- [`sl.element.on_change()`](../../../backend/tauri-host/src/plugins/runtime/element/watch.rs:13) 当前监听的是 DOM `change` 事件，不是更高频的 `input` 事件。
- 当前选择器语义完全依赖前端 [`document.querySelector`](../../../frontend/src/stores/pluginStore.ts:964) / [`document.querySelectorAll`](../../../frontend/src/stores/pluginStore.ts:924) 的行为。
