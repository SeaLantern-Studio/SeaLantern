//! 归档发布：把完成的临时产物移动到最终位置，且绝不覆盖既有条目。
//!
//! 归档创建与解压都遵循「目标必须不存在」的约定，因此发布这一步必须是
//! create-new 语义。[`std::fs::rename`] 不满足这一点——本机实测（Windows）
//! 文件覆盖文件、目录覆盖空目录都会静默成功，仅目录覆盖非空目录才会因
//! `DirectoryNotEmpty` 失败。仅靠事前检查无法排除检查与发布之间的竞争窗口：
//! 并发的另一次备份，或用户在此期间放入同名文件，都会被无声抹掉。
//!
//! 关键观察是 [`std::fs::hard_link`] 本身即具备 create-new 语义：目标已存在时
//! 返回 `AlreadyExists`（Unix 上是 `EEXIST`），全平台一致，无需任何平台 API。
//! 因此发布逻辑是：
//!
//! - 文件用 `hard_link` + `remove_file`。两个路径此前指向同一份内容，删除临时
//!   路径不会丢失数据
//! - 目录无法建立硬链接（Unix 与 Windows 均不允许），只能显式检查后 `rename`
//! - Linux 额外优先尝试 `renameat2(RENAME_NOREPLACE)`：单次系统调用，对文件与
//!   目录一律适用，连目录路径的检查窗口都不存在。经 `rustix` 的安全封装调用，
//!   本模块不含 `unsafe`
//!
//! 目录路径的残余风险仅限于「检查通过之后、`rename` 之前目标恰好变成空目录」
//! 这一窄窗口，且覆盖空目录不会丢失数据。
//!
//! 硬链接在不支持它的文件系统（FAT32/exFAT）上会失败。这是有意的取舍：宁可
//! 发布失败并向调用方报错，也不静默覆盖用户的既有备份。

use std::io;
use std::path::Path;

use super::ArchiveError;

/// 把 `temporary` 移动到 `destination`，目标已存在时不做覆盖。
///
/// 目标存在返回 [`ArchiveError::DestinationExists`]，其余失败返回
/// [`ArchiveError::Io`]。调用方负责在失败后清理 `temporary`。
pub(super) fn publish_new(temporary: &Path, destination: &Path) -> Result<(), ArchiveError> {
    publish_inner(temporary, destination).map_err(|error| {
        if error.kind() == io::ErrorKind::AlreadyExists {
            ArchiveError::DestinationExists { path: destination.to_path_buf() }
        } else {
            ArchiveError::io("publish completed archive", destination, error)
        }
    })
}

fn publish_inner(temporary: &Path, destination: &Path) -> io::Result<()> {
    if let Some(result) = try_rename_no_replace(temporary, destination) {
        return result;
    }
    if temporary.symlink_metadata()?.is_dir() {
        return publish_directory(temporary, destination);
    }
    publish_file(temporary, destination)
}

/// 在支持的平台上尝试原子的 create-new 重命名。
///
/// 返回 `None` 表示该平台没有此能力、或内核与文件系统不支持，调用方应走
/// 通用路径。
#[cfg(target_os = "linux")]
fn try_rename_no_replace(temporary: &Path, destination: &Path) -> Option<io::Result<()>> {
    use rustix::fs::{CWD, RenameFlags, renameat_with};

    // rustix 提供安全封装，此处不涉及 unsafe。
    match renameat_with(CWD, temporary, CWD, destination, RenameFlags::NOREPLACE) {
        Ok(()) => Some(Ok(())),
        Err(rustix::io::Errno::EXIST | rustix::io::Errno::NOTEMPTY) => {
            Some(Err(io::Error::from(io::ErrorKind::AlreadyExists)))
        }
        // RENAME_NOREPLACE 自 Linux 3.15 引入，且需要文件系统支持；缺失时
        // 内核返回 EINVAL 或 ENOSYS，改走通用路径。
        Err(rustix::io::Errno::INVAL | rustix::io::Errno::NOSYS) => None,
        Err(errno) => Some(Err(io::Error::from(errno))),
    }
}

#[cfg(not(target_os = "linux"))]
fn try_rename_no_replace(_temporary: &Path, _destination: &Path) -> Option<io::Result<()>> {
    None
}

