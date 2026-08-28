# 插件 v2 设计与安全边界

当前插件实现是 Lua 插件 API v2。插件不是主应用业务层的前置依赖；它通过明确的 manifest、能力声明和宿主 dispatcher 接入。

## 代码落位

| 层           | 路径                                                             | 职责                                                                           |
| ------------ | ---------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| 能力契约     | `crates/core/src/app_plugin`                                     | 能力目录、调用请求、scope、trust、风险和策略决策类型；不执行 Lua、不访问持久化 |
| 运行时       | `crates/extra/src/app_plugin`                                    | API v2 manifest 校验、Lua engine、loader、manager、生命周期和插件私有数据边界  |
| 应用层       | `application/src/plugin`                                         | 策略 SQLite、持久/会话授权、审批令牌、审计记录和能力 dispatcher                |
| Desktop 适配 | `src-tauri/src/adapter/tauri/commands/plugin.rs`                 | `plugin_v2_*` Tauri commands                                                   |
| Web 适配     | `server/src/rpc/methods/plugin`、`server/src/rpc/plugin_auth.rs` | `plugin.v2.invoke` RPC 与 Bearer token 认证                                    |

## Manifest

运行时只接受 `apiVersion: 2` 的严格 manifest。顶层至少需要插件身份、版本和入口脚本，能力必须显式声明：

```json
{
  "apiVersion": 2,
  "id": "example.plugin",
  "name": "Example Plugin",
  "version": "1.0.0",
  "main": "main.lua",
  "capabilities": [{ "id": "plugin.log.emit" }, { "id": "plugin.storage.read" }]
}
```

manifest 和能力声明拒绝未知字段；未知的能力 ID、旧 API 版本和已删除的 `permissions` 形状都应在脚本执行前失败。插件 bundle 指纹会在加载时与策略状态协调，不能从 HTTP 请求中接受“我声明过这个能力”的信任。

## 一次能力调用

```text
宿主入口
  -> 解析插件身份与请求
  -> 检查插件已加载/已启用
  -> 按 manifest 确认 capability + scope 已声明
  -> 应用层重新计算策略
       capability / scope / trust / session / approval token
  -> dispatcher 执行有界操作
  -> 记录允许、拒绝或失败的审计结果
```

应用层会重新读取已加载插件和持久化策略，不信任调用方携带的 `declared` 或 `trustSource` 字段。高风险能力需要相应的信任来源；会话授权、一次性审批令牌和持久授权各自有独立生命周期。文件、网络、市场和服务器能力还必须经过 scope、大小/速率/并发等运行时限制。

## 宿主差异

- Desktop 将生命周期、策略管理和能力调用暴露为 `plugin_v2_*` Tauri commands。
- Web 只通过 `POST /api/rpc/plugin/v2/invoke` 暴露当前插件能力调用；认证 token 来自 `SEALANTERN_PLUGIN_RPC_TOKEN`，少于 32 个字符时不会启用认证器。请求必须使用 `Authorization: Bearer <token>`，认证失败返回 `401`。
- 两个宿主都进入同一个 `CorePluginService` 和 dispatcher；宿主不应复制一套授权逻辑。
- 插件不获得完整的 Tauri/Axum Context。需要主应用数据时使用显式的只读 `PluginReadHost` 或已注册能力。

## 变更规则

1. 新能力先在 `crates/core/src/app_plugin/catalog.rs` 定义描述、风险、scope 和限制，再实现 dispatcher。
2. manifest、策略判断、dispatcher 和至少一个宿主入口必须同时更新并测试。
3. 不恢复旧的 Lua API 或隐式权限字段；兼容需求必须先形成独立设计。
4. 不在插件文档中维护一份脱离源码的完整函数清单；能力目录和 manifest 类型是唯一事实来源。

## 源码锚点

- [能力契约](../../crates/core/src/app_plugin)
- [manifest 与 loader](../../crates/extra/src/app_plugin)
- [插件应用服务](../../application/src/plugin)
- [Desktop plugin commands](../../src-tauri/src/adapter/tauri/commands/plugin.rs)
- [Web plugin RPC](../../server/src/rpc/methods/plugin/invoke.rs)
