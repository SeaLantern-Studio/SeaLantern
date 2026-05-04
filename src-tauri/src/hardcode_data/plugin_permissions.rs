//! 插件权限展示清单。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermissionInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub risk_level: String,
    pub category: String,
}

pub fn get_plugin_permission_list() -> Vec<PluginPermissionInfo> {
    vec![
        PluginPermissionInfo {
            id: "log".to_string(),
            name: "Logging".to_string(),
            description: "Allow plugin to write logs".to_string(),
            risk_level: "low".to_string(),
            category: "system".to_string(),
        },
        PluginPermissionInfo {
            id: "fs".to_string(),
            name: "File System (Legacy)".to_string(),
            description: "Allow plugin to read/write files in plugin data directory (deprecated, use fs.data)".to_string(),
            risk_level: "low".to_string(),
            category: "filesystem".to_string(),
        },
        PluginPermissionInfo {
            id: "fs.data".to_string(),
            name: "File System - Data".to_string(),
            description: "Allow plugin to read/write files in its private data directory".to_string(),
            risk_level: "low".to_string(),
            category: "filesystem".to_string(),
        },
        PluginPermissionInfo {
            id: "fs.server".to_string(),
            name: "File System - Server".to_string(),
            description: "Allow plugin to read/write files in server configuration directory".to_string(),
            risk_level: "medium".to_string(),
            category: "filesystem".to_string(),
        },
        PluginPermissionInfo {
            id: "fs.global".to_string(),
            name: "File System - Global".to_string(),
            description: "Allow plugin to read/write files in global application directory".to_string(),
            risk_level: "high".to_string(),
            category: "filesystem".to_string(),
        },
        PluginPermissionInfo {
            id: "http".to_string(),
            name: "HTTP Requests".to_string(),
            description: "Allow plugin to make HTTP requests to external servers".to_string(),
            risk_level: "medium".to_string(),
            category: "network".to_string(),
        },
        PluginPermissionInfo {
            id: "i18n".to_string(),
            name: "Internationalization".to_string(),
            description: "Allow plugin to access and modify locale settings".to_string(),
            risk_level: "low".to_string(),
            category: "system".to_string(),
        },
        PluginPermissionInfo {
            id: "process".to_string(),
            name: "Process Control".to_string(),
            description: "Allow plugin to start and manage system processes".to_string(),
            risk_level: "high".to_string(),
            category: "system".to_string(),
        },
        PluginPermissionInfo {
            id: "server".to_string(),
            name: "Server Control".to_string(),
            description: "Allow plugin to control Minecraft servers".to_string(),
            risk_level: "medium".to_string(),
            category: "server".to_string(),
        },
        PluginPermissionInfo {
            id: "storage".to_string(),
            name: "Storage".to_string(),
            description: "Allow plugin to store persistent data".to_string(),
            risk_level: "low".to_string(),
            category: "storage".to_string(),
        },
        PluginPermissionInfo {
            id: "ui".to_string(),
            name: "UI Components".to_string(),
            description: "Allow plugin to create and manage UI components".to_string(),
            risk_level: "low".to_string(),
            category: "ui".to_string(),
        },
        PluginPermissionInfo {
            id: "system".to_string(),
            name: "System Information".to_string(),
            description: "Allow plugin to access system information".to_string(),
            risk_level: "medium".to_string(),
            category: "system".to_string(),
        },
        PluginPermissionInfo {
            id: "console".to_string(),
            name: "Console Access".to_string(),
            description: "Allow plugin to access and control the console".to_string(),
            risk_level: "medium".to_string(),
            category: "system".to_string(),
        },
        PluginPermissionInfo {
            id: "element".to_string(),
            name: "DOM Elements".to_string(),
            description: "Allow plugin to create and manipulate DOM elements".to_string(),
            risk_level: "low".to_string(),
            category: "ui".to_string(),
        },
        PluginPermissionInfo {
            id: "api".to_string(),
            name: "Plugin API".to_string(),
            description: "Allow plugin to call other plugins' APIs".to_string(),
            risk_level: "medium".to_string(),
            category: "api".to_string(),
        },
    ]
}
