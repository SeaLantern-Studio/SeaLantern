//! 应用更新安装服务实现。
//!
//! 实现 [`sealantern_interface::UpdateInstallService`] 能力端口，组合
//! `extra` 的更新落盘能力（[`download_update_file_without_events`]、
//! [`write_pending_update`]、[`check_pending_update`] 等），向宿主提供
//! 更新文件下载、待安装记录查询 / 清理与安装进程拉起。
//!
//! 错误分层：底层下载 / 落盘 / 查询失败统一收敛为
//! [`UpdateInstallServiceError::OperationFailed`]；版本号为空等非法参数
//! 收敛为 [`UpdateInstallServiceError::InvalidInput`]；非 Windows 平台
//! 无法拉起安装进程，返回 [`UpdateInstallServiceError::Unsupported`]。

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
    /// 版本号为空视为非法输入；下载成功后把文件路径与版本写入待安装
    /// 记录，供应用重启后的安装流程读取。
    async fn download(
        &self,
        url: String,
        expected_hash: Option<String>,
        version: String,
    ) -> Result<String, UpdateInstallServiceError> {
        if version.trim().is_empty() {
            return Err(UpdateInstallServiceError::InvalidInput);
        }
        let path = download_update_file_without_events(url, expected_hash, get_update_cache_dir())
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
