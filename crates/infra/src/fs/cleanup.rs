use std::path::Path;

use crate::observability;

use super::FsError;

/// 当文件或目录存在时将其删除。
pub async fn remove_if_exists(path: impl AsRef<Path>) -> Result<bool, FsError> {
    let path = path.as_ref();
    let result = match tokio::fs::symlink_metadata(path)
        .await
        .map_err(|error| FsError::io("read path metadata before removal", path, error))
    {
        Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {
            tokio::fs::remove_dir_all(path)
                .await
                .map_err(|error| FsError::io("remove directory", path, error))?;
            Ok(true)
        }
        Ok(_) => {
            tokio::fs::remove_file(path)
                .await
                .map_err(|error| FsError::io("remove file", path, error))?;
            Ok(true)
        }
        Err(FsError::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => {
            Ok(false)
        }
        Err(error) => Err(error),
    };
    if let Err(error) = &result {
        observability::operation_failed("remove path", path, error);
    }
    result
}

/// 删除目录的所有直接子项，但不删除目录本身。
pub async fn clear_directory(path: impl AsRef<Path>) -> Result<(), FsError> {
    let path = path.as_ref();
    let result = async {
        let mut entries = tokio::fs::read_dir(path)
            .await
            .map_err(|error| FsError::io("read directory", path, error))?;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|error| FsError::io("iterate directory", path, error))?
        {
            remove_if_exists(entry.path()).await?;
        }
        Ok(())
    }
    .await;
    if let Err(error) = &result {
        observability::operation_failed("clear directory", path, error);
    }
    result
}
