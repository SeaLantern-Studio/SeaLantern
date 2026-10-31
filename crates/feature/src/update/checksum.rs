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

/// 从发布平台给出的摘要中提取 SHA-256。
///
/// 摘要格式为 `算法:<hex>`，只接受 `sha256`；算法不匹配或长度、字符不合法时
/// 返回 `None`，由调用方回落到下载校验文件。
pub fn sha256_from_github_digest(digest: &str) -> Option<String> {
    let (algorithm, value) = digest.split_once(':')?;
    // 两侧都做 trim：摘要值在下面处理，算法名同样不应受空白影响。
    if !algorithm.trim().eq_ignore_ascii_case("sha256") {
        return None;
    }

    let value = value.trim();
    is_sha256_hex(value).then(|| value.to_ascii_lowercase())
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

    if let Some(content_length) = response.content_length()
        && content_length > 1024 * 1024
    {
        return None;
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

    const SAMPLE: &str = "427a87e592cbc2843cbf0ba8e5c928297c0e198036e48e1c55da8964e71700db";

    #[test]
    fn digest_parses_standard_format() {
        assert_eq!(
            sha256_from_github_digest(&format!("sha256:{SAMPLE}")),
            Some(SAMPLE.to_string())
        );
    }

    #[test]
    fn digest_accepts_uppercase_algorithm_and_hex() {
        // 算法名与摘要都需要归一为小写。
        let upper = format!("SHA256:{}", SAMPLE.to_ascii_uppercase());
        assert_eq!(sha256_from_github_digest(&upper), Some(SAMPLE.to_string()));
    }

    #[test]
    fn digest_tolerates_surrounding_whitespace() {
        assert_eq!(
            sha256_from_github_digest(&format!(" sha256:  {SAMPLE} ")),
            Some(SAMPLE.to_string())
        );
    }

    #[test]
    fn digest_rejects_other_algorithms() {
        for algorithm in ["sha1", "sha512", "md5", "blake3"] {
            let digest = format!("{algorithm}:{SAMPLE}");
            assert!(sha256_from_github_digest(&digest).is_none(), "{algorithm} should be rejected");
        }
    }

    #[test]
    fn digest_rejects_malformed_values() {
        // 缺少算法前缀、前缀为空、摘要长度不足、含非十六进制字符。
        let cases = [
            SAMPLE.to_string(),
            format!(":{SAMPLE}"),
            "sha256:".to_string(),
            "sha256:abc".to_string(),
            format!("sha256:{}", &SAMPLE[..63]),
            format!("sha256:{}g", &SAMPLE[..63]),
        ];
        for digest in cases {
            assert!(sha256_from_github_digest(&digest).is_none(), "{digest} should be rejected");
        }
    }
}
