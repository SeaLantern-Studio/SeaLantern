mod cache;
mod parser;
mod version;

use std::path::Path;

pub use cache::load_or_refresh_starter_links_json;

use parser::{resolve_installer_url_from_nested_json, validate_starter_links_json, StarterLinksPayload};

pub fn resolve_installer_url_from_cache_file(
    links_file_path: &Path,
    core_type_key: &str,
    mc_version: &str,
) -> Result<(String, Option<String>), String> {
    let body = std::fs::read(links_file_path)
        .map_err(|e| format!("读取本地 Starter 下载信息失败: {}", e))?;
    validate_starter_links_json(&body)?;

    let payload: StarterLinksPayload =
        serde_json::from_slice(&body).map_err(|e| format!("解析 Starter 下载信息失败: {}", e))?;
    let core_key = core_type_key.trim().to_ascii_lowercase();
    let target_version = mc_version.trim().to_ascii_lowercase();
    if core_key.is_empty() || target_version.is_empty() {
        return Err("Starter 下载参数缺少核心类型或 MC 版本".to_string());
    }

    if let Some(installer_url) =
        resolve_installer_url_from_nested_json(&payload, &core_key, &target_version)?
    {
        return Ok((installer_url, None));
    }

    Err(format!(
        "未在 CNB 镜像中找到匹配下载链接：core={}, version={}",
        core_type_key, mc_version
    ))
}

#[cfg(test)]
mod tests {
    use super::resolve_installer_url_from_cache_file;

    fn write_links_file(body: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("jar_lfs_links.json");
        std::fs::write(file, body).unwrap();
        dir
    }

    #[test]
    fn resolve_installer_url_from_cache_file_finds_exact_match() {
        let dir = write_links_file(
            r#"{
  "types": ["forge"],
  "forge": {
    "1.20.1": {
      "forge-installer.jar": "https://example.com/forge-1.20.1-installer.jar"
    }
  }
}"#,
        );
        let file = dir.path().join("jar_lfs_links.json");

        let result = resolve_installer_url_from_cache_file(&file, "forge", "1.20.1").unwrap();
        assert_eq!(
            result.0,
            "https://example.com/forge-1.20.1-installer.jar"
        );
        assert_eq!(result.1, None);
    }

    #[test]
    fn resolve_installer_url_from_cache_file_normalizes_core_version_and_url_whitespace() {
        let dir = write_links_file(
            r#"{
  "types": ["Forge"],
  "Forge": {
    "1.20.1": {
      "forge-installer.jar": "  https://example.com/forge-1.20.1-installer.jar  "
    }
  }
}"#,
        );
        let file = dir.path().join("jar_lfs_links.json");

        let result =
            resolve_installer_url_from_cache_file(&file, "  forge  ", "  1.20.1  ").unwrap();

        assert_eq!(result.0, "https://example.com/forge-1.20.1-installer.jar");
        assert_eq!(result.1, None);
    }

    #[test]
    fn resolve_installer_url_from_cache_file_prefers_specific_prefix_bucket_and_installer() {
        let dir = write_links_file(
            r#"{
  "types": ["forge"],
  "forge": {
    "1.20": {
      "forge-server.jar": "https://example.com/forge-1.20-server.jar"
    },
    "1.20.1": {
      "forge-server.jar": "https://example.com/forge-1.20.1-server.jar",
      "forge-installer.jar": "https://example.com/forge-1.20.1-installer.jar"
    }
  }
}"#,
        );
        let file = dir.path().join("jar_lfs_links.json");

        let result =
            resolve_installer_url_from_cache_file(&file, "forge", "1.20.1-47.3.0").unwrap();

        assert_eq!(result.0, "https://example.com/forge-1.20.1-installer.jar");
        assert_eq!(result.1, None);
    }

    #[test]
    fn resolve_installer_url_from_cache_file_prefers_installer_jar_over_non_jar_installer_files() {
        let dir = write_links_file(
            r#"{
  "types": ["forge"],
  "forge": {
    "1.20.1": {
      "forge-installer.txt": "https://example.com/forge-1.20.1-installer.txt",
      "forge-installer.jar": "https://example.com/forge-1.20.1-installer.jar"
    }
  }
}"#,
        );
        let file = dir.path().join("jar_lfs_links.json");

        let result = resolve_installer_url_from_cache_file(&file, "forge", "1.20.1").unwrap();

        assert_eq!(result.0, "https://example.com/forge-1.20.1-installer.jar");
        assert_eq!(result.1, None);
    }

    #[test]
    fn resolve_installer_url_from_cache_file_prefers_more_specific_installer_file_name_not_url_text() {
        let dir = write_links_file(
            r#"{
  "types": ["forge"],
  "forge": {
    "1.20.1": {
      "forge-installer.jar": "https://z.example.com/downloads/forge-installer.jar",
      "forge-1.20.1-installer.jar": "https://a.example.com/downloads/forge-1.20.1-installer.jar"
    }
  }
}"#,
        );
        let file = dir.path().join("jar_lfs_links.json");

        let result = resolve_installer_url_from_cache_file(&file, "forge", "1.20.1").unwrap();

        assert_eq!(
            result.0,
            "https://a.example.com/downloads/forge-1.20.1-installer.jar"
        );
        assert_eq!(result.1, None);
    }

    #[test]
    fn resolve_installer_url_from_cache_file_rejects_empty_url_for_matching_installer() {
        let dir = write_links_file(
            r#"{
  "types": ["forge"],
  "forge": {
    "1.20.1": {
      "forge-installer.jar": "   "
    }
  }
}"#,
        );
        let file = dir.path().join("jar_lfs_links.json");

        let error = resolve_installer_url_from_cache_file(&file, "forge", "1.20.1")
            .expect_err("empty URL for matching installer should not be silently ignored");

        assert!(error.contains("Starter 下载信息无效"), "unexpected error: {}", error);
        assert!(error.contains("forge-installer.jar"), "unexpected error: {}", error);
    }
}
