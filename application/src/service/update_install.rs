use async_trait::async_trait;
use sealantern_extra::update::{
    check_pending_update, clear_pending_update, download_update_file_without_events,
    get_pending_update_file, get_update_cache_dir, write_pending_update, PendingUpdate,
};
use sealantern_interface::{UpdateInstallService, UpdateInstallServiceError};

#[derive(Debug, Default)]
pub struct CoreUpdateInstallService;

#[async_trait]
impl UpdateInstallService for CoreUpdateInstallService {
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
    async fn pending(&self) -> Result<Option<PendingUpdate>, UpdateInstallServiceError> {
        check_pending_update()
            .await
            .map_err(|_| UpdateInstallServiceError::OperationFailed)
    }
    async fn clear_pending(&self) -> Result<(), UpdateInstallServiceError> {
        clear_pending_update()
            .await
            .map_err(|_| UpdateInstallServiceError::OperationFailed)
    }
    async fn install(
        &self,
        file_path: String,
        arguments: Vec<String>,
    ) -> Result<(), UpdateInstallServiceError> {
        #[cfg(target_os = "windows")]
        {
            let refs = arguments.iter().map(String::as_str).collect::<Vec<_>>();
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
            let _ = (file_path, arguments);
            Err(UpdateInstallServiceError::Unsupported)
        }
    }
}
