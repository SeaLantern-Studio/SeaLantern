pub mod async_loader;
pub mod config_parser;
pub mod download_manager;
pub mod global;
#[cfg(feature = "docker")]
pub mod http_command_handlers;
#[cfg(not(feature = "docker"))]
pub mod http_command_handlers {
    //! HTTP 命令处理模块（Stub）
    //!
    //! 当未启用 `docker` feature 时，真实的 HTTP 命令处理器（基于 axum/tower）
    //! 不会被编译。为了让引用该模块的代码在非 docker 构建下仍然能够编译，
    //! 本模块提供最小的类型与接口替身（stub）。
    //!
    //! 这些替身只提供 API 的最小签名与安全的空实现，以避免编译器关于未使用
    //! 项目的大量警告或链接错误。真正的实现仍在开启 `docker` feature 时编译。

    #![allow(dead_code, unused_imports, unused_variables, unused_mut)]

    use futures::future::BoxFuture;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::sync::Arc;

    /// HTTP API 命令处理器类型（占位签名，与真实实现保持一致）
    pub type CommandHandler = fn(Value) -> BoxFuture<'static, Result<Value, String>>;

    /// 命令注册表（Stub）
    ///
    /// 在非 docker 构建时提供最小实现：保持接口兼容，但不注册任何命令。
    pub struct CommandRegistry {
        handlers: HashMap<String, CommandHandler>,
    }

    impl CommandRegistry {
        /// 返回一个空的命令注册表
        pub fn new() -> Self {
            Self { handlers: HashMap::new() }
        }

        /// 获取命令处理器（Stub 始终返回 None）
        pub fn get_handler(&self, _command: &str) -> Option<&CommandHandler> {
            None
        }

        /// 列出已注册的命令（Stub 始终返回空列表）
        pub fn list_commands(&self) -> Vec<String> {
            Vec::new()
        }
    }

    impl Default for CommandRegistry {
        fn default() -> Self {
            Self::new()
        }
    }

    // 不导出其它具体 handler 函数（create/import/start/...），这些只在 docker feature 下存在。
}
#[cfg(feature = "docker")]
pub mod http_server;
#[cfg(not(feature = "docker"))]
pub mod http_server {
    //! HTTP 服务模块（Stub）
    //!
    //! 当未启用 `docker` feature 时，真实的 HTTP 服务实现（基于 axum/tower-http）
    //! 不会被编译。此处提供最小的 `run_http_server` 替身以保证在非 docker 构建下
    //! 引用该接口的代码仍能编译并正常运行（仅为空实现，不启动任何监听）。

    #![allow(dead_code)]

    pub async fn run_http_server(_addr: &str, _static_dir: Option<String>) {
        // 非 docker 构建时不启动 HTTP 服务
    }
}
pub mod i18n;
pub mod java_detector;
pub mod java_installer;
pub mod join_manager;
pub mod mcs_plugin_manager;
pub mod mod_manager;
pub mod panic_report;
pub mod player_manager;
pub mod server_downloader;
pub mod server_id_manager;
pub mod server_installer;
pub mod server_log_pipeline;
pub mod server_manager;
pub mod settings_manager;
pub mod starter_installer_links;
