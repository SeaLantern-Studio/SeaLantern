# log Lua 接口说明

本文档说明插件运行时暴露的 [`sl.log`](../log.rs) Lua 接口，用于向 SeaLantern 运行时输出插件日志，并同步发送日志事件。

## 接口总览

| Lua 接口         | 参数              | 返回值 | 说明                                                 | 对应实现                               |
| ---------------- | ----------------- | ------ | ---------------------------------------------------- | -------------------------------------- |
| `sl.log.debug()` | `message: string` | `nil`  | 输出调试日志；当插件缺少日志权限时，该调用会静默忽略 | [`create_log_function()`](./emit.rs:4) |
| `sl.log.info()`  | `message: string` | `nil`  | 输出信息日志，并发送对应日志事件                     | [`create_log_function()`](./emit.rs:4) |
| `sl.log.warn()`  | `message: string` | `nil`  | 输出警告日志，并发送对应日志事件                     | [`create_log_function()`](./emit.rs:4) |
| `sl.log.error()` | `message: string` | `nil`  | 输出错误日志，并发送对应日志事件                     | [`create_log_function()`](./emit.rs:4) |

## 使用说明

### 1. 输出调试日志

```lua
sl.log.debug("plugin initialized")
```

当插件拥有日志权限时，会输出一条 `DEBUG` 级别日志；否则该调用会直接返回，不产生任何输出。权限分支注册逻辑见 [`PluginRuntime::setup_log_namespace()`](./mod.rs:10)。

### 2. 输出信息日志

```lua
sl.log.info("download task started")
```

该接口适合记录普通运行信息。日志消息会先转换为 Rust 字符串，再执行输出与事件发送，转换逻辑见 [`convert_lua_string()`](./common.rs:20)。

### 3. 输出警告日志

```lua
sl.log.warn("config file is missing, using default values")
```

警告日志适合表示可恢复问题或降级行为。

### 4. 输出错误日志

```lua
sl.log.error("failed to parse manifest")
```

错误日志适合表示当前操作失败或关键异常。

## 输出行为

当前 [`sl.log`](../log.rs) 在每次成功调用时会执行两类动作：

1. 向标准输出打印格式化日志
2. 通过事件机制发送日志记录

控制台打印格式如下，具体实现见 [`create_log_function()`](./emit.rs:4)：

```text
[LEVEL] [plugin_id] message
```

其中：

- `LEVEL` 为大写日志级别，例如 `INFO`、`WARN`
- `plugin_id` 为当前插件标识，来源于 [`LogContext`](./common.rs:5)
- `message` 为 Lua 侧传入的字符串内容

## 权限模型

[`sl.log`](../log.rs) 的四个接口并非完全相同：

- [`sl.log.debug()`](./emit.rs:4) 受 `log` 权限控制
- [`sl.log.info()`](./emit.rs:4) 不受该权限限制
- [`sl.log.warn()`](./emit.rs:4) 不受该权限限制
- [`sl.log.error()`](./emit.rs:4) 不受该权限限制

权限开关由 [`PluginRuntime::setup_log_namespace()`](./mod.rs:10) 的 `has_log_permission` 参数决定：

- 为 `true` 时，`debug` 会注册为真实日志函数
- 为 `false` 时，`debug` 仍存在，但调用后不会输出任何内容
- `info`、`warn`、`error` 始终可用

## 限制与注意事项

| 限制项       | 规则                                                      | 对应实现                                              |
| ------------ | --------------------------------------------------------- | ----------------------------------------------------- |
| 参数类型     | 当前仅接受 Lua 字符串参数，未直接支持 number/table 等类型 | [`create_log_function()`](./emit.rs:15)               |
| debug 权限   | 无 `log` 权限时，`sl.log.debug()` 会静默忽略              | [`PluginRuntime::setup_log_namespace()`](./mod.rs:10) |
| 输出副作用   | 日志会同时打印到标准输出，并发送运行时日志事件            | [`create_log_function()`](./emit.rs:21)               |
| 命名空间注入 | 最终通过 `sl.log = table` 的方式注册到插件运行时          | [`set_log_table()`](./common.rs:32)                   |

## 备注

- [`sl.log`](../log.rs) 当前不返回日志对象或状态值，调用成功时统一返回 `nil`。
- [`sl.log.debug()`](./emit.rs:4) 在无权限时采用“静默忽略”策略，而不是抛出异常。
- [`sl.log.info()`](./emit.rs:4)、[`sl.log.warn()`](./emit.rs:4)、[`sl.log.error()`](./emit.rs:4) 当前都共享同一套日志构造逻辑，仅日志级别不同。
