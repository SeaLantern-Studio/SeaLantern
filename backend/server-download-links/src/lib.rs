use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseDownloadLinks {
    pub server_types: Vec<String>,
    pub links: Vec<TypeDownloadLinks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeDownloadLinks {
    pub server_type: String,
    pub versions: Vec<String>,
    pub links: Vec<DownloadLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadLink {
    pub version: String,
    pub file_name: String,
    pub url: String,
}

impl BaseDownloadLinks {
    pub fn get_types(&self) -> Vec<String> {
        self.server_types.clone()
    }

    pub fn get_type_by_name(&self, type_name: &str) -> Option<&TypeDownloadLinks> {
        self.links
            .iter()
            .find(|group| group.server_type.eq_ignore_ascii_case(type_name))
    }
}

impl TypeDownloadLinks {
    pub fn get_versions(&self) -> Vec<String> {
        self.versions.clone()
    }

    pub fn get_link_by_version(&self, version: &str) -> Option<&DownloadLink> {
        let normalized_version = version.trim();
        self.links
            .iter()
            .filter(|link| link.version.trim() == normalized_version)
            .min_by_key(|link| download_link_priority(link))
    }
}

pub fn parse_base_download_links(response_body: &str) -> Result<BaseDownloadLinks, String> {
    let root_json: Value =
        serde_json::from_str(response_body).map_err(|e| format!("解析下载配置失败: {}", e))?;

    let mut all_server_types = Vec::new();
    let mut type_download_groups = Vec::new();

    if let Some(type_name_list) = root_json.get("types").and_then(|t| t.as_array()) {
        for type_node in type_name_list {
            let server_type_name = require_string(type_node, "types[]")?.to_string();
            all_server_types.push(server_type_name.clone());

            let type_detail_data = require_object_value_case_insensitive(
                &root_json,
                &server_type_name,
                &format!("types.{}", server_type_name),
            )?;

            let mut mc_versions_under_type = Vec::new();
            let mut download_links_under_type = Vec::new();

            if let Some(version_list_node) =
                type_detail_data.get("versions").and_then(|v| v.as_array())
            {
                for version_node in version_list_node {
                    let mc_version_str = require_string(version_node, "versions[]")?.to_string();
                    mc_versions_under_type.push(mc_version_str.clone());

                    let file_mapping = require_object_field(
                        type_detail_data,
                        &mc_version_str,
                        &format!("{}.{}", server_type_name, mc_version_str),
                    )?;

                    for (file_key_name, file_url_value) in file_mapping {
                        let download_entry = DownloadLink {
                            version: mc_version_str.clone(),
                            file_name: file_key_name.clone(),
                            url: require_string(
                                file_url_value,
                                &format!("{}.{}", server_type_name, file_key_name),
                            )?
                            .to_string(),
                        };
                        download_links_under_type.push(download_entry);
                    }
                }
            }

            let type_group = TypeDownloadLinks {
                server_type: server_type_name,
                versions: mc_versions_under_type,
                links: download_links_under_type,
            };
            type_download_groups.push(type_group);
        }
    }

    Ok(BaseDownloadLinks {
        server_types: all_server_types,
        links: type_download_groups,
    })
}

fn require_string<'a>(value: &'a Value, field_name: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .ok_or_else(|| format!("下载配置字段 {} 必须是字符串", field_name))
}

fn require_object_field<'a>(
    value: &'a Value,
    field_name: &str,
    path: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .get(field_name)
        .and_then(|field| field.as_object())
        .ok_or_else(|| format!("下载配置字段 {} 必须是对象", path))
}

fn require_object_value_case_insensitive<'a>(
    root: &'a Value,
    key: &str,
    field_name: &str,
) -> Result<&'a Value, String> {
    get_object_value_case_insensitive(root, key)
        .ok_or_else(|| format!("下载配置字段 {} 缺失", field_name))
}

fn get_object_value_case_insensitive<'a>(root: &'a Value, key: &str) -> Option<&'a Value> {
    root.as_object()?.iter().find_map(|(name, value)| {
        if name.eq_ignore_ascii_case(key) {
            Some(value)
        } else {
            None
        }
    })
}

fn download_link_priority(link: &DownloadLink) -> (u8, String) {
    let file_name = link.file_name.to_ascii_lowercase();
    let score = if file_name.contains("installer") && file_name.ends_with(".jar") {
        0
    } else if file_name.ends_with(".jar") {
        1
    } else {
        2
    };

    (score, file_name)
}

#[cfg(test)]
mod tests {
    use super::parse_base_download_links;

    #[test]
    fn parse_base_download_links_reads_types_versions_and_urls() {
        let parsed = parse_base_download_links(
            r#"{
  "types": ["paper"],
  "paper": {
    "versions": ["1.20.1"],
    "1.20.1": {
      "paper-1.20.1.jar": "https://example.com/paper-1.20.1.jar"
    }
  }
}"#,
        )
        .unwrap();

