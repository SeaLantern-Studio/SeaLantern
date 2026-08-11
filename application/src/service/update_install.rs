//! 应用更新安装服务实现。
//!
//! 实现 [`sealantern_interface::UpdateInstallService`] 能力端口，组合
//! `extra` 的更新落盘能力（[`download_update_file_without_events`]、
//! [`write_pending_update`]、[`check_pending_update`] 等），向宿主提供
//! 更新文件下载、待安装记录查询 / 清理与安装进程拉起。
//!
//! 错误分层：底层下载 / 落盘 / 查询失败统一收敛为
//! [`UpdateInstallServiceError::OperationFailed`]；URL 非法、版本号为空、
//! 哈希格式非法等参数问题收敛为 [`UpdateInstallServiceError::InvalidInput`]；
//! 非 Windows 平台无法拉起安装进程，返回
//! [`UpdateInstallServiceError::Unsupported`]。

use async_trait::async_trait;
use sealantern_extra::update::{
    check_pending_update, clear_pending_update, download_update_file_without_events,
    get_pending_update_file, get_update_cache_dir, write_pending_update, PendingUpdate,
};
use sealantern_interface::{UpdateInstallService, UpdateInstallServiceError};

/// 基于 `extra` 更新落盘能力的更新安装服务实现。
#[derive(Debug, Default)]
pub struct CoreUpdateInstallService;

#[async_trait]
impl UpdateInstallService for CoreUpdateInstallService {
    /// 下载更新文件并登记为待安装。
    ///
    /// 参数校验：`url` 必须非空且为 http / https 协议，`version` 不能为空，
    /// `expected_hash` 若存在必须是偶数长度的十六进制字符串，不满足时视为
    /// 非法输入；下载成功后把文件路径与版本写入待安装记录，供应用重启后的
    /// 安装流程读取。
    async fn download(
        &self,
        url: String,
        expected_hash: Option<String>,
        version: String,
    ) -> Result<String, UpdateInstallServiceError> {
        // URL 基本校验：非空 + 基础协议检查。
        let trimmed_url = url.trim();
        if trimmed_url.is_empty()
            || !(trimmed_url.starts_with("http://") || trimmed_url.starts_with("https://"))
        {
            return Err(UpdateInstallServiceError::InvalidInput);
        }
        // 版本号校验（原有语义保留）。
        if version.trim().is_empty() {
            return Err(UpdateInstallServiceError::InvalidInput);
        }
        // 期望哈希格式校验：必须是偶数长度的十六进制字符串。
        let trimmed_hash = expected_hash.as_deref().map(str::trim);
        if let Some(hash) = trimmed_hash {
            if hash.is_empty() || !hash.chars().all(|c| c.is_ascii_hexdigit()) || hash.len() % 2 != 0 {
                return Err(UpdateInstallServiceError::InvalidInput);
            }
        }
        let path = download_update_file_without_events(
            trimmed_url.to_string(),
            trimmed_hash.map(str::to_string),
            get_update_cache_dir(),
        )
        .await
        .map_err(|_| UpdateInstallServiceError::OperationFailed)?;
        write_pending_update(&get_pending_update_file(), &path, version)
            .map_err(|_| UpdateInstallServiceError::OperationFailed)?;
        Ok(path)
    }

    /// 查询是否存在待安装更新（下载完成但尚未安装）。
    async fn pending(&self) -> Result<Option<PendingUpdate>, UpdateInstallServiceError> {
        check_pending_update()
            .await
            .map_err(|_| UpdateInstallServiceError::OperationFailed)
    }

    /// 清除待安装更新记录。
    async fn clear_pending(&self) -> Result<(), UpdateInstallServiceError> {
        clear_pending_update()
            .await
            .map_err(|_| UpdateInstallServiceError::OperationFailed)
    }

    /// 启动更新安装流程。
    ///
    /// Windows 下通过提权进程拉起安装器；安装完成后由底层监视进程
    /// 删除安装文件与待安装记录，并重启主应用。其他平台暂不支持
    /// 进程级安装，直接返回不支持。
    async fn install(
        &self,
        file_path: String,
        arguments: Vec<String>,
    ) -> Result<(), UpdateInstallServiceError> {
        #[cfg(target_os = "windows")]
        {
            let refs = arguments.iter().map(String::as_str).collect::<Vec<_>>();
            // 提权启动安装器，并把安装文件与待安装记录路径交给底层监视进程，
            // 使其在安装完成后执行清理并重启主应用。
            sealantern_extra::update::spawn_elevated_windows_process(
                &file_path,
                &refs,
                Some(&file_path),
                get_pending_update_file().to_str(),
            )
            .map_err(|_| UpdateInstallServiceError::OperationFailed)
        }
        #[cfg(not(target_os = "windows"))]
        {
            // 非 Windows 平台暂无安装通道，参数仅做丢弃处理。
            let _ = (file_path, arguments);
            Err(UpdateInstallServiceError::Unsupported)
        }
    }
}
