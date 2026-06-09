---
title: Plugin Lua API Docs
description: "按命名空间和单个 API 拆分后的插件 Lua API 文档入口。"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "index"]
author: Codex
---

## Plugin Lua API Docs

该目录按命名空间拆分 Sea Lantern 插件 Lua API 文档。每个子目录包含一个命名空间索引页，以及对应 API 的独立说明页。

## Categories

- [sl.console](./console/README.md): 3 个 API。本文档说明插件运行时暴露的 [`sl.console`](../../backend/tauri-host/src/plugins/runtime/console/mod.rs) Lua 接口，用于向指定服务器发送控制台命令、读取控制台日志，以及查询服务器运行状态。
- [sl.element](./element/README.md): 17 个 API。本文档说明插件运行时暴露的 [`sl.element`](../../backend/tauri-host/src/plugins/runtime/element.rs) Lua 接口，用于按 CSS 选择器查询页面元素状态、触发常见交互操作，以及订阅元素变化事件。
- [sl.fs](./filesystem/README.md): 12 个 API。本文档说明插件运行时暴露的 [`sl.fs`](../../backend/tauri-host/src/plugins/runtime/filesystem.rs) Lua 接口，用于在受限 sandbox 内执行文件读写、目录管理、元信息查询与条目转移操作。
- [sl.http](./http/README.md): 4 个 API。本文档说明插件运行时暴露的 [`sl.http`](../../backend/tauri-host/src/plugins/runtime/http/mod.rs) Lua 接口，用于在插件中发起受限的 HTTP 网络请求访问外部互联网资源。
- [sl.i18n](./i18n/README.md): 13 个 API。本文档说明插件运行时暴露的 [`sl.i18n`](../../backend/tauri-host/src/plugins/runtime/i18n/mod.rs) Lua 接口，用于查询当前语言、读取翻译、监听语言切换，以及注册插件自己的国际化资源。
- [sl.log](./log/README.md): 4 个 API。本文档说明插件运行时暴露的 [`sl.log`](../../backend/tauri-host/src/plugins/runtime/log.rs) Lua 接口，用于向 SeaLantern 运行时输出插件日志，并同步发送日志事件。
- [sl.server](./server/README.md): 8 个 API。本文档说明插件运行时暴露的 [`sl.server`](../../backend/tauri-host/src/plugins/runtime/server.rs) Lua 接口，用于查询服务器实例信息、访问服务器目录中的受限文件能力，以及读取运行日志。
- [sl.storage](./storage/README.md): 4 个 API。本文档说明插件运行时暴露的 [`sl.storage`](../../backend/tauri-host/src/plugins/runtime/storage.rs) Lua 接口，用于保存插件私有的小型结构化数据。该模块基于单个 JSON 存储文件实现，适合保存设置项、运行状态、缓存元数据等键值数据。
- [sl.ui](./ui/README.md): 29 个 API。本文档说明插件运行时暴露的 [`sl.ui`](../../backend/tauri-host/src/plugins/runtime/ui/mod.rs) Lua 接口，用于注入 HTML / CSS、控制页面元素、显示反馈、注册侧边栏与上下文菜单，以及与宿主组件系统交互。

## Notes

- 模板文件保留在 [`.example.md`](./.example.md)。
- 原始聚合文档目前仍保留在 [`docs/lua-api/`](../lua-api/) 作为历史参考来源。
