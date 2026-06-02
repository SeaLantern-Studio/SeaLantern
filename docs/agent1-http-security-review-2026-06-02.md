# Agent 1 审查记录（2026-06-02）

范围限定在 `src-tauri/src/runtime/*`、`src-tauri/src/adapters/http/server.rs`、`src-tauri/src/adapters/http/mod.rs` 以及相关测试挂载点；本轮不修改 `src-tauri/src/adapters/http/command_registry/*`，只给出可直接落地的收口方案。

## 结论

当前 headless HTTP 链路的默认行为仍然是“外网可达 + 无鉴权 + 宽松跨域 + 高权限命令全量暴露”，风险已经是远程管理面直接暴露，而不是单点配置偏宽松。

从现有代码看，最小可落地修正不需要改 `command_registry`：

1. 把 headless HTTP 默认绑定从 `0.0.0.0:3000` 收紧到 `127.0.0.1:3000`。
2. 只在显式配置时才允许外网监听。
3. 在 HTTP 层统一加 bearer token 鉴权，至少保护 `/api/*`、`/upload`、`/api/logs/stream`。
4. 把 CORS 从 `Any` 收紧为“默认关闭跨域，只有显式 origin 白名单才放行”。

这样可以在不碰命令注册表的前提下，先把当前最危险的暴露面切断。

## 已确认问题

### 1. 默认绑定地址直接暴露到所有网卡

- 证据：`src-tauri/src/runtime/mode.rs:23-30` 在检测到 `/.dockerenv` 后，直接返回 `bind_addr: "0.0.0.0:3000"`。
- 影响：只要容器端口映射存在，HTTP 管理面就天然对容器外可见。
- 对照：CLI Web 侧已经有“显式外网绑定、默认本地访问”的先例，`src-tauri/src/utils/cli/cli_env.rs:38-45` 默认仅在 headless 场景下推导 `0.0.0.0`，并允许通过 `SEALANTERN_WEB_BIND` 覆盖；而 `runtime/mode.rs` 这条启动链路没有复用该安全边界。

### 2. HTTP 层没有鉴权边界

- 证据：`src-tauri/src/adapters/http/server.rs:119-127` 直接挂出了 `/api/{command}`、`/upload`、`/api/logs/stream` 等路由，没有任何鉴权层。
- 证据：`src-tauri/src/adapters/http/server.rs:266-297` 的 `handle_api_command()` 只做命令查找和执行，没有 bearer token、session 或来源校验。
- 影响：任何能连到端口的请求方，都能直接调用后端命令、上传文件、订阅日志。

### 3. CORS 处于完全放开状态

- 证据：`src-tauri/src/adapters/http/server.rs:114-117` 使用 `CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)`。
- 影响：一旦服务对外可达，浏览器上下文中的任意站点都能直接向该管理面发起跨域请求；在“无鉴权”前提下，这等于把管理接口暴露给任意网页脚本。

### 4. 高危命令默认全量注册

- 证据：`src-tauri/src/adapters/http/command_registry/registry.rs:12-22` 初始化时默认注册 server、plugin、settings、tunnel、update 等命令集合。
- 证据：`src-tauri/src/adapters/http/command_registry/server.rs:12-41` 中明确包含 `create_server`、`delete_server`、`send_command`、`copy_directory_contents`、`add_existing_server` 等高危操作。
- 影响：由于上层 HTTP 没有鉴权，当前不是“接口很多”，而是“高权限操作默认匿名开放”。
- 备注：本轮边界不允许修改 `command_registry/*`，因此第一阶段应先在 HTTP 入口统一加鉴权，第二阶段再做命令分级暴露。

### 5. 启动与配置边界存在分裂，后续容易继续漂移

- 证据：`src-tauri/src/runtime/mod.rs:16-18` 直接把 `RuntimeMode::detect()` 返回的 `bind_addr` 传给 `run_headless_http()`。
- 证据：`src-tauri/src/adapters/http/server.rs:159-162` 会把最终监听地址和接口 URL 打印到日志中。
- 证据：CLI Web 传输链路 `src-tauri/src/utils/cli/server_transport.rs:87-113` 已经支持显式绑定地址、启动状态通知和浏览器访问地址渲染，但 headless runtime 并没有共享同一套绑定策略。
- 影响：绑定策略、安全策略和启动日志分散在两条链路里，后续修补很容易只修 CLI Web 或只修 runtime 其中一边。

## 最小修正方案

目标是不编辑 `command_registry/*` 的前提下，先把匿名远程管理面关掉。

### A. 在 `runtime/*` 收敛 headless HTTP 配置来源

建议修改：

- `src-tauri/src/runtime/mode.rs`
- `src-tauri/src/runtime/mod.rs`

建议做法：

1. 为 headless HTTP 引入显式配置解析，而不是在 `detect()` 里硬编码 `0.0.0.0:3000`。
2. 默认地址改为 `127.0.0.1:3000`。
3. 只有在显式环境变量存在时才允许外网绑定，例如复用 `SEALANTERN_WEB_BIND`，或新增更明确的 `SEALANTERN_HTTP_BIND`。
4. 在 runtime 启动时生成一次随机 bearer token；若环境变量显式提供 token，则优先使用配置值，避免容器重启后外部编排无法接入。
5. 把“绑定地址 + 允许的 origin + bearer token”作为一个显式的 HTTP 安全配置对象传给 `run_http_server()`，不要继续靠散落的环境读取拼行为。

