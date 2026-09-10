//! 服务器配置（server.properties）契约模型。
//!
//! 定义宿主消费的配置条目与配置结构等模型，全部可序列化，供跨传输面传递。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 配置条目信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    /// 配置键
    pub key: String,
    /// 配置值
    pub value: String,
    /// 配置项描述
    pub description: String,
    /// 值类型（`number` / `boolean` / `string`）
    pub value_type: String,
    /// 默认值
    pub default_value: String,
    /// 配置分组（`network` / `player` / `game` / `world` / `performance` / `display` / `other`）
    pub category: String,
}

/// 服务器配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProperties {
    /// 可视化配置条目列表
    pub entries: Vec<ConfigEntry>,
    /// 原始键值对
    pub raw: BTreeMap<String, String>,
}
