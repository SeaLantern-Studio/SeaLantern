//! HTTP 子模块：集中管理 HTTP server 与命令处理器，并统一 `docker` feature 逻辑。
//!
//! - 在启用 `docker` feature 时，导出真实的 HTTP 实现：
//!   - `http_server`：基于 axum/tower-http 的 HTTP 服务
//!   - `http_command_handlers`：将 HTTP API 映射到内部 commands 模块
//! - 在未启用 `docker` feature 时，仅提供最小的 stub 实现，保证其他模块编译通过。

// docker 模式下：启用真实实现
#[cfg(feature = "docker")]
pub mod http_command_handlers;
#[cfg(feature = "docker")]
pub mod http_server;

#[cfg(feature = "docker")]
#[allow(unused_imports)]
pub use http_command_handlers::{CommandHandler, CommandRegistry};
#[cfg(feature = "docker")]
pub use http_server::run_http_server;

// 非 docker 模式下：提供 stub 实现
#[cfg(not(feature = "docker"))]
pub mod http_command_handlers {
    //! HTTP 命令处理模块（Stub，非 docker 构建）。
    //!
    //! 当未启用 `docker` feature 时，真实的 HTTP 命令处理器不会被编译。
    //! 为了让引用该模块的代码在非 docker 构建下仍然能够编译，
    //! 本模块提供最小的类型与接口替身（stub）。

    #![allow(dead_code, unused_imports, unused_variables, unused_mut)]

    use futures::future::BoxFuture;
    use serde_json::Value;
    use std::collections::HashMap;

    /// HTTP API 命令处理器类型（占位签名，与真实实现保持一致）。
    pub type CommandHandler = fn(Value) -> BoxFuture<'static, Result<Value, String>>;

    /// 命令注册表（Stub）。
    ///
    /// 在非 docker 构建时提供最小实现：保持接口兼容，但不注册任何命令。
    pub struct CommandRegistry {
        handlers: HashMap<String, CommandHandler>,
    }

    impl CommandRegistry {
        /// 返回一个空的命令注册表。
        pub fn new() -> Self {
            Self { handlers: HashMap::new() }
        }

        /// 获取命令处理器（Stub 始终返回 None）。
        pub fn get_handler(&self, _command: &str) -> Option<&CommandHandler> {
            None
        }

        /// 列出已注册的命令（Stub 始终返回空列表）。
        pub fn list_commands(&self) -> Vec<String> {
            Vec::new()
        }
    }

    impl Default for CommandRegistry {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(not(feature = "docker"))]
pub mod http_server {
    //! HTTP 服务模块（Stub，非 docker 构建）。
    //!
    //! 当未启用 `docker` feature 时，真实的 HTTP 服务实现不会被编译。
    //! 此处提供最小的 `run_http_server` 替身以保证在非 docker 构建下
    //! 引用该接口的代码仍能编译并正常运行（仅为空实现，不启动任何监听）。

    #![allow(dead_code)]

    pub async fn run_http_server(_addr: &str, _static_dir: Option<String>) {
        // 非 docker 构建时不启动 HTTP 服务
    }
}

// 为了让上层只关心 `services::http::run_http_server` / `CommandRegistry` 等 API，
// 在非 docker 模式下也导出同名符号。
#[cfg(not(feature = "docker"))]
#[allow(unused_imports)]
pub use http_command_handlers::{CommandHandler, CommandRegistry};
#[cfg(not(feature = "docker"))]
pub use http_server::run_http_server;
