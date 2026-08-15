//! HTTP 路由处理器。
//!
//! handler 只做传输层薄转发：解析请求 → 调用应用层服务 → 收敛错误。

pub mod console;
pub mod cron;
pub mod download;
pub mod instance;
pub mod server;
pub mod settings;
pub mod system;
pub mod update;

pub use console::console_logs;
pub use cron::{
    create_cron_task, delete_cron_task, list_cron_tasks, run_cron_task, set_cron_task_enabled,
    update_cron_task,
};
pub use download::{cancel_download, create_download, query_download};
pub use instance::{
    create_instance, delete_instance, get_instance, list_instances, rename_instance,
    update_instance_path,
};
pub use server::{
    force_stop_server, restart_server, send_server_command, server_status, start_server,
    stop_server,
};
pub use settings::settings_overview;
pub use system::{default_run_path, directory_usage, process_usage, system_snapshot};
pub use update::check_update;