/// 以 create-new 语义发布文件。
///
/// [`std::fs::hard_link`] 在目标已存在时返回 `AlreadyExists`，这正是所需的
/// 语义且全平台一致。链接建立后临时路径与目标指向同一份内容，删除前者不会
/// 丢失数据。
///
/// `remove_file` 失败（例如杀毒软件短暂占用临时文件）时**不视为发布失败**：
/// 发布的实质目标——目标文件就位且未覆盖既有内容——此时已达成，临时文件残留
/// 只是清理问题，交由 observability 记录后照常返回成功。若此处返回 Err，
/// 调用方会向用户报告备份失败，但目标文件其实已经就位；用户看到失败重试时，
/// 又因目标已存在而再次失败，形成无法自愈的错误提示。
fn publish_file(temporary: &Path, destination: &Path) -> io::Result<()> {
    std::fs::hard_link(temporary, destination)?;
    if let Err(error) = std::fs::remove_file(temporary)
        && error.kind() != io::ErrorKind::NotFound
    {
        // 目标已就位，残留的临时文件交给上层清理路径；此处仅记录日志。
        crate::observability::archive_cleanup_failed(temporary, &error);
    }
    Ok(())
}

/// 以近似 create-new 的语义发布目录。
///
/// 目录不能建立硬链接，只能先确认目标不存在再 `rename`。`rename` 对非空目录
/// 目标会失败，因此残余窗口仅限于检查之后目标恰好变成空目录的情形。
fn publish_directory(temporary: &Path, destination: &Path) -> io::Result<()> {
    match std::fs::symlink_metadata(destination) {
        Ok(_) => return Err(io::Error::from(io::ErrorKind::AlreadyExists)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    std::fs::rename(temporary, destination)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn publishes_when_the_destination_is_absent() {
        let root = crate::fs::test_dir("publish-new");
        fs::create_dir_all(&root).unwrap();
        let temporary = root.join("archive.tmp");
        let destination = root.join("archive.zip");
        fs::write(&temporary, b"fresh archive").unwrap();

        publish_new(&temporary, &destination).unwrap();

        assert_eq!(fs::read(&destination).unwrap(), b"fresh archive");
        // 发布成功后临时路径不应残留。
        assert!(!temporary.exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_to_replace_an_existing_destination() {
        let root = crate::fs::test_dir("publish-existing");
        fs::create_dir_all(&root).unwrap();
        let temporary = root.join("archive.tmp");
        let destination = root.join("archive.zip");
        fs::write(&temporary, b"new archive").unwrap();
        // 模拟事前检查通过之后、发布之前目标才出现的竞争场景。
        fs::write(&destination, b"existing archive").unwrap();

        assert!(matches!(
            publish_new(&temporary, &destination),
            Err(ArchiveError::DestinationExists { .. })
        ));
        // 既有内容必须完好，临时文件留给调用方清理。
        assert_eq!(fs::read(&destination).unwrap(), b"existing archive");
        assert!(temporary.exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_to_replace_an_existing_directory() {
        let root = crate::fs::test_dir("publish-existing-dir");
        fs::create_dir_all(&root).unwrap();
        let temporary = root.join("extracted.tmp");
        let destination = root.join("extracted");
        fs::create_dir_all(temporary.join("config")).unwrap();
        fs::create_dir_all(destination.join("existing")).unwrap();

        assert!(matches!(
            publish_new(&temporary, &destination),
            Err(ArchiveError::DestinationExists { .. })
        ));
        assert!(destination.join("existing").is_dir());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_to_replace_an_existing_empty_directory() {
        let root = crate::fs::test_dir("publish-existing-empty-dir");
        fs::create_dir_all(&root).unwrap();
        let temporary = root.join("extracted.tmp");
        let destination = root.join("extracted");
        fs::create_dir_all(temporary.join("config")).unwrap();
        fs::create_dir_all(&destination).unwrap();

        // 空目录是 std::fs::rename 会静默覆盖的场景之一（非空目录反而会因
        // DirectoryNotEmpty 失败），因此单独锁定。
        assert!(matches!(
            publish_new(&temporary, &destination),
            Err(ArchiveError::DestinationExists { .. })
        ));
        assert!(destination.is_dir());
        assert!(!destination.join("config").exists());
        // 临时目录保持原样，留给调用方清理。
        assert!(temporary.join("config").is_dir());

        fs::remove_dir_all(root).unwrap();
    }
}
