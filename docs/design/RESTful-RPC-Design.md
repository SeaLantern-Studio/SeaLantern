# RESTful RPC 设计

状态：已采纳的当前基线。本文描述普通业务 API 的目标形态和迁移收尾标准；源码仍处于迁移完成前的过渡状态时，必须明确标注残留位置。

## 决策

SeaLantern 的普通业务使用 RESTful RPC：

1. Web 宿主只有一个 Axum 应用和一个监听入口，由 `server/src/adapter/http/router.rs` 统一组装 SPA、普通业务 API 和插件 API。
2. 普通业务按资源和业务动作组织路径，例如 `/api/instances/{id}`、`/api/instances/{id}/start`、`/api/downloads/{id}`；生命周期动作可以使用动作子路径，不要求为了形式上的 CRUD 强行改写。
3. HTTP 方法、状态码和 JSON 响应表达业务结果。成功响应直接返回类型化 JSON；错误统一为 `{ "code": "...", "message": "..." }`，不使用普通业务通用的 `method_id + JSON` 调度包络。
4. `application/src/port` 定义类型化业务能力端口，`AppServices` 是进程内服务访问 facade；Tauri command 和 Axum handler 都是薄适配器，共用同一组端口和实现。
5. 插件 v2 的 `POST /api/rpc/plugin/v2/invoke` 是有意保留的 RPC 例外。插件能力调用具有动态 capability、scope、trust 和授权令牌语义，必须经过独立 Bearer 认证和应用层策略检查，不能借此为普通业务恢复通用 RPC。

这里的“合二为一”指普通业务的 Web 入口和路由模型合并为同一个 Axum RESTful API；不把 Desktop 的 Tauri IPC 强行改成 HTTP，也不取消插件的专用 RPC 边界。

## 当前状态

- `server/src/main.rs` 只创建一个 `TcpListener`，并把 `build_router` 交给 `axum::serve`。
- `server/src/adapter/http/router.rs` 是生产路由入口，已挂载普通业务 REST 路由和插件 v2 RPC 路由。
- `server/src/rpc/router.rs` 仍保留旧的通用 RPC 方法注册器，当前只被 RPC 模块自己的测试使用，没有被生产入口嵌套。这是迁移残留，不是第二个监听服务，也不是普通业务的目标架构。
- `src/api/invoke.ts` 是前端的统一调用入口，负责 Tauri invoke 与 Axum REST 映射；`src/api/rpc.ts` 没有生产调用方，待调用清理后删除。

## 分层关系

```text
HTTP/Tauri adapter
        │
        ▼
application::port  ───────► contract DTO / error
        │
        ▼
application::service + AppServices
        │
        ├──────────────► core
        ├──────────────► feature
        └──────────────► infra
```

适配器负责解析请求、调用端口、选择传输响应；端口负责业务能力的类型化契约；应用实现负责用例编排和错误收敛；`contract` 不依赖任何具体实现层。

## 收尾标准

完成普通业务 RESTful RPC 迁移需要同时满足：

1. 生产入口不再挂载普通业务的通用 method dispatcher；旧 `server/src/rpc/router.rs` 在没有消费者后删除，或明确限于插件/测试用途。
2. 前端所有需要 Web 的业务都通过 `src/api/invoke.ts` 和已注册 REST 路由访问；未支持能力继续显式返回 `NotImplementedError`。
3. `src/api/rpc.ts` 没有调用方后删除，并同步删除无消费者的映射和类型。
4. 每个新业务端点都有 handler、路由方法/状态码、JSON 契约和至少一个路由级测试；错误不得泄漏底层路径、进程或网络细节。
5. 路由、`application::port`、`contract` DTO 和本目录设计文档在同一变更中更新。

## 源码锚点

- [Web server entry](../../server/src/main.rs)
- [production Axum router](../../server/src/adapter/http/router.rs)
- [legacy generic RPC router](../../server/src/rpc/router.rs)
- [plugin v2 RPC method](../../server/src/rpc/methods/plugin/invoke.rs)
- [frontend unified invocation](../../src/api/invoke.ts)
- [frontend legacy RPC adapter](../../src/api/rpc.ts)
