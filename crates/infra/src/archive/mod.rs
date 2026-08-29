//! 可移植的归档基础设施。
//!
//! 归档从目录内容写入，并通过经过验证的相对路径进行解压。
//! 在解压过程中有意拒绝符号链接：创建符号链接在支持的
//! 平台上具有不兼容的权限和语义，调用者必须先选择专门的
//! 策略才能这样做。
//!
//! 模块划分：
//! - [`limits`] / [`entry`]：与归档格式无关的资源限制、统计与路径校验
//! - [`symbol_link`]：符号链接载荷解析
//! - [`zip_read`] / [`zip_write`]：ZIP 格式的解压与创建
//! - [`tar_read`]：tar.gz 格式的解压
//!
//! 本文件内的目录句柄辅助函数同样与格式无关，供各格式适配器共用。

mod entry;
mod error;
mod limits;
mod symbol_link;
mod tar_read;
mod zip_read;
mod zip_write;

use std::path::Path;

use cap_std::ambient_authority;
use cap_std::fs::Dir;

pub use error::ArchiveError;
pub use limits::{ArchiveSummary, ExtractionLimits, ExtractionSummary};
pub use symbol_link::{is_symbolic_link, parse_symbolic_link_target};
pub use tar_read::{extract_tar_gz, extract_tar_gz_with_limits};
pub use zip_read::{extract_zip, extract_zip_with_limits};
pub use zip_write::create_zip;

use entry::{check_entry_path_length, ensure_directory, ensure_parent_dirs, safe_entry_path};

/// 打开一个已存在的普通目录，拒绝符号链接与非目录路径。
///
/// 通过父目录句柄 + `symlink_metadata` 校验，避免直接打开时跟随符号链接。
fn open_existing_directory(path: &Path, role: &'static str) -> Result<Dir, ArchiveError> {
    let parent_path = parent_path(path);
    let name = path
        .file_name()
        .ok_or_else(|| ArchiveError::InvalidSource {
            path: path.to_path_buf(),
            reason: "source directory must have a final path component",
        })?;
    let parent = Dir::open_ambient_dir(parent_path, ambient_authority())
        .map_err(|error| ArchiveError::io("open archive source parent", parent_path, error))?;
    let metadata = parent
        .symlink_metadata(name)
        .map_err(|error| ArchiveError::io("read archive source metadata", path, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ArchiveError::InvalidSource { path: path.to_path_buf(), reason: role });
    }
    parent
        .open_dir(name)
        .map_err(|error| ArchiveError::io("open archive source directory", path, error))
}

/// 创建一个全新的目录并返回其句柄，目录已存在时报错。
///
/// 解压目标必须是新建目录，避免与既有内容混合或覆盖此前的解压结果。
fn create_new_directory(path: &Path) -> Result<Dir, ArchiveError> {
    let parent_path = parent_path(path);
    std::fs::create_dir_all(parent_path).map_err(|error| {
        ArchiveError::io("create archive destination parent", parent_path, error)
    })?;
    let name = path
        .file_name()
        .ok_or_else(|| ArchiveError::InvalidDestination {
            path: path.to_path_buf(),
            reason: "destination directory must have a final path component",
        })?;
    let parent = Dir::open_ambient_dir(parent_path, ambient_authority())
        .map_err(|error| ArchiveError::io("open archive destination parent", parent_path, error))?;
    match parent.create_dir(name) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(ArchiveError::DestinationExists { path: path.to_path_buf() });
        }
        Err(error) => {
            return Err(ArchiveError::io("create archive extraction directory", path, error));
        }
    }
    parent
        .open_dir(name)
        .map_err(|error| ArchiveError::io("open archive extraction directory", path, error))
}

/// 返回路径的父目录，无父目录时回落到当前目录。
fn parent_path(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}
