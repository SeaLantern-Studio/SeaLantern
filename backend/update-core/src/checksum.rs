use std::path::Path;

use crate::types::ReleaseAsset;

#[cfg_attr(debug_assertions, allow(dead_code))]
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

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

#[cfg_attr(debug_assertions, allow(dead_code))]
pub(crate) fn find_sha256_assets<'a>(
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

#[cfg_attr(debug_assertions, allow(dead_code))]
async fn fetch_sha256_from_asset(
    client: &reqwest::Client,
    hash_asset: &ReleaseAsset,
    target_name: &str,
) -> Result<Option<String>, String> {
    let response = client
        .get(&hash_asset.browser_download_url)
        .send()
        .await
        .map_err(|e| format!("checksum request failed for {}: {}", hash_asset.name, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "checksum asset {} returned status {}",
            hash_asset.name,
            response.status()
        ));
    }

    if let Some(content_length) = response.content_length() {
        if content_length > 1024 * 1024 {
            return Err(format!(
                "checksum asset {} is too large: {} bytes",
                hash_asset.name, content_length
            ));
        }
    }

    let content = response
        .text()
        .await
        .map_err(|e| format!("checksum asset {} body read failed: {}", hash_asset.name, e))?;
    Ok(parse_sha256_from_checksum_content(&content, target_name))
}

#[cfg_attr(debug_assertions, allow(dead_code))]
pub(crate) async fn resolve_asset_sha256(
    client: &reqwest::Client,
    assets: &[ReleaseAsset],
    target_asset: &ReleaseAsset,
) -> Result<Option<String>, String> {
    let candidates = find_sha256_assets(assets, &target_asset.name);
    for hash_asset in candidates {
        if let Some(hash) = fetch_sha256_from_asset(client, hash_asset, &target_asset.name).await? {
            return Ok(Some(hash));
        }
    }

    Ok(None)
}

#[cfg(test)]
#[path = "checksum_tests.rs"]
mod tests;
