---
title: sl.console Namespace
description: "本文档说明插件运行时暴露的 [`sl.console`](../../backend/tauri-host/src/plugins/runtime/console/mod.rs) Lua 接口，用于向指定服务器发送控制台命令、读取控制台日志，以及查询服务器运行状态。"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "console"]
author: Codex
---

## sl.console

本文档说明插件运行时暴露的 [`sl.console`](../../../backend/tauri-host/src/plugins/runtime/console/mod.rs) Lua 接口，用于向指定服务器发送控制台命令、读取控制台日志，以及查询服务器运行状态。

## APIs

- [sl.console.send()](./send.md): 向指定服务器发送一条控制台命令；发送成功返回 `true`
- [sl.console.getLogs()](./get-logs.md): 按分页方式读取指定服务器控制台日志，返回结构化结果
- [sl.console.getStatus()](./get-status.md): 获取指定服务器当前状态，例如 `running`、`stopped` 等

## 安全限制

当前 console 接口带有基础安全控制，主要包括：

| 限制项       | 规则                                                   | 对应实现                                                                                                                                                     |
| ------------ | ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 服务器存在性 | 所有接口都会先校验 `serverId` 是否存在                 | [`validate_server_id()`](../../../backend/tauri-host/src/plugins/runtime/console/common.rs:56)                                                                           |
| 状态读取校验 | 读取状态与日志前会先做存在性检查                       | [`get_server_status_checked()`](../../../backend/tauri-host/src/plugins/runtime/console/common.rs:66)                                                                    |
| 空命令限制   | 命令去首尾空白后不能为空                               | [`sanitize_command()`](../../../backend/tauri-host/src/plugins/runtime/console/common.rs:81)                                                                             |
| 换行限制     | 命令中不允许出现 `\n` 或 `\r`                          | [`sanitize_command()`](../../../backend/tauri-host/src/plugins/runtime/console/common.rs:81)                                                                             |
| 白名单控制   | 命令首词必须出现在允许命令列表中                       | [`is_command_allowed()`](../../../backend/tauri-host/src/plugins/runtime/console/common.rs:107)                                                                          |
| 黑名单控制   | 命令首词若出现在阻止命令列表中，则优先拒绝             | [`is_command_allowed()`](../../../backend/tauri-host/src/plugins/runtime/console/common.rs:107)                                                                          |
| 大小写归一化 | 允许/阻止命令列表会统一做 `ASCII lowercase` 再进行比较 | [`normalized_command_list()`](../../../backend/tauri-host/src/plugins/runtime/console/common.rs:98)                                                                      |
| 日志读取上限 | 单次日志读取数量最大为 `1000`                          | [`MAX_LOG_COUNT`](../../../backend/tauri-host/src/plugins/runtime/console/common.rs:28)                                                                                  |
| 发送审计     | 成功、拒绝、失败的发送行为都会进入插件权限审计日志     | [`emit_console_log()`](../../../backend/tauri-host/src/plugins/runtime/console/common.rs:131) / [`send::send()`](../../../backend/tauri-host/src/plugins/runtime/console/send.rs:13) |

## 错误行为

当调用失败时，Lua 侧会收到 runtime error。常见失败场景包括：

- 服务器不存在
- 服务器未运行但尝试调用 [`sl.console.send()`](../../../backend/tauri-host/src/plugins/runtime/console/mod.rs:20)
- 命令为空
- 命令包含换行符
- 命令不在允许列表中
- 命令命中阻止列表
- 后端写入服务器 stdin 失败

> 这里的允许/阻止名单只作用于 `sl.console.send()` 的控制台命令首词，不作用于 `sl.process.exec()`。后者使用 `execute_program` 权限 + `manifest.json` 中 `programs` 声明的独立边界。

相关错误映射辅助函数：

- [`map_console_err()`](../../../backend/tauri-host/src/plugins/runtime/console/common.rs:30)
- [`runtime_console_err()`](../../../backend/tauri-host/src/plugins/runtime/console/common.rs:73)
- [`runtime_console_msg()`](../../../backend/tauri-host/src/plugins/runtime/console/common.rs:77)

## 备注

- [`sl.console.send()`](../../../backend/tauri-host/src/plugins/runtime/console/mod.rs:20) 只返回是否发送成功，不返回命令执行结果。
- [`sl.console.getStatus()`](../../../backend/tauri-host/src/plugins/runtime/console/mod.rs:32) 返回的是宿主统一定义的状态字符串，实际来源见 [`ServerStatus::as_str()`](../../../backend/tauri-host/src/models/server.rs:16)。
- [`sl.console.getLogs()`](../../../backend/tauri-host/src/plugins/runtime/console/mod.rs:26) 当前每条日志仅返回 `index` 与 `content`，后续如底层暴露更多结构化字段，可继续扩展返回表而不破坏分页语义。
