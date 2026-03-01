# 插件 API 文档

本文档列出了所有提供给前端插件的 API 接口，方便前端开发者直接查看和使用。

## 目录

- [Element API](#element-api)
- [Server API](#server-api)
- [Console API](#console-api)
- [FileSystem API](#filesystem-api)

## Element API

| 方法名                                     | 参数                                                                   | 返回值                           | 描述                     |
| ------------------------------------------ | ---------------------------------------------------------------------- | -------------------------------- | ------------------------ |
| `sl.element.get_text(selector)`            | `selector: string` - CSS 选择器                                        | `string` 或 `nil` - 元素文本内容 | 获取指定元素的文本内容   |
| `sl.element.get_value(selector)`           | `selector: string` - CSS 选择器                                        | `string` 或 `nil` - 元素值       | 获取指定元素的值         |
| `sl.element.get_attribute(selector, attr)` | `selector: string` - CSS 选择器<br>`attr: string` - 属性名             | `string` 或 `nil` - 属性值       | 获取指定元素的指定属性值 |
| `sl.element.get_attributes(selector)`      | `selector: string` - CSS 选择器                                        | `object` 或 `nil` - 元素所有属性 | 获取指定元素的所有属性   |
| `sl.element.click(selector)`               | `selector: string` - CSS 选择器                                        | `boolean` - 操作是否成功         | 点击指定元素             |
| `sl.element.set_value(selector, value)`    | `selector: string` - CSS 选择器<br>`value: string` - 要设置的值        | `boolean` - 操作是否成功         | 设置指定元素的值         |
| `sl.element.check(selector, checked)`      | `selector: string` - CSS 选择器<br>`checked: boolean` - 是否选中       | `boolean` - 操作是否成功         | 检查或取消检查指定元素   |
| `sl.element.select(selector, value)`       | `selector: string` - CSS 选择器<br>`value: string` - 要选择的值        | `boolean` - 操作是否成功         | 选择指定元素的选项       |
| `sl.element.focus(selector)`               | `selector: string` - CSS 选择器                                        | `boolean` - 操作是否成功         | 聚焦指定元素             |
| `sl.element.blur(selector)`                | `selector: string` - CSS 选择器                                        | `boolean` - 操作是否成功         | 使指定元素失焦           |
| `sl.element.on_change(selector, callback)` | `selector: string` - CSS 选择器<br>`callback: function` - 变化回调函数 | `function` - 清理函数            | 监听指定元素的变化       |

## Server API

| 方法名                                                    | 参数                                                                                                  | 返回值                           | 描述                       |
| --------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- | -------------------------------- | -------------------------- |
| `sl.server.list()`                                        | 无                                                                                                    | `table` - 服务器列表             | 获取所有服务器列表         |
| `sl.server.get_path(server_id)`                           | `server_id: string` - 服务器 ID                                                                       | `string` - 服务器路径            | 获取指定服务器的路径       |
| `sl.server.read_file(server_id, relative_path)`           | `server_id: string` - 服务器 ID<br>`relative_path: string` - 相对路径                                 | `string` - 文件内容              | 读取服务器文件内容         |
| `sl.server.write_file(server_id, relative_path, content)` | `server_id: string` - 服务器 ID<br>`relative_path: string` - 相对路径<br>`content: string` - 文件内容 | `boolean` - 操作是否成功         | 写入服务器文件             |
| `sl.server.list_dir(server_id, relative_path)`            | `server_id: string` - 服务器 ID<br>`relative_path: string` - 相对路径                                 | `table` - 目录内容列表           | 列出服务器目录内容         |
| `sl.server.exists(server_id, relative_path)`              | `server_id: string` - 服务器 ID<br>`relative_path: string` - 相对路径                                 | `boolean` - 文件是否存在         | 检查服务器文件是否存在     |
| `sl.server.logs.get(server_id, count)`                    | `server_id: string` - 服务器 ID<br>`count: number` - 日志行数 (可选，默认 100)                        | `table` - 日志列表               | 获取指定服务器的日志       |
| `sl.server.logs.getAll(count)`                            | `count: number` - 日志行数 (可选，默认 100)                                                           | `table` - 所有运行中服务器的日志 | 获取所有运行中服务器的日志 |

## Console API

| 方法名                                  | 参数                                                                           | 返回值                              | 描述                 |
| --------------------------------------- | ------------------------------------------------------------------------------ | ----------------------------------- | -------------------- |
| `sl.console.send(server_id, command)`   | `server_id: string` - 服务器 ID<br>`command: string` - 命令内容                | `boolean` - 发送是否成功            | 发送命令到指定服务器 |
| `sl.console.get_logs(server_id, count)` | `server_id: string` - 服务器 ID<br>`count: number` - 日志行数 (可选，默认 100) | `table` - 包含content字段的日志列表 | 获取指定服务器的日志 |
| `sl.console.get_status(server_id)`      | `server_id: string` - 服务器 ID                                                | `string` - 服务器状态               | 获取指定服务器的状态 |

## 权限说明

- `sl.element` 相关 API：需要 `ui` 权限
- `sl.server` 相关 API：需要 `server` 权限
- `sl.console` 相关 API：需要 `console` 权限
- `sl.fs` 相关 API：需要 `fs.data`、`fs.server` 或 `fs.global` 权限
  - `fs.data`：只能访问插件数据目录
  - `fs.server`：只能访问服务器目录
  - `fs.global`：可以访问全局目录

## FileSystem API

| 方法名                             | 参数                                                       | 返回值                            | 描述                 |
| ---------------------------------- | ---------------------------------------------------------- | --------------------------------- | -------------------- |
| `sl.fs.read(path)`                 | `path: string` - 文件路径                                  | `string` - 文件内容               | 读取文件内容         |
| `sl.fs.read_binary(path)`          | `path: string` - 文件路径                                  | `string` - Base64编码的二进制内容 | 读取二进制文件内容   |
| `sl.fs.write(path, content)`       | `path: string` - 文件路径<br>`content: string` - 文件内容  | `nil`                             | 写入文件内容         |
| `sl.fs.exists(path)`               | `path: string` - 文件路径                                  | `boolean` - 文件是否存在          | 检查文件是否存在     |
| `sl.fs.list(path)`                 | `path: string` - 目录路径                                  | `table` - 文件名列表              | 列出目录内容         |
| `sl.fs.mkdir(path)`                | `path: string` - 目录路径                                  | `nil`                             | 创建目录（递归）     |
| `sl.fs.remove(path)`               | `path: string` - 文件或目录路径                            | `nil`                             | 删除文件或目录       |
| `sl.fs.info(path)`                 | `path: string` - 文件或目录路径                            | `table` - 文件信息                | 获取文件信息         |
| `sl.fs.copy(src, dst)`             | `src: string` - 源路径<br>`dst: string` - 目标路径         | `nil`                             | 复制文件或目录       |
| `sl.fs.move(src, dst)`             | `src: string` - 源路径<br>`dst: string` - 目标路径         | `nil`                             | 移动文件或目录       |
| `sl.fs.rename(old_path, new_path)` | `old_path: string` - 旧路径<br>`new_path: string` - 新路径 | `nil`                             | 重命名文件或目录     |
| `sl.fs.get_path(scope)`            | `scope: string` - 作用域 (`data`, `server`, `global`)      | `string` - 路径                   | 获取指定作用域的路径 |

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

### 文件系统操作

```lua
-- 读取文件
local content = sl.fs.read("config.txt")
print("File content: " .. content)

-- 写入文件
sl.fs.write("output.txt", "Hello, world!")

-- 检查文件是否存在
local exists = sl.fs.exists("config.txt")
print("File exists: " .. tostring(exists))

-- 列出目录内容
local files = sl.fs.list("")
for i, file in ipairs(files) do
    print("File " .. i .. ": " .. file)
end
```
