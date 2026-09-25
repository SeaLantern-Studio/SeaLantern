//! 服务器插件（`plugins` 目录下的 jar）契约模型。
//!
//! 服务器插件由 Minecraft 服务端自身加载，SeaLantern 只读取目录内容与 jar 内的
//! `plugin.yml` / `bungee.yml` 元数据，不解析也不执行插件实现。

use serde::{Deserialize, Serialize};

/// 一个已安装服务器插件的摘要信息。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginSummary {
    /// 插件标识，取 `名称-版本`；元数据缺失时回落到文件名。
    pub id: String,
    /// 插件名（`plugin.yml` 的 `name`）。
    pub name: String,
    /// 插件版本（`plugin.yml` 的 `version`）。
    pub version: String,
    /// 插件描述（`plugin.yml` 的 `description`）。
    pub description: String,
    /// 插件作者：优先 `author`，其次 `authors` 的首项。
    pub author: String,
    /// jar 文件名，始终不含 `.disabled` 后缀。
    pub file_name: String,
    /// 文件字节数。
    pub file_size: u64,
    /// 是否处于启用状态（磁盘上为 `.jar` 而非 `.jar.disabled`）。
    pub enabled: bool,
    /// 主类全名（`plugin.yml` 的 `main`）。
    pub main_class: String,
    /// 是否存在与插件同名的配置目录（`plugins/<name>/`）。
    pub has_config_folder: bool,
    /// 配置目录中的可读文件；列表接口恒返回空数组，由按需读取填充。
    pub config_files: Vec<PluginConfigFile>,
}

/// 插件配置目录中的一个可读文本文件。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginConfigFile {
    /// 文件名。
    pub file_name: String,
    /// 文件全文。
    pub content: String,
    /// 扩展名，仅限 `yml` / `yaml` / `json` / `properties`。
    pub file_type: String,
    /// 文件绝对路径，供宿主交给系统程序打开。
    pub file_path: String,
}
