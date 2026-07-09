## sl.storage

本文档说明插件运行时暴露的 [`sl.storage`](../../backend/tauri-host/src/plugins/runtime/storage.rs) Lua 接口，用于保存插件私有的小型结构化数据。该模块基于单个 JSON 存储文件实现，适合保存设置项、运行状态、缓存元数据等键值数据。

## APIs

- `sl.storage.get(key)`: 按键读取存储值；键不存在时返回 `nil`
- `sl.storage.keys()`: 返回当前所有键名，按字典序排序
- `sl.storage.set(key, value)`: 写入或覆盖一个键值；值会先转换为 JSON 再落盘
- `sl.storage.remove(key)`: 删除指定键；键不存在时静默忽略

## 数据模型

[`sl.storage`](../../backend/tauri-host/src/plugins/runtime/storage.rs) 不是通用文件系统接口，而是一个面向插件状态保存的 JSON 键值存储。其数据模型等价于：

```json
{
  "theme": "dark",
  "window_width": 1280,
  "auto_start": true,
  "server_draft": {
    "name": "Paper 1.21",
    "java": "C:/Java/bin/java.exe",
    "memory": 4096,
    "eula": true
  }
}
```

底层文件路径由 [`StorageContext::new()`](../../backend/tauri-host/src/plugins/runtime/storage/common.rs:21) 构造，默认保存在插件运行时数据目录下的 [`storage.json`](../../backend/tauri-host/src/plugins/runtime/storage/common.rs)。

## 权限模型

[`sl.storage`](../../backend/tauri-host/src/plugins/runtime/storage.rs) 属于运行时内建存储能力，不像 [`sl.fs`](../../backend/tauri-host/src/plugins/runtime/filesystem) 那样区分多级 scope 与文件权限。该模块通过固定存储路径、大小限制和 JSON 数据模型来约束能力边界。

装配入口见 [`setup_storage_namespace()`](../../backend/tauri-host/src/plugins/runtime/storage.rs:17)：

- 创建 [`sl.storage`](../../backend/tauri-host/src/plugins/runtime/storage.rs) 表
- 注册读接口 [`read::register()`](../../backend/tauri-host/src/plugins/runtime/storage/read.rs:7)
- 注册写接口 [`write::register()`](../../backend/tauri-host/src/plugins/runtime/storage/write.rs:8)
- 挂载到 Lua 命名空间 [`sl.storage`](../../backend/tauri-host/src/plugins/runtime/storage.rs)

## 限制与约束

当前 [`sl.storage`](../../backend/tauri-host/src/plugins/runtime/storage.rs) 主要限制如下：

| 限制项        | 规则                                                                                               | 对应实现                                                                                                                                  |
| ------------- | -------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| 键长度限制    | 单个 key 最长 `256` 字节                                                                           | [`MAX_KEY_LENGTH`](../../backend/tauri-host/src/plugins/runtime/storage/common.rs:10)                                                              |
| 值大小限制    | 单个 value 序列化后最大 `1MB`                                                                      | [`MAX_VALUE_SIZE`](../../backend/tauri-host/src/plugins/runtime/storage/common.rs:11)                                                              |
| 总存储限制    | 整个存储文件序列化后最大 `10MB`                                                                    | [`MAX_TOTAL_SIZE`](../../backend/tauri-host/src/plugins/runtime/storage/common.rs:12)                                                              |
| 空 key 禁止   | key 会先执行 [`trim()`](../../backend/tauri-host/src/plugins/runtime/storage/write.rs:23)，空字符串会报错   | [`set()`](../../backend/tauri-host/src/plugins/runtime/storage/write.rs:19)、[`remove()`](../../backend/tauri-host/src/plugins/runtime/storage/write.rs:55) |
| JSON 数据边界 | 仅支持可表示为 JSON 的 Lua 值                                                                      | [`storage_value_from_lua()`](../../backend/tauri-host/src/plugins/runtime/storage/common.rs:104)                                                   |
| 顺序稳定      | [`keys()`](../../backend/tauri-host/src/plugins/runtime/storage/read.rs:32) 返回前会排序                    | [`keys()`](../../backend/tauri-host/src/plugins/runtime/storage/read.rs:32)                                                                        |
| 原子写入      | 先写临时文件，再 [`rename`](../../backend/tauri-host/src/plugins/runtime/storage/common.rs:94) 替换目标文件 | [`write_storage()`](../../backend/tauri-host/src/plugins/runtime/storage/common.rs:80)                                                             |
| 并发串行化    | 读写都受同一把互斥锁保护                                                                           | [`with_storage_lock()`](../../backend/tauri-host/src/plugins/runtime/storage/common.rs:37)                                                         |

## 错误行为

[`sl.storage`](../../backend/tauri-host/src/plugins/runtime/storage.rs) 的读写接口在失败时会抛出 Lua runtime error，而不是返回 `false`。

常见错误场景包括：

- key 为空，见 [`storage.key_empty`](../../backend/i18n/locales/zh-CN.json)
- key 超过长度限制，见 [`storage.key_too_long`](../../backend/i18n/locales/zh-CN.json)
- value 超过大小限制，见 [`storage.value_too_large`](../../backend/i18n/locales/zh-CN.json)
- 总存储超过限制，见 [`storage.total_too_large`](../../backend/i18n/locales/zh-CN.json)
- 存储文件包含非法 JSON，见 [`storage.invalid_json`](../../backend/i18n/locales/zh-CN.json)
- 读取或写入文件失败，见 [`storage.read_failed`](../../backend/i18n/locales/zh-CN.json)、[`storage.write_failed`](../../backend/i18n/locales/zh-CN.json)
- 互斥锁获取失败，见 [`storage.lock_failed`](../../backend/i18n/locales/zh-CN.json)

例如：

```lua
local ok, err = pcall(function()
  sl.storage.set("", "invalid")
end)

print(ok)   -- false
print(err)  -- 存储键不能为空
```

## 与 [`sl.fs`](../../backend/tauri-host/src/plugins/runtime/filesystem) 的区别

[`sl.storage`](../../backend/tauri-host/src/plugins/runtime/storage.rs) 与 [`sl.fs`](../../backend/tauri-host/src/plugins/runtime/filesystem) 的定位不同：

- [`sl.storage`](../../backend/tauri-host/src/plugins/runtime/storage.rs)：保存小型结构化键值数据
- [`sl.fs`](../../backend/tauri-host/src/plugins/runtime/filesystem)：读写真实文件与目录

选择建议：

- 保存插件配置、开关、上次状态、缓存元数据：使用 [`sl.storage`](../../backend/tauri-host/src/plugins/runtime/storage.rs)
- 保存文本文件、二进制文件、模板目录、导入导出内容：使用 [`sl.fs`](../../backend/tauri-host/src/plugins/runtime/filesystem)

## 备注

- [`sl.storage.get()`](../../backend/tauri-host/src/plugins/runtime/storage/read.rs:12) 读取不存在的键时返回 `nil`，不会报错。
- [`sl.storage.set()`](../../backend/tauri-host/src/plugins/runtime/storage/write.rs:13) 对同名 key 会直接覆盖旧值。
- [`sl.storage.remove()`](../../backend/tauri-host/src/plugins/runtime/storage/write.rs:14) 删除不存在的 key 时不会报错。
- 当前模块底层使用单文件 JSON 存储，适合小体量数据，不适合作为大文件或高频大规模数据写入方案。
- 为避免文件损坏，当前写入流程已采用临时文件替换策略，见 [`write_storage()`](../../backend/tauri-host/src/plugins/runtime/storage/common.rs:80)。
