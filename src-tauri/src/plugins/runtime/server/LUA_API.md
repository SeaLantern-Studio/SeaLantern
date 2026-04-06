# server Lua 接口说明

本文档说明插件运行时暴露的 [`sl.server`](../server.rs) Lua 接口，用于查询服务器实例信息、访问服务器目录中的受限文件能力，以及读取运行日志。

## 接口总览

| Lua 接口                                        | 参数                                                  | 返回值                  | 说明                                                       | 对应实现                               |
| ----------------------------------------------- | ----------------------------------------------------- | ----------------------- | ---------------------------------------------------------- | -------------------------------------- |
| `sl.server.list()`                              | 无                                                    | `table<number, table>`  | 列出当前已注册的服务器实例                                 | [`files::list()`](./files.rs:8)        |
| `sl.server.get_path(serverId)`                  | `serverId: string`                                    | `string`                | 获取指定服务器的根目录路径                                 | [`files::get_path()`](./files.rs:22)   |
| `sl.server.read_file(serverId, path)`           | `serverId: string`，`path: string`                    | `string`                | 读取服务器目录中的文本文件                                 | [`files::read_file()`](./files.rs:31)  |
| `sl.server.write_file(serverId, path, content)` | `serverId: string`，`path: string`，`content: string` | `boolean`               | 向服务器目录写入文本文件，不存在的父目录会自动创建         | [`files::write_file()`](./files.rs:49) |
| `sl.server.list_dir(serverId, path)`            | `serverId: string`，`path: string`                    | `table<number, table>`  | 列出目录下的直接子项及其基础元信息                         | [`files::list_dir()`](./files.rs:75)   |
| `sl.server.exists(serverId, path)`              | `serverId: string`，`path: string`                    | `boolean`               | 判断服务器目录中的文件或目录是否存在                       | [`files::exists()`](./files.rs:120)    |
| `sl.server.logs.get(serverId, count?)`          | `serverId: string`，`count?: integer`                 | `table<number, string>` | 获取指定服务器最近 N 条日志，默认 `100`，最大 `1000`       | [`get()`](./logs.rs:19)                |
| `sl.server.logs.getAll(count?)`                 | `count?: integer`                                     | `table<number, table>`  | 获取所有运行中服务器最近 N 条日志，默认 `100`，最大 `1000` | [`get_all()`](./logs.rs:37)            |

## 使用说明

### 1. 列出服务器实例

```lua
local servers = sl.server.list()
for i, server in ipairs(servers) do
  print(i, server.id, server.name, server.path)
end
```

每个服务器条目包含以下字段，构造逻辑见 [`create_server_entry()`](./common.rs:120)：

- `id`
- `name`
- `path`
- `version`
- `server_type`

### 2. 获取服务器根目录

```lua
local path = sl.server.get_path("my-server")
print(path)
```

### 3. 读取服务器文本文件

```lua
local eula = sl.server.read_file("my-server", "eula.txt")
print(eula)
```

该接口只适用于文本内容读取，读取前会执行文件大小检查，限制见 [`MAX_FILE_SIZE`](./common.rs:10)。

### 4. 写入服务器文本文件

```lua
sl.server.write_file("my-server", "plugins/example/config.yml", "enabled: true\n")
```

如果目标父目录不存在，会自动创建，具体逻辑见 [`files::write_file()`](./files.rs:49)。

### 5. 列出目录内容

```lua
local entries = sl.server.list_dir("my-server", "plugins")
for _, entry in ipairs(entries) do
  print(entry.name, entry.is_dir, entry.size)
end
```

每个目录项包含：

- `name`
- `is_dir`
- `size`

### 6. 检查路径是否存在

```lua
if sl.server.exists("my-server", "server.properties") then
  print("server.properties exists")
end
```

### 7. 获取单个服务器日志

```lua
local logs = sl.server.logs.get("my-server", 50)
for i, line in ipairs(logs) do
  print(i, line)
end
```

### 8. 获取所有运行中服务器日志

```lua
local allLogs = sl.server.logs.getAll(20)
for _, item in ipairs(allLogs) do
  print("server:", item.server_id)
  for _, line in ipairs(item.logs) do
    print(line)
  end
end
```

## 权限模型

所有 [`sl.server`](../server.rs) 接口都要求插件拥有 `server` 权限，校验逻辑见 [`check_server_permission()`](./common.rs:23)。

- 缺少权限时会拒绝执行
- 文件接口与日志接口使用相同权限门禁
- 权限上下文由 [`ServerContext`](./common.rs:13) 持有

## 安全限制

当前 [`sl.server`](../server.rs) 具备基础的路径限制与读写保护，主要包括：

| 限制项           | 规则                                          | 对应实现                                             |
| ---------------- | --------------------------------------------- | ---------------------------------------------------- |
| 权限校验         | 仅允许拥有 `server` 权限的插件调用            | [`check_server_permission()`](./common.rs:23)        |
| 服务器存在性校验 | `serverId` 必须对应已注册服务器               | [`find_server()`](./common.rs:30)                    |
| 路径越界防护     | 拒绝越过服务器根目录的访问                    | [`validated_server_path()`](./common.rs:90)          |
| 路径校验核心     | 统一使用运行时共享路径校验逻辑                | [`validate_server_path()`](../shared.rs:216)         |
| 大文件读取限制   | 读取前检查文件大小，超过 `128 MiB` 会拒绝     | [`checked_file_metadata()`](./common.rs:139)         |
| 目录类型校验     | `list_dir` 仅允许对目录执行，非目录会直接报错 | [`files::list_dir()`](./files.rs:75)                 |
| 日志数量限制     | 日志接口默认返回 `100` 条，最大限制为 `1000`  | [`get()`](./logs.rs:19)、[`get_all()`](./logs.rs:37) |
| 运行中筛选       | `getAll` 仅返回当前运行中服务器的日志         | [`running_log_pairs()`](./common.rs:150)             |

## 备注

- [`sl.server.read_file()`](./files.rs:31) 使用 Rust 的文本读取方式实现，不适合读取二进制内容。
- [`sl.server.write_file()`](./files.rs:49) 当前写入的是完整文本内容，不提供追加写入能力。
- [`sl.server.logs.get()`](./logs.rs:19) 当前返回字符串数组，而不是带结构字段的对象数组。
- [`sl.server.logs.getAll()`](./logs.rs:37) 返回项格式为 `{ server_id = string, logs = table<number, string> }`。
