---
name: network-client-usage
description: 引导在 crates/application 层统一使用进程级全局网络客户端（global_client / ClientProvider）与下载管理器（DownloadManager / CoreDownloadService）的规范。使用场景：新增联网的业务服务、改造现有服务使其跟随全局代理、需要创建/查询/取消下载任务、判断某个位置应该直接获取客户端还是注入 provider、审查是否违反“请求时获取、不缓存固定客户端、插件客户端独立”等约定。Also use when writing or reviewing Rust code that touches sealantern_infra::net or sealantern_infra::download so the producer stays on the global proxy path.
description_en: Guidance for uniformly using the process-level global network client (global_client / ClientProvider) and download manager (DownloadManager / CoreDownloadService) at the crates/application layer. Use when adding network-facing services, migrating existing services to follow the global proxy, creating/querying/canceling download tasks, deciding whether to fetch a client directly or inject a provider, or reviewing code for the "fetch-per-request, never cache a fixed client, plugin client stays separate" conventions. Also use when writing or reviewing Rust code that touches sealantern_infra::net or sealantern_infra::download so the producer stays on the global proxy path.
---

# Global Network Client & Download Manager Usage Guide

The repository's process-level networking is managed by `crates/infra`. Proxy policies
(Adaptive / Preserve / Manual / Disabled) are written into the process-level global network
runtime by the settings layer and the system-proxy polling service. Business code is only
responsible for reading the _current_ global client and issuing requests against it.

## Core Principles

1. **Single source of truth**: production code only obtains a client from `global_client()` or
   `global_client_provider()`. Never call `NetClient::from_config` / `reqwest::Client::builder`
   yourself.
2. **Fetch per request**: obtain a client right before each request (or before each download
   task). Services must not cache a fixed `NetClient` at construction time.
3. **Inject a provider, not a client**: for dependency injection, pass a `ClientProvider`
   (a functional fetcher). Tests can inject fake providers.
4. **Plugin exception**: plugin networking must keep using the standalone `PluginNetworkClient`
   (DNS/IP pinning, SSRF protection, request-header allowlist). Do not migrate it to the global client.
5. **Error propagation**: `global_client()` returns `Result<NetClient, NetError>`. Propagate with
   `?`; never use `unwrap`/`expect`.

## 1. How to Use the Global Client

### Direct Access (simple call sites)

```rust
use sealantern_infra::net::global_client;

async fn fetch_text(url: &str) -> Result<String, NetError> {
    let client = global_client()?; // fetch the current global client before each request
    let response = client.get(url)?.send().await?;
    Ok(response.text().await.unwrap_or_default())
}
```

Key types and functions:

```rust
// crates/infra/src/net/runtime.rs
pub fn global_client() -> Result<NetClient, NetError>;                    // current global client
pub type ClientProvider = Box<dyn Fn() -> Result<NetClient, NetError> + Send + Sync>;
pub fn global_client_provider() -> ClientProvider;                         // default provider
```

`NetClient` is a cheap clone (it wraps a `reqwest::Client` plus a retry policy). Every call
returns the client that matches the _current_ proxy policy. After the proxy settings or the
system proxy change and the runtime rebuilds the client, the next `global_client()` call
automatically returns the new client.

### Inject a Provider (recommended for long-lived services)

```rust
use std::sync::Arc;
use sealantern_infra::net::{global_client_provider, ClientProvider, NetClient, NetError};

struct MarketService {
    client_provider: ClientProvider,   // inject the provider, do not store a fixed client
}

impl MarketService {
    fn new() -> Self {
        Self { client_provider: global_client_provider() }
    }

    fn with_provider(client_provider: ClientProvider) -> Self {
        Self { client_provider }
    }

    async fn search(&self, query: &str) -> Result<String, NetError> {
        let client = (self.client_provider)()?; // fetch per request
        // ... issue the request
        Ok(String::new())
    }
}
```

