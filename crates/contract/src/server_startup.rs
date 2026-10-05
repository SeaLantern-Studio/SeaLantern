//! 实例启动配置（`sl.json` 的 `startup` 段）契约模型。
//!
//! 只承载前端「启动属性」页关心的内存覆盖值；其余启动字段由实例文档整体
//! 维护。字段缺省（`None`）表示「沿用实例/全局默认」，由前端回落到默认值。

use serde::{Deserialize, Serialize};

/// 实例启动配置的内存覆盖。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SLStartupConfig {
    /// 最大堆内存（MiB）；`None` 表示未覆盖。
    #[serde(default)]
    pub max_memory: Option<u32>,
    /// 最小堆内存（MiB）；`None` 表示未覆盖。
    #[serde(default)]
    pub min_memory: Option<u32>,
}