推荐的第一阶段策略：

- 默认：`127.0.0.1:3000`
- 显式外网：仅当 `SEALANTERN_WEB_BIND` 或 `SEALANTERN_HTTP_BIND` 被设置时才允许 `0.0.0.0`
- 默认 token：启动期随机生成并打印“仅首段 + 指纹”，不要把完整 token 原样写进长期日志

### B. 在 `server.rs` 加统一鉴权中间层

建议修改：

- `src-tauri/src/adapters/http/server.rs`
- `src-tauri/src/adapters/http/mod.rs`

建议做法：

1. 新增一个 HTTP 共享安全配置结构，例如：
   - bearer token
   - 可选 origin 白名单
   - 是否允许匿名访问健康检查
2. 只保留 `/health` 为匿名可访问。
3. 对以下路由统一加 `Authorization: Bearer <token>` 校验：
   - `/api/{command}`
   - `/api/list`
   - `/upload`
   - `/api/logs/stream`
4. 未带 token 或 token 不匹配时直接返回 `401 Unauthorized`，不要进入命令查找和业务执行。

这一步已经能把 `create_server`、`delete_server`、`send_command` 等高危能力挡在 HTTP 边界外，不需要先动命令注册表。

### C. 把 CORS 从“全放开”改成“默认不放行”

建议修改：

- `src-tauri/src/adapters/http/server.rs`

建议做法：

1. 默认不要使用 `allow_origin(Any)`。
2. 如果没有配置允许的 WebUI origin，则不要主动放开浏览器跨域访问。
3. 如果配置了 origin 白名单，则仅放行明确的 origin 列表，方法也只保留实际需要的 `GET`、`POST`、`OPTIONS`。
4. 请求头至少只允许 `content-type`、`authorization`。

考虑到本轮不能重构 HTTP 服务层，第一阶段完全可以把 CORS 收紧成“仅显式 origin 白名单可跨域”。即使未来有外置 WebUI，也应由部署方显式声明来源，而不是默认 `Any`。

### D. 危险命令分层建议放到第二阶段

虽然最终应该按风险等级拆分命令暴露面，但在当前文件边界下不建议第一阶段强行在 `server.rs` 里硬编码命令黑名单，原因是：

1. 这样会把命令分级知识再复制一份，继续制造策略漂移。
2. 统一鉴权先到位之后，匿名远程调用面已经被切断，风险会明显下降。

因此更稳妥的顺序是：

1. 先加统一鉴权和默认 loopback 绑定。
2. 再在允许编辑 `command_registry/*` 的后续轮次里把命令定义按安全级别分层。

## 建议补的测试

现有仓库没有看到针对 `run_http_server()` 安全边界的单测挂载。可以新增：

- `src-tauri/tests/unit/adapters_http_server_security_tests.rs`

并在以下位置之一挂载测试模块：

- `src-tauri/src/adapters/http/mod.rs`
- 或 `src-tauri/src/adapters/http/server.rs`

建议覆盖这些场景：

### 1. 运行时绑定策略

- `RuntimeMode::detect()` 在容器环境下默认返回 `127.0.0.1:3000`，而不是 `0.0.0.0:3000`。
- 显式配置 `SEALANTERN_WEB_BIND=0.0.0.0` 或新的 `SEALANTERN_HTTP_BIND` 后，才允许外网绑定。
- 未设置静态目录时不影响安全配置生成。

### 2. 鉴权边界

- `GET /health` 无 token 仍然返回 `200`。
- `GET /api/list` 无 token 返回 `401`。
- `POST /api/nonexistent` 无 token 返回 `401`，而不是 `404`。
- `POST /api/nonexistent` 携带正确 token 时返回 `404`，证明鉴权在命令分发前生效。
- `GET /api/logs/stream` 无 token 返回 `401`。
- `POST /upload` 无 token 返回 `401`。

### 3. CORS 策略

- 未配置允许 origin 时，跨域预检请求不应得到放行的 `access-control-allow-origin`。
- 配置单个允许 origin 后，匹配来源可以通过，不匹配来源被拒绝或不返回允许头。
- `authorization` 头在允许列表内，其余无关请求头默认不放行。

### 4. 启动日志与运维提示

- 启动日志不应输出完整 bearer token。
- 若启用了随机 token，应至少输出“如何传入 Authorization 头”的运维提示。

## 建议改动文件

如果下一轮要按本审查结果直接落代码，建议只动这些文件：

- `src-tauri/src/runtime/mode.rs`
- `src-tauri/src/runtime/mod.rs`
- `src-tauri/src/adapters/http/server.rs`
- `src-tauri/src/adapters/http/mod.rs`
- `src-tauri/tests/unit/adapters_http_server_security_tests.rs`（新增）

这样可以保持在 Agent1 的边界内完成第一阶段安全收口，不与 `command_registry/*` 或上传实现修复产生冲突。

## 优先级建议

1. 先改默认绑定为 loopback，并加 bearer token 鉴权。
2. 然后收紧 CORS。
3. 最后再做命令分级暴露和更细的 HTTP 服务接口拆分。

第一步完成后，当前“匿名远程管理面”这个最高风险问题就会被实质性压下去。
