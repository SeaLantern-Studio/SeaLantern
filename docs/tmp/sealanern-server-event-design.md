# server 宿主事件推送（WebSocket）对接说明 v3

> v3 修订：游标语义精确定义（R1）、应用层 ping 改为必需（R2）、首次连接纳入初始化（R3）、补漏失败兜底（R4）、两宿主补漏通道与参数名差异（R5）、连接状态暴露（R6）、多标签页限制（R7）。

## 1. 端点

```
GET /api/events/ws      （升级为 WebSocket）
ws://127.0.0.1:3000/api/events/ws
```

- 连接后**服务端立即推送**，无需订阅请求
- 推送是**全量**的（所有实例混流），前端按 `instance_id` 过滤
- 当前**无鉴权**

## 2. 消息格式

**事件**（带 `event` 字段）：

```json
{
  "event": "server-log-line",
  "instance_id": "e7e7e34d-0dcd-41d8-8576-80e1ba88712d",
  "line": {
    "sequence": 1201,
    "timestamp": 1789788161,
    "source": "server",
    "line": "Done (1.557s)!"
  }
}
```

**控制消息**（带 `type` 字段）：

```json
{ "type": "lagged", "skipped": 37 }
```

```json
{ "type": "pong" }
```

**客户端 → 服务端**：

```json
{ "type": "ping" }
```

### 判别规则（按顺序）

1. 有 `event` → 事件，用 `msg.event` 分发
2. 否则有 `type` → 控制消息
3. 都没有 → **忽略**（前向兼容）

## 3. 事件清单

| event             | 触发           | 负载                  | 可补漏 |
| ----------------- | -------------- | --------------------- | ------ |
| `server-log-line` | 进程输出落库后 | `instance_id`、`line` | 是     |

`line`（`ConsoleLogLine`）：

| 字段        | 类型   | 说明                                       |
| ----------- | ------ | ------------------------------------------ |
| `sequence`  | number | 行号，**单调递增**（补漏与去重的唯一依据） |
| `timestamp` | number | Unix **秒**（渲染 ×1000）                  |
| `source`    | string | `"server"` / `"sealantern"`                |
| `line`      | string | 文本                                       |

**同一连接上的事件按 `sequence` 单调递增到达**（单连接 FIFO，不会乱序）。

## 4. 心跳与连接活性（应用层 ping 为必需）

服务端在**无日志输出时不推送任何消息**，且协议层 Ping 对 JS 不可见——**因此前端必须自己做应用层心跳**：

```
每 30s 发送 { "type": "ping" }
收到任意消息（含 { "type": "pong" } 与事件）→ 刷新 lastActiveAt
lastActiveAt 距今 > 75s → 主动 close() 并进入重连流程
```

服务端会回 `{ "type": "pong" }`；协议层 Ping（30s）由服务端自动发送以保活中间设备，前端无需关心。

## 5. 补漏

> 事件是**瞬时通知，不保证送达**。

触发补漏的两种情况：

1. 收到 `{ "type": "lagged", "skipped": N }`
2. **连接建立（首次或重连）**

### 5.1 游标定义（关键）

**只用"已交付边界"，绝不使用"收到过的最大值"。**

```ts
/** 已按序交付给消费方的最后一条 sequence */
const deliveredSeq = new Map<string, number>();
```

| 用途         | 规则                                |
| ------------ | ----------------------------------- |
| 补漏传参     | `since = deliveredSeq.get(id) ?? 0` |
| 去重边界     | `sequence <= deliveredSeq` → 丢弃   |
| 缓冲中的新行 | **不推进** `deliveredSeq`           |

**为什么不能用"收到的最大值"**：

```
断线前 delivered=99；断线期间产生 100~124
重连后实时先到 125、126（进缓冲）
若用"收到的最大值"126 补漏 → since=126 → 100~124 永久丢失
若用 126 做去重边界     → 补回的 100~124 全被误删
```

### 5.2 连接建立（首次或重连）的初始化流程

```
1. WS 连接建立
2. 实时事件进入「待合并缓冲」（不直接吐给消费方）
3. REST 拉取 since = deliveredSeq（首次该实例无游标 → since=0）
4. 以 deliveredSeq 为界，按 sequence 归并去重（历史 + 缓冲）
5. 按序吐出，deliveredSeq = 实际吐出的最后一条
6. 切换到直通模式，后续事件直接吐出
```

**首次连接**同样走这个流程（否则只能看到连上之后的日志）。首次若不想要全部历史，可用窗口参数只取最近 N 行：

```
GET /api/instances/{id}/logs?since=0&limit=200      // 最近 200 行窗口
```

### 5.3 补漏接口的两个宿主

