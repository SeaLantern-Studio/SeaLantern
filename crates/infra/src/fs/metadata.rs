use std::path::Path;
use std::time::SystemTime;

use crate::observability;

use super::FsError;

/// 不追踪符号链接收集的基本元数据。
#[derive(Clone, Debug)]
pub struct FileMetadata {
    /// 文件的字节大小（对于目录则为平台报告的元数据大小）。
    pub size: u64,
    /// 文件系统支持时的最后修改时间。
    pub modified: Option<SystemTime>,
    /// 目标是否为常规文件。
    pub is_file: bool,
    /// 目标是否为目录。
    pub is_dir: bool,
    /// 目标本身是否为符号链接。
    pub is_symlink: bool,
}

/// 描述路径（不追踪符号链接）。
pub async fn describe(path: impl AsRef<Path>) -> Result<FileMetadata, FsError> {
    let path = path.as_ref();
    let result = async {
        let metadata = tokio::fs::symlink_metadata(path)
            .await
            .map_err(|error| FsError::io("read file metadata", path, error))?;
        let file_type = metadata.file_type();
        Ok(FileMetadata {
            size: metadata.len(),
            modified: metadata.modified().ok(),
            is_file: file_type.is_file(),
            is_dir: file_type.is_dir(),
            is_symlink: file_type.is_symlink(),
        })
    }
    .await;
    if let Err(error) = &result {
        observability::operation_failed("describe path", path, error);
    }
    result
}

/// 返回常规文件的大小。
pub async fn file_size(path: impl AsRef<Path>) -> Result<u64, FsError> {
    let path = path.as_ref();
    let metadata = describe(path).await?;
    if !metadata.is_file {
        return Err(FsError::InvalidPath {
            path: path.to_path_buf(),
            reason: "path is not a regular file",
        });
    }
    Ok(metadata.size)
}
