use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::Deserialize;

// 负责 Starter 模式下 --installer 链接的获取链路：
// 1) 24 小时缓存读取/刷新 jar_lfs_links.json
// 2) 解析 CNB 分层 JSON（types -> core -> version -> file/url）
// 3) 按固定优先级选择安装器下载 URL
const STARTER_INSTALLER_LINKS_URL: &str = "https://cnb.cool/SeaLantern-studio/ServerCore-Mirror/-/releases/download/26.02.27/jar_lfs_links.json";
const STARTER_INSTALLER_LINKS_FILE: &str = "jar_lfs_links.json";
const STARTER_INSTALLER_LINKS_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Deserialize)]
struct StarterLinksPayload {
    #[serde(default)]
    types: StarterTypes,
    #[serde(flatten)]
    cores: HashMap<String, StarterCoreNode>,
}

#[derive(Debug, Deserialize)]
struct StarterCoreNode {
    #[serde(rename = "versions")]
    #[serde(default)]
    _versions: Option<serde_json::Value>,
    #[serde(flatten)]
    version_files: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StarterTypes {
    List(Vec<String>),
    Map(HashMap<String, serde_json::Value>),
    Other(serde_json::Value),
}

impl Default for StarterTypes {
    fn default() -> Self {
        Self::Other(serde_json::Value::Null)
    }
}

pub fn fetch_starter_installer_url(
    core_type_key: &str,
    mc_version: &str,
) -> Result<(String, Option<String>), String> {
    // 先走本地缓存；仅在缓存缺失或超过 24 小时时重新拉取远端 JSON。
    let data_dir = PathBuf::from(crate::utils::path::get_or_create_app_data_dir());
    std::fs::create_dir_all(&data_dir).map_err(|e| format!("创建软件目录失败: {}", e))?;
    let links_file_path = data_dir.join(STARTER_INSTALLER_LINKS_FILE);
    let body = load_or_refresh_starter_links_json(&links_file_path)?;

    let payload: StarterLinksPayload =
        serde_json::from_slice(&body).map_err(|e| format!("解析 Starter 下载信息失败: {}", e))?;
    let core_key = core_type_key.trim().to_ascii_lowercase();
    let target_version = mc_version.trim().to_ascii_lowercase();
    if core_key.is_empty() || target_version.is_empty() {
        return Err("Starter 下载参数缺少核心类型或 MC 版本".to_string());
    }

    if let Some(installer_url) =
        resolve_installer_url_from_nested_json(&payload, &core_key, &target_version)
    {
        return Ok((installer_url, None));
    }

    Err(format!(
        "未在 CNB 镜像中找到匹配下载链接：core={}, version={}",
        core_type_key, mc_version
    ))
}

fn load_or_refresh_starter_links_json(links_file_path: &Path) -> Result<Vec<u8>, String> {
    if should_use_cached_links_file(links_file_path)? {
        return match read_and_validate_cached_links_file(links_file_path) {
            Ok(body) => Ok(body),
            Err(local_error) => {
                fetch_and_cache_starter_links_json(links_file_path).map_err(|refresh_error| {
                    format!(
                        "读取本地 Starter 下载信息失败: {}; 刷新远端 Starter 下载信息也失败: {}",
                        local_error, refresh_error
                    )
                })
            }
        };
    }

    match fetch_and_cache_starter_links_json(links_file_path) {
        Ok(body) => Ok(body),
        Err(refresh_error) => {
            if links_file_path.is_file() {
                return read_and_validate_cached_links_file(links_file_path).map_err(
                    |local_error| {
                        format!("{}；且本地 Starter 下载信息不可用: {}", refresh_error, local_error)
                    },
                );
            }
            Err(refresh_error)
        }
    }
}

fn read_and_validate_cached_links_file(links_file_path: &Path) -> Result<Vec<u8>, String> {
    let body = std::fs::read(links_file_path)
        .map_err(|e| format!("读取本地 Starter 下载信息失败: {}", e))?;
    validate_starter_links_json(&body)?;
    Ok(body)
}

fn validate_starter_links_json(body: &[u8]) -> Result<(), String> {
    serde_json::from_slice::<StarterLinksPayload>(body)
        .map(|_| ())
        .map_err(|e| format!("解析 Starter 下载信息失败: {}", e))
}

fn fetch_and_cache_starter_links_json(links_file_path: &Path) -> Result<Vec<u8>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建 Starter 请求客户端失败: {}", e))?;
    let response = client
        .get(STARTER_INSTALLER_LINKS_URL)
        .send()
        .map_err(|e| format!("请求 Starter 下载信息失败: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "Starter 下载接口返回异常状态: {} ({})",
            status, STARTER_INSTALLER_LINKS_URL
        ));
    }

    let body = response
        .bytes()
        .map_err(|e| format!("读取 Starter 下载信息失败: {}", e))?
        .to_vec();

    validate_starter_links_json(&body)
        .map_err(|e| format!("远端 Starter 下载信息校验失败: {}", e))?;

    std::fs::write(links_file_path, &body)
        .map_err(|e| format!("写入 Starter 下载信息失败: {}", e))?;

    Ok(body)
}

