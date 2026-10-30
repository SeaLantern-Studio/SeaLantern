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

        #[cfg(target_os = "linux")]
        {
            let client = client.get_reqwest_client();
            if super::is_arch_linux() {
                return super::arch::check_aur_update_with_client(client, current_version)
                    .await
                    .map_err(|message| UpdateCheckError::ProviderFailed {
                        provider: "arch-aur",
                        message,
                    });
            }

            let github_config = super::get_github_config();
            let (cnb, github) = tokio::join!(
                super::fetch_cnb_release(client, current_version),
                super::fetch_github_release(client, &github_config, current_version)
            );
            return select_linux_result(cnb, github);
        }

        #[cfg(not(target_os = "linux"))]
        {
            super::fetch_github_release(
                client.get_reqwest_client(),
                &super::get_github_config(),
                current_version,
            )
            .await
            .map_err(|message| UpdateCheckError::ProviderFailed { provider: "github", message })
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
        release_notes: None,
        published_at: None,
        source: None,
        sha256: None,
    }
}

#[cfg(any(test, target_os = "linux"))]
fn select_linux_result(
    cnb: Result<UpdateInfo, String>,
    github: Result<UpdateInfo, String>,
) -> Result<UpdateInfo, UpdateCheckError> {
    match (cnb, github) {
        (_, Ok(github_info)) if github_info.has_update => Ok(github_info),
        (Ok(cnb_info), _) => Ok(cnb_info),
        (Err(_), Ok(github_info)) => Ok(github_info),
        (Err(cnb), Err(github)) => Err(UpdateCheckError::ProvidersFailed { cnb, github }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn update(source: &str, has_update: bool) -> UpdateInfo {
        UpdateInfo {
            has_update,
            latest_version: if has_update { "2.0.0" } else { "1.0.0" }.to_owned(),
            current_version: "1.0.0".to_owned(),
            download_url: has_update.then(|| "https://example.com/update".to_owned()),
            release_notes: None,
            published_at: None,
            source: Some(source.to_owned()),
            sha256: None,
        }
    }

    #[test]
    fn linux_selection_prefers_available_github_release() {
        let selected = select_linux_result(Ok(update("cnb", true)), Ok(update("github", true)))
            .expect("select update");

        assert_eq!(selected.source.as_deref(), Some("github"));
    }

    #[test]
    fn linux_selection_falls_back_to_cnb() {
        let selected = select_linux_result(Ok(update("cnb", true)), Err("offline".to_owned()))
            .expect("select update");

        assert_eq!(selected.source.as_deref(), Some("cnb"));
    }

    #[test]
    fn linux_selection_preserves_both_failures() {
        let error =
            select_linux_result(Err("cnb failed".to_owned()), Err("github failed".to_owned()))
                .expect_err("both providers should fail");

        assert!(matches!(
            error,
            UpdateCheckError::ProvidersFailed { ref cnb, ref github }
                if cnb == "cnb failed" && github == "github failed"
        ));
    }

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
}
