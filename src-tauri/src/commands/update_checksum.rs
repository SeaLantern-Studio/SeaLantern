use std::path::Path;

use crate::commands::update_types::ReleaseAsset;

/// 解析 SHA256 校验文件内容
pub fn parse_sha256_from_checksum_content(content: &str, target_name: &str) -> Option<String> {
    let target_lower = target_name.to_ascii_lowercase();
    let target_file_name = Path::new(target_name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(target_name)
        .to_ascii_lowercase();

    let mut single_hash: Option<String> = None;
    let mut hash_line_count = 0_usize;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let hash = match find_sha256_in_line(trimmed) {
            Some(value) => value,
            None => continue,
        };

        hash_line_count += 1;
        if hash_line_count == 1 {
            single_hash = Some(hash.clone());
        } else {
            single_hash = None;
        }

        let line_lower = trimmed.to_ascii_lowercase();
        if line_lower.contains(&target_lower) || line_lower.contains(&target_file_name) {
            return Some(hash);
        }
    }

    if hash_line_count == 1 {
        return single_hash;
    }

    None
}

/// 在行中查找 SHA256 哈希值
fn find_sha256_in_line(line: &str) -> Option<String> {
    for token in line.split(|ch: char| {
        ch.is_ascii_whitespace()
            || matches!(ch, '=' | ':' | ',' | ';' | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>')
    }) {
        let candidate = token.trim_matches(|ch| ch == '*' || ch == '"' || ch == '\'');
        if is_sha256_hex(candidate) {
            return Some(candidate.to_ascii_lowercase());
        }
    }

    None
}

/// 检查字符串是否为有效的 SHA256 十六进制值
fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

/// 查找 SHA256 校验文件资源
pub fn find_sha256_assets<'a>(
    assets: &'a [ReleaseAsset],
    target_name: &str,
) -> Vec<&'a ReleaseAsset> {
    let target_lower = target_name.to_ascii_lowercase();
    let target_file_name = Path::new(target_name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(target_name)
        .to_ascii_lowercase();

    let exact_names = [
        format!("{target_lower}.sha256"),
        format!("{target_lower}.sha256sum"),
        format!("{target_lower}.sha256.txt"),
        format!("{target_lower}.sha256sums"),
    ];

    let mut primary = Vec::new();
    let mut secondary = Vec::new();
    let mut generic = Vec::new();

    for asset in assets {
        let name = asset.name.to_ascii_lowercase();
        if exact_names.iter().any(|item| item == &name) {
            primary.push(asset);
            continue;
        }

        let is_hash_file =
            name.contains("sha256") || name.contains("checksum") || name.contains("checksums");
        if !is_hash_file {
            continue;
        }

        if name.contains(&target_lower) {
            primary.push(asset);
            continue;
        }

        if name.contains(&target_file_name) {
            secondary.push(asset);
        } else {
            generic.push(asset);
        }
    }

    primary.extend(secondary);
    primary.extend(generic);
    primary
}

/// 从校验文件资源中获取 SHA256 值
pub async fn fetch_sha256_from_asset(
    client: &reqwest::Client,
    hash_asset: &ReleaseAsset,
    target_name: &str,
) -> Option<String> {
    let response = client
        .get(&hash_asset.browser_download_url)
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    if let Some(content_length) = response.content_length() {
        if content_length > 1024 * 1024 {
            return None;
        }
    }

    let content = response.text().await.ok()?;
    parse_sha256_from_checksum_content(&content, target_name)
}

/// 解析资源文件的 SHA256 值
pub async fn resolve_asset_sha256(
    client: &reqwest::Client,
    assets: &[ReleaseAsset],
    target_asset: &ReleaseAsset,
) -> Option<String> {
    let candidates = find_sha256_assets(assets, &target_asset.name);
    for hash_asset in candidates {
        if let Some(hash) = fetch_sha256_from_asset(client, hash_asset, &target_asset.name).await {
            return Some(hash);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sha256_matches_target_file() {
        let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let content = format!("{hash}  SeaLantern-setup.exe");
        assert_eq!(
            parse_sha256_from_checksum_content(&content, "SeaLantern-setup.exe"),
            Some(hash.to_string())
        );
    }

    #[test]
    fn parse_sha256_accepts_single_hash_file() {
        let hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        assert_eq!(
            parse_sha256_from_checksum_content(hash, "SeaLantern-setup.exe"),
            Some(hash.to_string())
        );
    }

    #[test]
    fn parse_sha256_rejects_multi_hash_without_target_match() {
        let first = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let second = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
        let content = format!("{first} other.exe\n{second} another.exe");
        assert_eq!(parse_sha256_from_checksum_content(&content, "SeaLantern-setup.exe"), None);
    }
}