Inject a fake provider in tests:

```rust
let provider: ClientProvider = Box::new(|| {
    let client = NetClient::from_config(&ClientConfig::default())?; // test-only construction
    Ok(client)
});
let service = MarketService::with_provider(provider);
```

### Anti-Patterns

```rust
// ❌ Cache a fixed client at construction time; proxy updates will not take effect.
struct BadService {
    client: NetClient, // do not do this
}
fn new() -> Self {
    Self { client: global_client().expect("...") } // do not unwrap
}
```

## 2. How to Use the Download Manager

### infra layer: `DownloadManager`

```rust
use sealantern_infra::download::DownloadManager;

// production: process-level singleton (uses the global client provider internally,
// re-fetching the client for each download)
let manager = DownloadManager::instance();

// production (application composition): custom provider
let manager = DownloadManager::with_provider(global_client_provider());

// tests: hold a fixed client
let manager = DownloadManager::new(client);
```

Task lifecycle:

```rust
let id = manager.create("https://example.com/file.zip", "./download/file.zip", 8).await?;
// or also grab the status handle:
let (id, status) = manager.create_with_handle(url, path, thread_count).await?;

let snapshot = manager.get_progress(id).await;      // query one task; auto-cleanup when finished
let all = manager.get_all_progress().await;          // query all tasks
manager.cancel(id).await;                            // cancel and remove
let count = manager.task_count().await;              // current task count
```

Notes:

- `create` / `create_with_handle`: url, output path, thread count; thread count must be > 0.
- When the server does not support Range or provides no Content-Length, the manager falls back to
  single-threaded streaming download automatically.
- Finished or canceled tasks are removed from the manager to avoid unbounded growth.
- `Downloader` is `pub(crate)`; business code only talks to `DownloadManager`.

### application layer: `CoreDownloadService`

```rust
use sealantern_application::service::CoreDownloadService;

// production: uses the global client provider by default
let service = CoreDownloadService::new();

// test injection:
let service = CoreDownloadService::with_provider(provider);
let service = CoreDownloadService::with_manager(manager);
```

It implements the `sealantern_interface::DownloadService` contract:

```rust
let id = service.create(DownloadRequest {
    url: "...".into(),
    save_path: "...".into(),
    thread_count: 8,
}).await?;                       // Result<String, DownloadServiceError>

let info = service.poll(&id).await?;   // Result<Option<DownloadTaskInfo>, ...>
service.cancel(&id).await?;
```

Note: the thread-count limit is validated at the application layer
(`MAX_DOWNLOAD_THREAD_COUNT = 64`); 0 or values above the limit return `InvalidInput`.

## 3. Usage Conventions (Checklist)

Verify each item when writing or reviewing code:

- [ ] Does production code obtain clients only through `global_client()` / `global_client_provider()`?
- [ ] Do services inject a `ClientProvider` instead of caching a fixed `NetClient`?
- [ ] Is a new client fetched before every request/download?
- [ ] Are there remaining production calls to `NetClient::from_config` / `reqwest::Client::builder`
      (excluding tests and infra-internal construction)?
- [ ] Do plugin capabilities still use the standalone `PluginNetworkClient` and were not migrated?
- [ ] Is the `global_client()` result propagated with `?` instead of `unwrap`/`expect`?
- [ ] Do new log statements avoid recording proxy usernames, passwords, or fully-qualified
      credential-bearing URLs?
- [ ] Do platform-specific code paths carry `#[cfg(...)]` (avoiding Linux/Windows dead-code warnings)?

## References

- Global network runtime: `crates/infra/src/net/runtime.rs`
- Download manager: `crates/infra/src/download/{manager.rs,multi.rs,tasks.rs,single.rs}`
- Application download service: `application/src/service/download.rs`
- Plugin secure client (stays standalone): `crates/infra/src/net/plugin/`

---

中文原版：`references/zh-CN.md`
