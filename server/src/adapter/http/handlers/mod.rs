//! HTTP 路由处理器。
//!
//! handler 只做传输层薄转发：解析请求 → 调用应用层服务 → 收敛错误。

pub mod console;
pub mod cron;
pub mod download;
pub mod instance;
pub mod provisioning;
pub mod server;
pub mod server_config;
pub mod server_plugin;
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
    create_instance, delete_instance, get_instance, import_existing_instance, list_instances,
    rename_instance, update_instance_path,
};
pub use provisioning::inspect_server;
pub use server::{
    force_stop_server, restart_server, send_server_command, server_status, start_server,
    stop_server,
};
pub use server_config::{
    parse_server_properties_source, preview_server_properties_write,
    preview_server_properties_write_from_source, read_server_properties,
    read_server_properties_source, write_server_properties, write_server_properties_source,
};
pub use server_plugin::{
    delete_server_plugin, install_server_plugin, list_server_plugins,
    read_server_plugin_config_files, set_server_plugin_enabled,
};
pub use settings::{get_settings, settings_overview};
pub use system::{default_run_path, server_resource_usage, system_snapshot};
pub use update::check_update;
