//! Java 环境信息模型。

use serde::{Deserialize, Deserializer, Serialize};

/// Java 环境信息，用于检测结果和应用设置缓存。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JavaInfo {
    pub path: String,
    pub version: String,
    #[serde(default, deserialize_with = "deserialize_default_on_null")]
    pub vendor: String,
    #[serde(default, deserialize_with = "deserialize_default_on_null")]
    pub is_64bit: bool,
    #[serde(default, deserialize_with = "deserialize_default_on_null")]
    pub major_version: u32,
    /// Java 安装信息的规则置信度，范围为 0 到 100。
    #[serde(default, deserialize_with = "deserialize_default_on_null")]
    pub confidence: u8,
}

fn deserialize_default_on_null<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Option::<T>::deserialize(deserializer).map(Option::unwrap_or_default)
}

#[cfg(test)]
mod tests {
    use super::JavaInfo;

    #[test]
    fn legacy_java_cache_defaults_new_metadata() {
        let info: JavaInfo = serde_json::from_str(
            r#"{
                "path": "/opt/jdk/bin/java",
                "version": "21.0.1"
            }"#,
        )
        .expect("legacy Java info should remain readable");

        assert_eq!(info.vendor, "");
        assert!(!info.is_64bit);
        assert_eq!(info.major_version, 0);
        assert_eq!(info.confidence, 0);
    }

    #[test]
    fn legacy_java_cache_accepts_null_metadata() {
        let info: JavaInfo = serde_json::from_str(
            r#"{
                "path": "/opt/jdk/bin/java",
                "version": "21.0.1",
                "vendor": null,
                "is_64bit": null,
                "major_version": null,
                "confidence": null
            }"#,
        )
        .expect("null Java metadata should use safe defaults");

        assert_eq!(info.vendor, "");
        assert!(!info.is_64bit);
        assert_eq!(info.major_version, 0);
        assert_eq!(info.confidence, 0);
    }
}
