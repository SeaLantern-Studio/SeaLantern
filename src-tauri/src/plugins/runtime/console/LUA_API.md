# console Lua 接口说明

本文档说明插件运行时暴露的 [`sl.console`](./mod.rs) Lua 接口，用于向指定服务器发送控制台命令、读取控制台日志，以及查询服务器运行状态。

## 接口总览

| Lua 接口                 | 参数                                                      | 返回值    | 说明                                                 | 对应实现                                |
| ------------------------ | --------------------------------------------------------- | --------- | ---------------------------------------------------- | --------------------------------------- |
| `sl.console.send()`      | `serverId: string`，`command: string`                     | `boolean` | 向指定服务器发送一条控制台命令；发送成功返回 `true`  | [`send::send()`](./send.rs:13)          |
| `sl.console.getLogs()`   | `serverId: string`，`offset?: integer`，`count?: integer` | `table`   | 按分页方式读取指定服务器控制台日志，返回结构化结果   | [`logs::get_logs()`](./logs.rs:7)       |
| `sl.console.getStatus()` | `serverId: string`                                        | `string`  | 获取指定服务器当前状态，例如 `running`、`stopped` 等 | [`status::get_status()`](./status.rs:5) |

## 使用说明

### 1. 发送控制台命令

```lua
local ok = sl.console.send("my-server", "say hello from plugin")
print(ok)
```

常见场景：

```lua
sl.console.send("survival", "time set day")
sl.console.send("survival", "whitelist reload")
sl.console.send("survival", "tellraw @a {\"text\":\"Hello\",\"color\":\"green\"}")
```

说明：

- [`sl.console.send()`](./mod.rs:20) 发送前会先校验服务器是否存在。
- 仅允许发送在设置中声明为允许的命令首词，命令匹配规则见 [`is_command_allowed()`](./common.rs:107)。
- 命令会做基础清洗，见 [`sanitize_command()`](./common.rs:81)：
  - 自动去除首尾空白
  - 禁止空命令
  - 禁止包含换行符 `\n` / `\r`
- 该接口最终通过后端写入服务器进程的 stdin，而不是通过 shell 执行，对应实现见 [`ServerManager::send_command()`](../../../services/server/manager.rs:1205)。

### 2. 查询服务器状态

```lua
local status = sl.console.getStatus("my-server")
print(status)
```

可用于在发命令前做状态判断：

```lua
local status = sl.console.getStatus("my-server")
if status == "running" then
  sl.console.send("my-server", "list")
end
```

状态读取逻辑依赖 [`get_server_status_checked()`](./common.rs:66)，会先校验服务器存在，再返回宿主侧状态结果。

### 3. 分页读取控制台日志

```lua
local result = sl.console.getLogs("my-server")
print(result.server_id)
print(result.offset)
print(result.count)
print(result.next_offset)

for i, entry in ipairs(result.logs) do
  print(entry.index, entry.content)
end
```

指定偏移量和数量：

```lua
local result = sl.console.getLogs("my-server", 100, 50)
for i, entry in ipairs(result.logs) do
  print(entry.index, entry.content)
end
```

返回结构示例：

```lua
{
  server_id = "my-server",
  offset = 100,
  count = 50,
  next_offset = 150,
  logs = {
    { index = 100, content = "[12:00:01 INFO]: Done (1.234s)!" },
    { index = 101, content = "[12:00:05 INFO]: Player joined" }
  }
}
```

说明：

- [`sl.console.getLogs()`](./mod.rs:26) 默认读取 `100` 条日志，最大不超过 [`MAX_LOG_COUNT`](./common.rs:28)。
- `offset` 默认值为 `0`。
- 返回值中的 `next_offset` 可直接作为下一次分页读取的起点。
- 实际底层读取来自 [`server_log_pipeline::get_logs()`](../../../services/server/log_pipeline.rs:148)。

## 安全限制

当前 console 接口带有基础安全控制，主要包括：

| 限制项       | 规则                                                   | 对应实现                                                                 |
| ------------ | ------------------------------------------------------ | ------------------------------------------------------------------------ |
| 服务器存在性 | 所有接口都会先校验 `serverId` 是否存在                 | [`validate_server_id()`](./common.rs:56)                                 |
| 状态读取校验 | 读取状态与日志前会先做存在性检查                       | [`get_server_status_checked()`](./common.rs:66)                          |
| 空命令限制   | 命令去首尾空白后不能为空                               | [`sanitize_command()`](./common.rs:81)                                   |
| 换行限制     | 命令中不允许出现 `\n` 或 `\r`                          | [`sanitize_command()`](./common.rs:81)                                   |
| 白名单控制   | 命令首词必须出现在允许命令列表中                       | [`is_command_allowed()`](./common.rs:107)                                |
| 黑名单控制   | 命令首词若出现在阻止命令列表中，则优先拒绝             | [`is_command_allowed()`](./common.rs:107)                                |
| 大小写归一化 | 允许/阻止命令列表会统一做 `ASCII lowercase` 再进行比较 | [`normalized_command_list()`](./common.rs:98)                            |
| 日志读取上限 | 单次日志读取数量最大为 `1000`                          | [`MAX_LOG_COUNT`](./common.rs:28)                                        |
| 发送审计     | 成功、拒绝、失败的发送行为都会进入插件权限审计日志     | [`emit_console_log()`](./common.rs:131) / [`send::send()`](./send.rs:13) |

## 错误行为

当调用失败时，Lua 侧会收到 runtime error。常见失败场景包括：

- 服务器不存在
- 服务器未运行但尝试调用 [`sl.console.send()`](./mod.rs:20)
- 命令为空
- 命令包含换行符
- 命令不在允许列表中
- 命令命中阻止列表
- 后端写入服务器 stdin 失败

相关错误映射辅助函数：

- [`map_console_err()`](./common.rs:30)
- [`runtime_console_err()`](./common.rs:73)
- [`runtime_console_msg()`](./common.rs:77)

## 备注

- [`sl.console.send()`](./mod.rs:20) 只返回是否发送成功，不返回命令执行结果。
- [`sl.console.getStatus()`](./mod.rs:32) 返回的是宿主统一定义的状态字符串，实际来源见 [`ServerStatus::as_str()`](../../../models/server.rs:16)。
- [`sl.console.getLogs()`](./mod.rs:26) 当前每条日志仅返回 `index` 与 `content`，后续如底层暴露更多结构化字段，可继续扩展返回表而不破坏分页语义。
