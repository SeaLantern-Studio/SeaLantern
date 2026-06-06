use std::collections::HashMap;

use serde::Deserialize;

use crate::version::{choose_more_specific_bucket, compare_version_keys_numeric};

#[derive(Debug, Deserialize)]
pub(crate) struct StarterLinksPayload {
    #[serde(default)]
    pub(crate) types: StarterTypes,
    #[serde(flatten)]
    pub(crate) cores: HashMap<String, StarterCoreNode>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StarterCoreNode {
    #[serde(rename = "versions")]
    #[serde(default)]
    pub(crate) _versions: Option<serde_json::Value>,
    #[serde(flatten)]
    pub(crate) version_files: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum StarterTypes {
    List(Vec<String>),
    Map(HashMap<String, serde_json::Value>),
    Other(serde_json::Value),
}

impl Default for StarterTypes {
    fn default() -> Self {
        Self::Other(serde_json::Value::Null)
    }
}

pub(crate) fn validate_starter_links_json(body: &[u8]) -> Result<(), String> {
    serde_json::from_slice::<StarterLinksPayload>(body)
        .map(|_| ())
        .map_err(|e| format!("解析 Starter 下载信息失败: {}", e))
}

pub(crate) fn resolve_installer_url_from_nested_json(
    payload: &StarterLinksPayload,
    core_key: &str,
    target_version: &str,
) -> Result<Option<String>, String> {
    if !type_list_contains_core(&payload.types, core_key) {
        return Ok(None);
    }

    let Some(core_node) = find_core_node(payload, core_key) else {
        return Ok(None);
    };

    if let Some(files) = find_exact_version_files(core_node, target_version) {
        if let Some(url) = select_best_url_from_file_map(files)? {
            return Ok(Some(url));
        }
    }

    let Some(files) = find_prefix_version_files(core_node, target_version) else {
        return Ok(None);
    };

    select_best_url_from_file_map(files)
}

fn find_core_node<'a>(
    payload: &'a StarterLinksPayload,
    core_key: &str,
) -> Option<&'a StarterCoreNode> {
    payload.cores.get(core_key).or_else(|| {
        payload
            .cores
            .iter()
            .find(|(name, _)| name.as_str().eq_ignore_ascii_case(core_key))
            .map(|(_, node)| node)
    })
}

fn find_exact_version_files<'a>(
    core_node: &'a StarterCoreNode,
    target_version: &str,
) -> Option<&'a HashMap<String, String>> {
    core_node
        .version_files
        .iter()
        .find(|(version, _)| version.trim().eq_ignore_ascii_case(target_version))
        .map(|(_, files)| files)
}

fn find_prefix_version_files<'a>(
    core_node: &'a StarterCoreNode,
    target_version: &str,
) -> Option<&'a HashMap<String, String>> {
    let mut with_installer: Option<(&String, &HashMap<String, String>)> = None;
    let mut without_installer: Option<(&String, &HashMap<String, String>)> = None;

    for (version, files) in &core_node.version_files {
        let version_lower = version.trim().to_ascii_lowercase();
        if !version_lower.starts_with(target_version)
            && !target_version.starts_with(version_lower.as_str())
        {
            continue;
        }

        let has_installer = files
            .keys()
            .any(|file_name| file_name.to_ascii_lowercase().contains("installer"));
        if has_installer {
            choose_more_specific_bucket(&mut with_installer, version, files);
        } else {
            choose_more_specific_bucket(&mut without_installer, version, files);
        }
    }

    with_installer.or(without_installer).map(|(_, files)| files)
}

fn type_list_contains_core(types: &StarterTypes, core_key: &str) -> bool {
    match types {
        StarterTypes::List(values) => values
            .iter()
            .any(|value| value.eq_ignore_ascii_case(core_key)),
        StarterTypes::Map(values) => values
            .keys()
            .any(|name| name.eq_ignore_ascii_case(core_key)),
        StarterTypes::Other(value) => value.is_null(),
    }
}

fn select_best_url_from_file_map(
    files_obj: &HashMap<String, String>,
) -> Result<Option<String>, String> {
    if let Some(url) = select_url_by(
        files_obj,
        |file_name| file_name.contains("installer") && file_name.ends_with(".jar"),
        "installer JAR",
    )? {
        return Ok(Some(url));
    }

    if let Some(url) =
        select_url_by(files_obj, |file_name| file_name.contains("installer"), "installer")?
    {
        return Ok(Some(url));
    }

    if let Some(url) = select_url_by(files_obj, |file_name| file_name.ends_with(".jar"), "JAR")? {
        return Ok(Some(url));
    }

    select_url_by(files_obj, |_| true, "download")
}

fn select_url_by<F>(
    files_obj: &HashMap<String, String>,
    predicate: F,
    candidate_kind: &str,
) -> Result<Option<String>, String>
where
    F: Fn(&str) -> bool,
{
    let mut matched_empty_url: Option<String> = None;
    let candidates = files_obj
        .iter()
        .filter_map(|(file_name, url)| {
            let normalized_name = file_name.to_ascii_lowercase();
            if !predicate(&normalized_name) {
                return None;
            }

            let normalized_url = url.trim();
            if normalized_url.is_empty() {
                if matched_empty_url.is_none() {
                    matched_empty_url = Some(file_name.clone());
                }
                return None;
            }

            Some((normalized_name, normalized_url.to_string()))
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        if let Some(file_name) = matched_empty_url {
            return Err(format!(
                "Starter 下载信息无效: 匹配到 {} 文件 '{}' 但 URL 为空",
                candidate_kind, file_name
            ));
        }

        return Ok(None);
    }

    Ok(candidates
        .into_iter()
        .max_by(|left, right| {
            compare_version_keys_numeric(&left.0, &right.0)
                .then_with(|| compare_version_keys_numeric(&right.1, &left.1))
        })
        .map(|(_, url)| url))
}
