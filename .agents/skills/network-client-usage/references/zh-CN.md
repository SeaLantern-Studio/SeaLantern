---
name: network-client-usage
description: 引导在 crates/application 层统一使用进程级全局网络客户端（global_client / ClientProvider）与下载管理器（DownloadManager / CoreDownloadService）的规范。使用场景：新增联网的业务服务、改造现有服务使其跟随全局代理、需要创建/查询/取消下载任务、判断某个位置应该直接获取客户端还是注入 provider、审查是否违反“请求时获取、不缓存固定客户端、插件客户端独立”等约定。Also use when writing or reviewing Rust code that touches sealantern_infra::net or sealantern_infra::download so the producer stays on the global proxy path.
---

# 全局网络客户端与下载管理器使用指南（中文版）

> 本文件为中文参考版，主入口见 `../SKILL.md`（英文版）。

本仓库的进程级网络能力由 `crates/infra` 统一管理。代理策略（Adaptive / Preserve / Manual / Disabled）由
上层设置与系统代理轮询写入全局网络运行时，业务层只负责“读取当时的全局客户端”并发起请求。

## 核心原则

1. **唯一来源**：生产代码只从 `global_client()` 或 `global_client_provider()` 获取客户端，禁止自行
   `NetClient::from_config` / `reqwest::Client::builder`。
2. **请求时获取**：每次发请求（或每次创建下载任务）前获取一次客户端；服务构造时不得缓存固定 `NetClient`。
3. **注入 provider，不注入 client**：需要依赖注入的服务，传入 `ClientProvider`（函数式获取器），由测试注入假 provider。
4. **插件例外**：插件专用网络必须使用独立 `PluginNetworkClient`（DNS/IP 固定、SSRF 防护、请求头白名单），不要迁移到全局客户端。
5. **错误传播**：`global_client()` 返回 `Result<NetClient, NetError>`，用 `?` 传播，禁止 `unwrap`/`expect`。

## 1. 如何使用全局客户端

### 直接获取（简单调用）

```rust
use sealantern_infra::net::global_client;

async fn fetch_text(url: &str) -> Result<String, NetError> {
    let client = global_client()?; // 每次请求前获取当前全局客户端
    let response = client.get(url)?.send().await?;
    Ok(response.text().await.unwrap_or_default())
}
```

关键类型与函数：

```rust
// crates/infra/src/net/runtime.rs
pub fn global_client() -> Result<NetClient, NetError>;                    // 获取当前全局客户端
pub type ClientProvider = Box<dyn Fn() -> Result<NetClient, NetError> + Send + Sync>;
pub fn global_client_provider() -> ClientProvider;                         // 默认 provider
```

`NetClient` 是廉价 clone（内部持有 `reqwest::Client` 与重试策略），每次返回的都是“当前代理策略”对应的客户端。
代理设置或系统代理变化导致运行时重建客户端后，下一次 `global_client()` 自动拿到新客户端。

### 服务注入 provider（推荐用于长生命周期服务）

```rust
use std::sync::Arc;
use sealantern_infra::net::{global_client_provider, ClientProvider, NetClient, NetError};

struct MarketService {
    client_provider: ClientProvider,
}

impl MarketService {
    fn new() -> Self {
        Self { client_provider: global_client_provider() }
    }

    fn with_provider(client_provider: ClientProvider) -> Self {
        Self { client_provider }
    }

    async fn search(&self, query: &str) -> Result<String, NetError> {
        let client = (self.client_provider)()?; // 每次请求时获取
        // ... 发起请求
        Ok(String::new())
    }
}
```

测试时注入假 provider：

```rust
let provider: ClientProvider = Box::new(|| {
    let client = NetClient::from_config(&ClientConfig::default())?; // 测试专用构造
    Ok(client)
});
let service = MarketService::with_provider(provider);
```

### 禁止的做法

```rust
// ❌ 构造时缓存固定客户端，代理更新后不生效
struct BadService {
    client: NetClient, // 不要这样做
}
fn new() -> Self {
    Self { client: global_client().expect("...") } // 不要 unwrap
}
```

## 2. 如何使用下载管理器

### crates/infra 层：DownloadManager

```rust
use sealantern_infra::download::DownloadManager;

// 生产：进程级全局单例（内部使用全局客户端 provider，每次下载重新获取客户端）
let manager = DownloadManager::instance();

// 生产（application 装配）：自定义 provider
let manager = DownloadManager::with_provider(global_client_provider());

// 测试：持有固定客户端
let manager = DownloadManager::new(client);
```

任务生命周期：

```rust
let id = manager.create("https://example.com/file.zip", "./download/file.zip", 8).await?;
// 或同时拿到状态句柄：
let (id, status) = manager.create_with_handle(url, path, thread_count).await?;

let snapshot = manager.get_progress(id).await;      // 查询单个任务；完成自动清理
let all = manager.get_all_progress().await;          // 查询全部
manager.cancel(id).await;                            // 取消并移除
let count = manager.task_count().await;              // 当前任务数
```

说明：

- `create` / `create_with_handle`：url、输出路径、线程数；线程数必须大于 0。
- 服务器不支持 Range 或没有 Content-Length 时，自动降级为单线程流式下载。
- 已完成或已取消的任务会从管理器中移除，避免无限增长。
- `Downloader` 是 `pub(crate)`，业务代码只与 `DownloadManager` 打交道。

### application 层：CoreDownloadService

```rust
use sealantern_application::service::CoreDownloadService;

// 生产：默认使用全局客户端 provider
let service = CoreDownloadService::new();

// 测试注入：
let service = CoreDownloadService::with_provider(provider);
let service = CoreDownloadService::with_manager(manager);
```

它实现 `sealantern_interface::DownloadService` 契约：

```rust
let id = service.create(DownloadRequest {
    url: "...".into(),
    save_path: "...".into(),
    thread_count: 8,
}).await?;                       // Result<String, DownloadServiceError>

let info = service.poll(&id).await?;   // Result<Option<DownloadTaskInfo>, ...>
service.cancel(&id).await?;
```

注意：线程数上限由 application 层校验（`MAX_DOWNLOAD_THREAD_COUNT = 64`），0 或超过上限返回 `InvalidInput`。

## 3. 使用规范（Checklist）

代码审查与编写时逐条核对：

- [ ] 生产路径是否只通过 `global_client()` / `global_client_provider()` 获取客户端？
- [ ] 服务是否注入 `ClientProvider` 而非缓存固定 `NetClient`？
- [ ] 每次请求/下载是否在发起前重新获取客户端？
- [ ] 是否仍存在 `NetClient::from_config`、`reqwest::Client::builder` 的生产调用（测试与 infra 内部构建除外）？
- [ ] 插件能力是否仍使用独立的 `PluginNetworkClient`，没有被迁移？
- [ ] `global_client()` 返回值是否用 `?` 传播而不是 `unwrap`/`expect`？
- [ ] 新增日志是否避免记录代理用户名、密码或完整含凭据 URL？
- [ ] 平台相关代码是否带 `#[cfg(...)]`（避免 Linux/Windows 死代码告警）？

## 参考

- 全局网络运行时：`crates/infra/src/net/runtime.rs`
- 下载管理器：`crates/infra/src/download/{manager.rs,multi.rs,tasks.rs,single.rs}`
- 应用下载服务：`application/src/service/download.rs`
- 插件安全客户端（保持独立）：`crates/infra/src/net/plugin/`