        assert_eq!(parsed.server_types, vec!["paper"]);
        let group = parsed.get_type_by_name("paper").unwrap();
        assert_eq!(group.versions, vec!["1.20.1"]);
        let link = group.get_link_by_version("1.20.1").unwrap();
        assert_eq!(link.file_name, "paper-1.20.1.jar");
        assert_eq!(link.url, "https://example.com/paper-1.20.1.jar");
    }

    #[test]
    fn parse_base_download_links_matches_type_details_case_insensitively() {
        let parsed = parse_base_download_links(
            r#"{
  "types": ["Forge"],
  "forge": {
    "versions": ["1.20.1"],
    "1.20.1": {
      "forge-1.20.1-installer.jar": "https://example.com/forge-1.20.1-installer.jar"
    }
  }
}"#,
        )
        .unwrap();

        let group = parsed.get_type_by_name("forge").unwrap();
        assert_eq!(group.server_type, "Forge");
        assert_eq!(group.versions, vec!["1.20.1"]);
        let link = group.get_link_by_version("1.20.1").unwrap();
        assert_eq!(link.file_name, "forge-1.20.1-installer.jar");
    }

    #[test]
    fn get_type_by_name_is_case_insensitive() {
        let parsed = parse_base_download_links(
            r#"{
  "types": ["Paper"],
  "paper": {
    "versions": ["1.20.1"],
    "1.20.1": {
      "paper-1.20.1.jar": "https://example.com/paper-1.20.1.jar"
    }
  }
}"#,
        )
        .unwrap();

        assert!(parsed.get_type_by_name("paper").is_some());
        assert!(parsed.get_type_by_name("PAPER").is_some());
    }

    #[test]
    fn get_link_by_version_prefers_installer_jar_when_multiple_files_share_version() {
        let parsed = parse_base_download_links(
            r#"{
  "types": ["forge"],
  "forge": {
    "versions": ["1.20.1"],
    "1.20.1": {
      "forge-server.jar": "https://example.com/forge-1.20.1-server.jar",
      "forge-installer.txt": "https://example.com/forge-1.20.1-installer.txt",
      "forge-installer.jar": "https://example.com/forge-1.20.1-installer.jar"
    }
  }
}"#,
        )
        .unwrap();

        let group = parsed.get_type_by_name("forge").unwrap();
        let link = group.get_link_by_version("1.20.1").unwrap();

        assert_eq!(link.file_name, "forge-installer.jar");
        assert_eq!(link.url, "https://example.com/forge-1.20.1-installer.jar");
    }

    #[test]
    fn get_link_by_version_trims_requested_version() {
        let parsed = parse_base_download_links(
            r#"{
  "types": ["paper"],
  "paper": {
    "versions": ["1.20.1"],
    "1.20.1": {
      "paper-1.20.1.jar": "https://example.com/paper-1.20.1.jar"
    }
  }
}"#,
        )
        .unwrap();

        let group = parsed.get_type_by_name("paper").unwrap();
        let link = group.get_link_by_version(" 1.20.1 ").unwrap();

        assert_eq!(link.file_name, "paper-1.20.1.jar");
    }

    #[test]
    fn parse_base_download_links_rejects_non_string_type_name() {
        let error = parse_base_download_links(
            r#"{
  "types": [123],
  "paper": {
    "versions": ["1.20.1"],
    "1.20.1": {
      "paper-1.20.1.jar": "https://example.com/paper-1.20.1.jar"
    }
  }
}"#,
        )
        .expect_err("non-string type names should fail parsing");

        assert!(error.contains("types[]"));
    }

    #[test]
    fn parse_base_download_links_rejects_non_string_version_name() {
        let error = parse_base_download_links(
            r#"{
  "types": ["paper"],
  "paper": {
    "versions": [123],
    "1.20.1": {
      "paper-1.20.1.jar": "https://example.com/paper-1.20.1.jar"
    }
  }
}"#,
        )
        .expect_err("non-string versions should fail parsing");

        assert!(error.contains("versions[]"));
    }

    #[test]
    fn parse_base_download_links_rejects_non_string_url_value() {
        let error = parse_base_download_links(
            r#"{
  "types": ["paper"],
  "paper": {
    "versions": ["1.20.1"],
    "1.20.1": {
      "paper-1.20.1.jar": {"url": "https://example.com/paper-1.20.1.jar"}
    }
  }
}"#,
        )
        .expect_err("non-string url values should fail parsing");

        assert!(error.contains("paper.paper-1.20.1.jar"));
    }

    #[test]
    fn parse_base_download_links_rejects_missing_type_detail_node() {
        let error = parse_base_download_links(
            r#"{
  "types": ["paper"]
}"#,
        )
        .expect_err("listed type without detail node should not be silently ignored");

        assert!(error.contains("types.paper 缺失"), "unexpected error: {}", error);
    }

    #[test]
    fn parse_base_download_links_rejects_missing_version_file_mapping_object() {
        let error = parse_base_download_links(
            r#"{
  "types": ["paper"],
  "paper": {
    "versions": ["1.20.1"]
  }
}"#,
        )
        .expect_err("listed version without file mapping should not be silently ignored");

        assert!(error.contains("paper.1.20.1 必须是对象"), "unexpected error: {}", error);
    }
}
