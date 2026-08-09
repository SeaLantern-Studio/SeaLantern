//! HTTP 路由处理器。
//!
//! handler 只做传输层薄转发：解析请求 → 调用应用层服务 → 收敛错误。

pub mod instance;
pub mod server;
pub mod settings;
pub mod system;

pub use instance::{
    create_instance, delete_instance, get_instance, list_instances, rename_instance,
    update_instance_path,
};
pub use server::{
    force_stop_server, restart_server, send_server_command, server_status, start_server,
    stop_server,
};
pub use settings::settings_overview;
pub use system::{directory_usage, process_usage, system_snapshot};
