---
title: sl.server Namespace
description: "本文档说明插件运行时暴露的 [`sl.server`](../../backend/tauri-host/src/plugins/runtime/server.rs) Lua 接口，用于查询服务器实例信息、访问服务器目录中的受限文件能力，以及读取运行日志。"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "server"]
author: Codex
---

## sl.server

本文档说明插件运行时暴露的 [`sl.server`](../../../backend/tauri-host/src/plugins/runtime/server.rs) Lua 接口，用于查询服务器实例信息、访问服务器目录中的受限文件能力，以及读取运行日志。

## APIs

- [sl.server.list()](./list.md): 列出当前已注册的服务器实例
- [sl.server.get_path(serverId)](./get-path.md): 获取指定服务器的根目录路径
- [sl.server.read_file(serverId, path)](./read-file.md): 读取服务器目录中的文本文件
- [sl.server.write_file(serverId, path, content)](./write-file.md): 向服务器目录写入文本文件，不存在的父目录会自动创建
- [sl.server.list_dir(serverId, path)](./list-dir.md): 列出目录下的直接子项及其基础元信息
- [sl.server.exists(serverId, path)](./exists.md): 判断服务器目录中的文件或目录是否存在
- [sl.server.logs.get(serverId, count?)](./logs-get.md): 获取指定服务器最近 N 条日志，默认 `100`，最大 `1000`
- [sl.server.logs.getAll(count?)](./logs-get-all.md): 获取所有运行中服务器最近 N 条日志，默认 `100`，最大 `1000`

## 权限模型

所有 [`sl.server`](../../../backend/tauri-host/src/plugins/runtime/server.rs) 接口都要求插件拥有 `server` 权限，校验逻辑见 [`check_server_permission()`](../../../backend/tauri-host/src/plugins/runtime/server/common.rs:23)。

- 缺少权限时会拒绝执行
- 文件接口与日志接口使用相同权限门禁
- 权限上下文由 [`ServerContext`](../../../backend/tauri-host/src/plugins/runtime/server/common.rs:13) 持有

## 安全限制

当前 [`sl.server`](../../../backend/tauri-host/src/plugins/runtime/server.rs) 具备基础的路径限制与读写保护，主要包括：

| 限制项           | 规则                                          | 对应实现                                                                                                                               |
| ---------------- | --------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| 权限校验         | 仅允许拥有 `server` 权限的插件调用            | [`check_server_permission()`](../../../backend/tauri-host/src/plugins/runtime/server/common.rs:23)                                                 |
| 服务器存在性校验 | `serverId` 必须对应已注册服务器               | [`find_server()`](../../../backend/tauri-host/src/plugins/runtime/server/common.rs:30)                                                             |
| 路径越界防护     | 拒绝越过服务器根目录的访问                    | [`validated_server_path()`](../../../backend/tauri-host/src/plugins/runtime/server/common.rs:90)                                                   |
| 路径校验核心     | 统一使用运行时共享路径校验逻辑                | [`validate_server_path()`](../../../backend/tauri-host/src/plugins/runtime/shared.rs:216)                                                          |
| 大文件读取限制   | 读取前检查文件大小，超过 `128 MiB` 会拒绝     | [`checked_file_metadata()`](../../../backend/tauri-host/src/plugins/runtime/server/common.rs:139)                                                  |
| 目录类型校验     | `list_dir` 仅允许对目录执行，非目录会直接报错 | [`files::list_dir()`](../../../backend/tauri-host/src/plugins/runtime/server/files.rs:75)                                                          |
| 日志数量限制     | 日志接口默认返回 `100` 条，最大限制为 `1000`  | [`get()`](../../../backend/tauri-host/src/plugins/runtime/server/logs.rs:19)、[`get_all()`](../../../backend/tauri-host/src/plugins/runtime/server/logs.rs:37) |
| 运行中筛选       | `getAll` 仅返回当前运行中服务器的日志         | [`running_log_pairs()`](../../../backend/tauri-host/src/plugins/runtime/server/common.rs:150)                                                      |

## 备注

- [`sl.server.read_file()`](../../../backend/tauri-host/src/plugins/runtime/server/files.rs:31) 使用 Rust 的文本读取方式实现，不适合读取二进制内容。
- [`sl.server.write_file()`](../../../backend/tauri-host/src/plugins/runtime/server/files.rs:49) 当前写入的是完整文本内容，不提供追加写入能力。
- [`sl.server.logs.get()`](../../../backend/tauri-host/src/plugins/runtime/server/logs.rs:19) 当前返回字符串数组，而不是带结构字段的对象数组。
- [`sl.server.logs.getAll()`](../../../backend/tauri-host/src/plugins/runtime/server/logs.rs:37) 返回项格式为 `{ server_id = string, logs = table<number, string> }`。
