//! 实例启动配置（`SeaLantern/config.toml`）契约模型。
//!
//! 该文件保存实例级的启动参数覆盖：字段缺省表示「沿用实例默认值」，由应用层
//! 决定回落到实例注册表还是全局设置。此处只承载可跨传输面的数据形状。

use serde::{Deserialize, Serialize};

/// 实例级启动配置覆盖。
///
/// 两个字段都可缺省：`None` 既表示文件中未出现该字段，也表示字段存在但为空，
/// 调用方统一按「无覆盖」处理。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SLStartupConfig {
    /// 最大堆内存（MiB）。
    #[serde(default)]
    pub max_memory: Option<u32>,
    /// 最小堆内存（MiB）。
    #[serde(default)]
    pub min_memory: Option<u32>,
}
