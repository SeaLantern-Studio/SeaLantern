//! Java 环境信息模型。

use serde::{Deserialize, Serialize};

/// Java 环境信息，用于检测结果和应用设置缓存。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JavaInfo {
    pub path: String,
    pub version: String,
    pub vendor: String,
    pub is_64bit: bool,
    pub major_version: u32,
    /// Java 安装信息的规则置信度，范围为 0 到 100。
    #[serde(default)]
    pub confidence: u8,
}

#[cfg(test)]
mod tests {
    use super::JavaInfo;

    #[test]
    fn legacy_java_cache_defaults_confidence() {
        let info: JavaInfo = serde_json::from_str(
            r#"{
                "path": "/opt/jdk/bin/java",
                "version": "21.0.1",
                "vendor": "OpenJDK",
                "is_64bit": true,
                "major_version": 21
            }"#,
        )
        .expect("legacy Java info should remain readable");

        assert_eq!(info.confidence, 0);
    }
}
