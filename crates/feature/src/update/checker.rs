//! 多来源应用更新检查编排。

use async_trait::async_trait;
use sealantern_infra::net::{ClientProvider, NetClient};

use super::error::UpdateCheckError;
use super::types::UpdateInfo;

/// 应用更新检查能力。
#[async_trait]
pub trait UpdateChecker: Send + Sync {
    /// 检查当前平台是否存在新版本。
    async fn check(&self, current_version: &str) -> Result<UpdateInfo, UpdateCheckError>;
}

/// 基于 SeaLantern 官方发布源的更新检查器。
///
/// 持有客户端获取器（provider），每次检查时获取当前全局客户端，
/// 避免缓存固定客户端导致代理更新不生效。
pub struct ReleaseUpdateChecker {
    client_provider: ClientProvider,
}

impl ReleaseUpdateChecker {
    /// 使用全局网络客户端构造检查器。
    pub fn new() -> Result<Self, UpdateCheckError> {
        Ok(Self {
            client_provider: sealantern_infra::net::global_client_provider(),
        })
    }

    /// 使用客户端获取器构造检查器，供上层注入代理或测试替换。
    pub fn with_provider(client_provider: ClientProvider) -> Self {
        Self { client_provider }
    }

    /// 使用既有 HTTP 客户端构造检查器，兼容旧调用与测试注入。
    pub fn with_client(client: NetClient) -> Self {
        Self {
            client_provider: Box::new(move || Ok(client.clone())),
        }
    }
}

#[async_trait]
impl UpdateChecker for ReleaseUpdateChecker {
    async fn check(&self, current_version: &str) -> Result<UpdateInfo, UpdateCheckError> {
        if debug_checks_disabled() {
            crate::observability::update_check_started("disabled", current_version);
            crate::observability::update_check_completed("disabled", false, Some(current_version));
            return Ok(no_update(current_version));
        }

        let client = (self.client_provider)()
            .map_err(|source| UpdateCheckError::ClientInitialization { source })?;
        let client = client.get_reqwest_client();

        // Arch Linux 走 AUR 独占流程，不再查询发布页。
        #[cfg(target_os = "linux")]
        {
            if super::is_arch_linux() {
                return super::arch::check_aur_update_with_client(client, current_version)
                    .await
                    .map_err(|message| UpdateCheckError::ProviderFailed {
                        provider: "arch-aur",
                        message,
                    })
                    .map(with_mirror_url);
            }
        }

        match super::fetch_github_release(client, &super::get_github_config(), current_version)
            .await
        {
            Ok(info) => Ok(with_mirror_url(info)),
            Err(github) => {
                // GitHub 不可达时才付出解析镜像页的代价，成功则直接采用其结果。
                match super::fetch_cnb_release(client, current_version).await {
                    Ok(info) => {
                        crate::observability::update_check_cnb_fallback_triggered(&github);
                        Ok(with_mirror_url(info))
                    }
                    Err(cnb) => Err(UpdateCheckError::ProvidersFailed { cnb, github }),
                }
            }
        }
    }
}

/// 非发布构建默认跳过远程检查，避免开发期反复请求发布源。
///
/// 设置环境变量 `SEALANTERN_UPDATE_CHECK` 可在开发构建中强制走真实检查流程，
/// 用于验证多来源选择、下载源切换等依赖真实网络的逻辑。
fn debug_checks_disabled() -> bool {
    cfg!(debug_assertions) && std::env::var_os("SEALANTERN_UPDATE_CHECK").is_none()
}

fn no_update(current_version: &str) -> UpdateInfo {
    UpdateInfo {
        has_update: false,
        latest_version: current_version.to_owned(),
        current_version: current_version.to_owned(),
        download_url: None,
        mirror_url: None,
        release_notes: None,
        published_at: None,
        source: None,
        sha256: None,
    }
}

/// 为 GitHub 来源的检查结果补齐镜像下载地址。
///
/// 仅当结果来自 GitHub、存在下载地址且能从该地址还原出发布标签与文件名时才补齐；
/// 其余情况保持 `None`，下载流程退化为单源。
fn with_mirror_url(mut info: UpdateInfo) -> UpdateInfo {
    if info.source.as_deref() == Some("github")
        && let Some(download_url) = info.download_url.as_deref()
        && let Some((tag, file_name)) = parse_github_download_parts(download_url)
    {
        info.mirror_url = super::cnb::mirror_download_url(&tag, &file_name);
    }

    info
}

