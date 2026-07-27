//! 更新文件校验和验证模块。
//!
//! 提供 SHA256 校验和文件的解析、匹配和远程获取功能。
//! 支持常见的校验和文件名模式（`.sha256`、`.sha256sum`、`.sha256.txt` 等）。
//!
//! 校验和解析的基础能力（`is_sha256_hex`、`find_sha256_in_line`、
//! `parse_sha256_from_checksum_content`）委托给 `sealantern_infra::fs`。
//!
//! # 解析策略
//!
//! 采用多级候选匹配策略：精确名称匹配 > 目标文件名匹配 > 通用哈希文件匹配。

use std::path::Path;

use super::types::ReleaseAsset;
use crate::observability;

pub use sealantern_infra::fs::{
    find_sha256_in_line, is_sha256_hex, parse_sha256_from_checksum_content,
};

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
        .inspect_err(|e| {
            observability::update_api_request_failed(
                "github",
                "fetch_sha256_asset",
                None,
                &format!("{e}"),
            );
        })
        .ok()?;

    if !response.status().is_success() {
        observability::update_api_request_failed(
            "github",
            "fetch_sha256_asset",
            Some(response.status().as_u16()),
            &"non-success status",
        );
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