fn should_use_cached_links_file(links_file_path: &Path) -> Result<bool, String> {
    let metadata = match std::fs::metadata(links_file_path) {
        Ok(metadata) => metadata,
        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                return Ok(false);
            }
            return Err(format!("读取 Starter 缓存文件元数据失败: {}", error));
        }
    };

    let modified_time = metadata
        .modified()
        .map_err(|e| format!("读取 Starter 缓存文件时间失败: {}", e))?;
    let age = SystemTime::now()
        .duration_since(modified_time)
        .unwrap_or(Duration::ZERO);

    Ok(age <= STARTER_INSTALLER_LINKS_CACHE_TTL)
}

fn resolve_installer_url_from_nested_json(
    payload: &StarterLinksPayload,
    core_key: &str,
    target_version: &str,
) -> Option<String> {
    if !type_list_contains_core(&payload.types, core_key) {
        return None;
    }

    let core_node = find_core_node(payload, core_key)?;

    // 版本优先级：先精确版本，再前缀兜底版本。
    if let Some(files) = find_exact_version_files(core_node, target_version) {
        if let Some(url) = select_best_url_from_file_map(files) {
            return Some(url);
        }
    }

    let files = find_prefix_version_files(core_node, target_version)?;
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

fn choose_more_specific_bucket<'a>(
    selected: &mut Option<(&'a String, &'a HashMap<String, String>)>,
    version: &'a String,
    files: &'a HashMap<String, String>,
) {
    let should_replace = match selected {
        None => true,
        Some((selected_version, selected_files)) => {
            files.len() > selected_files.len()
                || (files.len() == selected_files.len()
                    && compare_version_keys_numeric(version, selected_version).is_gt())
        }
    };

    if should_replace {
        *selected = Some((version, files));
    }
}

fn compare_version_keys_numeric(left: &str, right: &str) -> Ordering {
    let mut left_index = 0usize;
    let mut right_index = 0usize;

    loop {
        let left_token = next_version_token(left, &mut left_index);
        let right_token = next_version_token(right, &mut right_index);

        let ordering = match (left_token, right_token) {
            (None, None) => return left.cmp(right),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (Some(VersionToken::Numeric(left_num)), Some(VersionToken::Numeric(right_num))) => {
                compare_numeric_token(left_num, right_num)
            }
            (Some(VersionToken::Text(left_text)), Some(VersionToken::Text(right_text))) => {
                compare_text_token(left_text, right_text)
            }
            (Some(VersionToken::Numeric(_)), Some(VersionToken::Text(_))) => Ordering::Greater,
            (Some(VersionToken::Text(_)), Some(VersionToken::Numeric(_))) => Ordering::Less,
        };

        if ordering != Ordering::Equal {
            return ordering;
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum VersionToken<'a> {
    Numeric(&'a str),
    Text(&'a str),
}

fn next_version_token<'a>(value: &'a str, index: &mut usize) -> Option<VersionToken<'a>> {
    let bytes = value.as_bytes();

    while *index < bytes.len() && !bytes[*index].is_ascii_alphanumeric() {
        *index += 1;
    }

    if *index >= bytes.len() {
        return None;
    }

    let start = *index;
    if bytes[*index].is_ascii_digit() {
        while *index < bytes.len() && bytes[*index].is_ascii_digit() {
            *index += 1;
        }
        return Some(VersionToken::Numeric(&value[start..*index]));
    }

    while *index < bytes.len() && bytes[*index].is_ascii_alphabetic() {
        *index += 1;
    }
    Some(VersionToken::Text(&value[start..*index]))
}

fn compare_numeric_token(left: &str, right: &str) -> Ordering {
    let left_trimmed = left.trim_start_matches('0');
    let right_trimmed = right.trim_start_matches('0');
    let left_normalized = if left_trimmed.is_empty() {
        "0"
    } else {
        left_trimmed
    };
    let right_normalized = if right_trimmed.is_empty() {
        "0"
    } else {
        right_trimmed
    };

    left_normalized
        .len()
        .cmp(&right_normalized.len())
        .then_with(|| left_normalized.cmp(right_normalized))
}

fn compare_text_token(left: &str, right: &str) -> Ordering {
    let case_insensitive = left.to_ascii_lowercase().cmp(&right.to_ascii_lowercase());
    if case_insensitive != Ordering::Equal {
        return case_insensitive;
    }
    left.cmp(right)
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

fn select_best_url_from_file_map(files_obj: &HashMap<String, String>) -> Option<String> {
    // 文件优先级：installer > .jar > 任意可用 URL。
    if let Some(url) = select_url_by(files_obj, |file_name| file_name.contains("installer")) {
        return Some(url);
    }

    if let Some(url) = select_url_by(files_obj, |file_name| file_name.ends_with(".jar")) {
        return Some(url);
    }

    select_url_by(files_obj, |_| true)
}

fn select_url_by<F>(files_obj: &HashMap<String, String>, predicate: F) -> Option<String>
where
    F: Fn(&str) -> bool,
{
    files_obj
        .iter()
        .filter_map(|(file_name, url)| {
            let normalized_name = file_name.to_ascii_lowercase();
            if !predicate(&normalized_name) {
                return None;
            }

            let normalized_url = url.trim();
            if normalized_url.is_empty() {
                return None;
            }

            Some(normalized_url.to_string())
        })
        .min()
}
