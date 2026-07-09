## sl.fs

本文档说明插件运行时暴露的 [`sl.fs`](../../backend/tauri-host/src/plugins/runtime/filesystem.rs) Lua 接口，用于在受限 sandbox 内执行文件读写、目录管理、元信息查询与条目转移操作。

## APIs

- `sl.fs.read(scope, path)`: 读取文本文件内容
- `sl.fs.read_binary(scope, path)`: 读取二进制文件并返回 Base64 字符串
- `sl.fs.exists(scope, path)`: 判断文件或目录是否存在
- `sl.fs.list(scope, path)`: 列出目录下的直接子项名称
- `sl.fs.info(scope, path)`: 获取条目元信息，包含 `size`、`is_dir`、`modified`
- `sl.fs.get_path(scope)`: 获取虚拟 sandbox 路径标识，而非宿主真实路径
- `sl.fs.write(scope, path, content)`: 写入文本内容，不存在的父目录会自动创建
- `sl.fs.mkdir(scope, path)`: 创建目录，等价于递归创建
- `sl.fs.remove(scope, path)`: 删除文件或空目录；拒绝递归删除非空目录与 sandbox 根
- `sl.fs.copy(scope, src, dst)`: 在同一 scope 内复制文件或目录；目标已存在时拒绝覆盖
- `sl.fs.move(scope, src, dst)`: 在同一 scope 内移动文件或目录
- `sl.fs.rename(scope, oldPath, newPath)`: 在同一 scope 内重命名文件或目录

## scope 说明

所有 [`sl.fs`](../../backend/tauri-host/src/plugins/runtime/filesystem.rs) 接口都要求显式传入 `scope`，当前支持以下三个值：

| scope    | 含义             | 对应目录来源                                                                          | 权限前缀    |
| -------- | ---------------- | ------------------------------------------------------------------------------------- | ----------- |
| `data`   | 插件私有数据目录 | [`FsContext.data_dir`](../../backend/tauri-host/src/plugins/runtime/filesystem/common.rs:9)    | `fs.data`   |
| `server` | 当前服务器目录   | [`FsContext.server_dir`](../../backend/tauri-host/src/plugins/runtime/filesystem/common.rs:10) | `fs.server` |
| `global` | 全局共享目录     | [`FsContext.global_dir`](../../backend/tauri-host/src/plugins/runtime/filesystem/common.rs:11) | `fs.global` |

无效 scope 会被 [`resolve_scope_action()`](../../backend/tauri-host/src/plugins/runtime/filesystem/common.rs:92) 拒绝。

## 权限模型

文件系统接口采用 "scope + action" 的组合权限模型，旧的 scope 级权限仍兼容。

### 兼容权限

以下权限表示对应 scope 的全部文件系统能力：

- `fs.data`
- `fs.server`
- `fs.global`

### 细粒度 action 权限

| action     | 允许的接口                   | 示例权限           |
| ---------- | ---------------------------- | ------------------ |
| `read`     | `read`、`read_binary`        | `fs.data.read`     |
| `meta`     | `exists`、`info`、`get_path` | `fs.data.meta`     |
| `list`     | `list`                       | `fs.data.list`     |
| `write`    | `write`、`mkdir`             | `fs.data.write`    |
| `delete`   | `remove`                     | `fs.data.delete`   |
| `transfer` | `copy`、`move`、`rename`     | `fs.data.transfer` |

权限判定由 [`resolve_scope_action()`](../../backend/tauri-host/src/plugins/runtime/filesystem/common.rs:92) 执行；若插件同时持有 `fs.data` 与 `fs.data.read`，则会按"允许"处理。

## 安全限制

当前 [`sl.fs`](../../backend/tauri-host/src/plugins/runtime/filesystem.rs) 带有基础 sandbox 与防破坏保护，主要包括：

| 限制项                   | 规则                                               | 对应实现                                                                                           |
| ------------------------ | -------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| 路径校验                 | 拒绝绝对路径、`..` 跳转与越界访问                  | [`validate_fs_path()`](../../backend/tauri-host/src/plugins/runtime/filesystem/common.rs:122)               |
| scope 校验               | 仅允许 `data`、`server`、`global`                  | [`resolve_scope_permission()`](../../backend/tauri-host/src/plugins/runtime/filesystem/common.rs:83)        |
| action 权限校验          | 需要对应的 `fs.<scope>.<action>` 或兼容 scope 权限 | [`resolve_scope_action()`](../../backend/tauri-host/src/plugins/runtime/filesystem/common.rs:92)            |
| 符号链接 / reparse point | 拒绝通过符号链接访问或穿透 sandbox                 | [`ensure_safe_path_for_access()`](../../backend/tauri-host/src/plugins/runtime/filesystem/common.rs:150)    |
| 目录树安全检查           | 写入/复制目标父目录会逐级检查是否为安全目录树      | [`ensure_safe_directory_tree()`](../../backend/tauri-host/src/plugins/runtime/filesystem/common.rs:139)     |
| 删除 sandbox 根目录      | 明确拒绝删除 scope 根目录                          | [`reject_dangerous_remove_target()`](../../backend/tauri-host/src/plugins/runtime/filesystem/common.rs:155) |
| 删除非空目录             | 不允许递归删除非空目录                             | [`write::remove()`](../../backend/tauri-host/src/plugins/runtime/filesystem/write.rs:63)                    |
| 复制覆盖                 | 目标已存在时拒绝复制，避免破坏已有内容             | [`copy_dir_recursive()`](../../backend/tauri-host/src/plugins/runtime/filesystem/common.rs:177)             |
| 真实路径暴露             | `get_path` 只返回 `sandbox://...` 虚拟路径         | [`read::get_path()`](../../backend/tauri-host/src/plugins/runtime/filesystem/read.rs:144)                   |

## 备注

- [`sl.fs.copy()`](../../backend/tauri-host/src/plugins/runtime/filesystem/transfer.rs:8)、[`sl.fs.move()`](../../backend/tauri-host/src/plugins/runtime/filesystem/transfer.rs:47)、[`sl.fs.rename()`](../../backend/tauri-host/src/plugins/runtime/filesystem/transfer.rs:77) 当前仅支持在同一 `scope` 内操作，不提供跨 scope 传输。
- [`sl.fs.list()`](../../backend/tauri-host/src/plugins/runtime/filesystem/read.rs:78) 返回的是目录直接子项名称，不包含递归结果。
- [`sl.fs.read_binary()`](../../backend/tauri-host/src/plugins/runtime/filesystem/read.rs:31) 返回 Base64 字符串，Lua 侧如需原始字节需自行解码。
- `fs` 旧权限会在运行时被规范化为 `fs.data`，用于兼容旧插件配置，相关逻辑位于 [`PluginRuntime::new()`](../../backend/tauri-host/src/plugins/runtime/core/setup.rs:6)。
