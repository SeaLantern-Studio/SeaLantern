use serde::Serialize;

/// 提供更新信息的发布源。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpdateSource {
    /// GitHub Releases。
    Github,
    /// CNB.cool 发布页。
    Cnb,
    /// Arch User Repository。
    ArchAur,
}

/// 当前平台的应用更新检查结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateInfo {
    /// 是否存在适合当前平台的新版本资源。
    pub has_update: bool,
    /// 最新发布版本。
    pub latest_version: String,
    /// 当前应用版本。
    pub current_version: String,
    /// 当前平台资源的下载地址。
    pub download_url: Option<String>,
    /// 发布说明。
    pub release_notes: Option<String>,
    /// 发布时间。
    pub published_at: Option<String>,
    /// 最终采用的发布源。
    pub source: Option<UpdateSource>,
    /// 发布资源的 SHA-256；发布源未提供时为空。
    pub sha256: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::UpdateCheckServiceError;

    use super::{UpdateInfo, UpdateSource};

    #[test]
    fn update_contract_uses_stable_snake_case_names() {
        let info = UpdateInfo {
            has_update: true,
            latest_version: "2.0.0".to_owned(),
            current_version: "1.0.0".to_owned(),
            download_url: Some("https://example.com/update".to_owned()),
            release_notes: None,
            published_at: None,
            source: Some(UpdateSource::ArchAur),
            sha256: None,
        };

        let value = serde_json::to_value(info).expect("serialize update info");

        assert_eq!(value["has_update"], true);
        assert_eq!(value["latest_version"], "2.0.0");
        assert_eq!(value["source"], "arch-aur");
        assert!(value.get("hasUpdate").is_none());
        assert!(value.get("latestVersion").is_none());
    }

    #[test]
    fn update_errors_use_snake_case_variants() {
        let value = serde_json::to_value(UpdateCheckServiceError::CheckFailed)
            .expect("serialize update check error");

        assert_eq!(value, "check_failed");
    }
}