| 宿主   | 调用                                                          | 参数名                  |
| ------ | ------------------------------------------------------------- | ----------------------- |
| server | `GET /api/instances/{id}/logs?since=<deliveredSeq>&limit=<N>` | `since`、`limit`        |
| Tauri  | `invoke("get_server_logs", { id, since, recent_limit })`      | `since`、`recent_limit` |

**语义相同，通道不同**：两宿主共用同一个应用层 `ConsoleService::logs`，`since` 均为 **exclusive**（只返回 `sequence > since`）。

⚠️ 参数名不同（`limit` vs `recent_limit`），宿主无关封装需做映射。

### 5.4 `limit` 的真实语义（注意）

`limit` / `recent_limit` 是**"最近 N 行窗口"**，**不是分页条数**：

```
GET logs?since=99&limit=200
  → 「sequence > 99」且「落在最近 200 行窗口内」  （不是「取 200 条」）
```

所以长时间断线后补漏**可能返回超大数组，且无法分页**。建议：

- 缺口不大 → 直接补
- 缺口很大 → 放弃补全，标记「存在缺口」状态，正常展示新日志

### 5.5 补漏失败兜底

```
补漏请求失败 → 保留缓冲并重试（建议 3 次，指数退避）
仍失败       → 记录缺口、降级为直通模式，连接状态标记「存在缺口」
```

### 5.6 `lagged` 的范围约定

`lagged` 是**连接级**提示，**不含实例信息**（服务端无法得知被跳过的是哪些实例——广播通道不保留被丢弃的消息）。

约定：收到 `lagged` 后，**对当前活跃实例补漏即可**（其他实例不渲染，无需补）。

## 6. 与 Tauri 宿主的对应

|        | Tauri 宿主                      | server 宿主                               |
| ------ | ------------------------------- | ----------------------------------------- |
| 订阅   | `listen("server-log-line", cb)` | `ws.onmessage` 按 `msg.event` 分发        |
| 事件名 | `"server-log-line"`             | 同                                        |
| 负载   | `{ instance_id, line }`         | 同（+ 顶层 `event`）                      |
| 心跳   | 无（无长连接）                  | 应用层 ping **必需**（第 4 节）           |
| 补漏   | `invoke("get_server_logs", …)`  | `GET /logs?since=…`（语义相同，通道不同） |

**注意**：WS 是**单连接多路复用**，不能"每个事件各开一条连接"。`core/events` 的浏览器实现应是**应用级单例连接管理器 + 按事件名分发**：

```text
subscribeServerEvents({
  onEvent: (eventName, payload) => { ... },
  onLagged: (skipped) => { ... },
  onStateChange: (state) => { ... },   // connecting | open | reconnecting | closed | gap
}): { close(): void }
```

## 7. 实现要点清单

- [ ] 连接：`new WebSocket(base.replace(/^http/, "ws") + "/api/events/ws")`（`https`→`wss` 自然成立）
- [ ] 分发：按第 2 节顺序判别（`event` → `type` → 忽略）
- [ ] 游标：`Map<instanceId, number>` **仅存"已交付边界"**（第 5.1 节）
- [ ] 归并：初始化阶段用缓冲 + 按 `sequence` 归并去重（第 5.2 节）
- [ ] 心跳：每 30s 发 `{"type":"ping"}`；75s 无消息主动重连（第 4 节）
- [ ] 重连：指数退避 + **jitter**（1s→2s→…上限 30s）
- [ ] 补漏失败：重试 3 次后降级并标记缺口（第 5.5 节）
- [ ] **暴露连接状态**：`connecting / open / reconnecting / closed / gap`（供 UI 提示"实时连接已断开，正在重连"）
- [ ] **主动关闭标志**：组件卸载后禁止重连逻辑继续运行，并 `close()` 连接
- [ ] JSON 解析 `try/catch`，失败不断连
- [ ] 未知 `event` 忽略（前向兼容）
- [ ] 过滤在**最外层**做（入库前按 `instance_id` 丢弃，不要先塞进响应式数组）
- [ ] `source: "sealantern"` 复用现有 `[Sea Lantern]` 系统行样式
- [ ] `timestamp` 秒 → 毫秒

## 8. 已知限制

- **无鉴权**：后续接入时连接方式（query / 子协议）会变化
- **无订阅协议**：全量推送，前端过滤（多实例需按最坏情况实现）
- **无服务端回放**：补漏一律走查询接口
- **补漏无分页**：`limit` 是"最近 N 行窗口"语义，大缺口只能截断并标记（第 5.4 节）
- **未做跨标签页连接复用**：每个浏览器标签页各建立一条 WS，服务端需按标签页数承受连接量
- **无持久化的事件**：`server-log-line` 有日志库可补；将来若加状态类事件（不落库），丢失后应重新调用对应 REST 接口重建视图