/// 从 GitHub 发布资源地址还原发布标签与文件名。
///
/// 地址形如 `https://github.com/{owner}/{repo}/releases/download/{tag}/{file}`。
fn parse_github_download_parts(url: &str) -> Option<(String, String)> {
    let (_, remainder) = url.split_once("/releases/download/")?;
    let (tag, file_name) = remainder.split_once('/')?;

    let tag = tag.trim();
    let file_name = file_name
        .split(['?', '#'])
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    if tag.is_empty() {
        return None;
    }

    let tag = urlencoding::decode(tag).ok()?.into_owned();
    Some((tag, file_name.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(debug_assertions)]
    #[tokio::test]
    async fn debug_checker_skips_remote_requests() {
        if std::env::var_os("SEALANTERN_UPDATE_CHECK").is_some() {
            // 开发构建强制启用真实检查时，本用例的前提不成立。
            return;
        }

        let checker = ReleaseUpdateChecker::new().expect("create update checker");
        let info = checker.check("1.2.3").await.expect("check update");

        assert!(!info.has_update);
        assert_eq!(info.current_version, "1.2.3");
        assert_eq!(info.latest_version, "1.2.3");
        assert!(info.source.is_none());
    }

    #[test]
    fn download_parts_split_standard_url() {
        let parts = parse_github_download_parts(
            "https://github.com/SeaLantern-Studio/SeaLantern/releases/download/v1.3.1/Sea.Lantern_1.3.1_x64_zh-CN.msi",
        )
        .expect("parse download parts");

        assert_eq!(parts.0, "v1.3.1");
        assert_eq!(parts.1, "Sea.Lantern_1.3.1_x64_zh-CN.msi");
    }

    #[test]
    fn download_parts_keep_version_tag_prefix() {
        // 从 GitHub 地址还原出的标签必须保留 `v` 前缀，CNB 路由依赖它。
        let (tag, _) =
            parse_github_download_parts("https://github.com/o/r/releases/download/v2.0.0/app.zip")
                .expect("parse download parts");

        assert_eq!(tag, "v2.0.0");
    }

    #[test]
    fn download_parts_strip_query_and_fragment() {
        let (_, file) = parse_github_download_parts(
            "https://github.com/o/r/releases/download/v1.0.0/app.zip?download=1",
        )
        .expect("parse download parts");
        assert_eq!(file, "app.zip");

        let (_, file) = parse_github_download_parts(
            "https://github.com/o/r/releases/download/v1.0.0/app.zip#section",
        )
        .expect("parse download parts");
        assert_eq!(file, "app.zip");
    }

    #[test]
    fn download_parts_decode_encoded_tag() {
        // 标签里出现空格等字符时会被编码，还原后应得到原始文本。
        let (tag, _) = parse_github_download_parts(
            "https://github.com/o/r/releases/download/v1.0.0%20rc1/app.zip",
        )
        .expect("parse download parts");

        assert_eq!(tag, "v1.0.0 rc1");
    }

    #[test]
    fn download_parts_reject_unrelated_urls() {
        // 非 releases/download 结构的地址不应产出镜像来源。
        let cases = [
            "https://github.com/o/r/releases/tag/v1.0.0",
            "https://github.com/o/r/archive/refs/tags/v1.0.0.zip",
            "https://example.com/app.zip",
            "not a url",
        ];
        for url in cases {
            assert!(parse_github_download_parts(url).is_none(), "{url} should be rejected");
        }
    }

    #[test]
    fn download_parts_reject_empty_tag_or_file() {
        // 空标签、空文件名、以及只有查询串没有文件名的情况。
        let cases = [
            "https://github.com/o/r/releases/download//app.zip",
            "https://github.com/o/r/releases/download/v1.0.0/",
            "https://github.com/o/r/releases/download/v1.0.0/?x=1",
            "https://github.com/o/r/releases/download/v1.0.0",
        ];
        for url in cases {
            assert!(parse_github_download_parts(url).is_none(), "{url} should be rejected");
        }
    }

    #[test]
    fn mirror_url_only_filled_for_github_results() {
        // 只有 source 为 github 且地址可还原时才补镜像，其它来源原样返回。
        let base = UpdateInfo {
            has_update: true,
            latest_version: "2.0.0".to_string(),
            current_version: "1.0.0".to_string(),
            download_url: Some(
                "https://github.com/o/r/releases/download/v2.0.0/app.zip".to_string(),
            ),
            mirror_url: None,
            release_notes: None,
            published_at: None,
            source: Some("github".to_string()),
            sha256: None,
        };

        assert!(with_mirror_url(base.clone()).mirror_url.is_some());

        let non_github = UpdateInfo {
            source: Some("aur".to_string()),
            ..base.clone()
        };
        assert!(with_mirror_url(non_github).mirror_url.is_none());

        let no_source = UpdateInfo { source: None, ..base.clone() };
        assert!(with_mirror_url(no_source).mirror_url.is_none());

        let no_download = UpdateInfo { download_url: None, ..base };
        assert!(with_mirror_url(no_download).mirror_url.is_none());
    }
}
