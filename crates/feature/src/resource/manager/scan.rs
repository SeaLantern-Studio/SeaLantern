//! 实例资源目录扫描（IO，只读）。
//!
//! 扫描是资源管理的"事实来源"：只读取目录与文件元数据，
//! 不创建、不修改、不删除任何文件。
//!
//! 目录遍历与元数据读取统一走 `tokio::fs`，避免阻塞异步运行时的工作线程。

use std::path::Path;

use crate::observability;

use super::ResourceManagerError;
use super::layout::ResourceTarget;
use super::models::{InstanceExtension, InstanceExtensionKind};
use super::naming;

/// 扫描单个资源目录，返回其中的可管理资源文件。
///
/// 目录不存在时返回空列表（视为该实例尚未创建该资源目录）。
/// 单个文件的元数据读取失败不阻断整体扫描，但会记录告警。
pub async fn scan_target(
    target_dir: &Path,
    kind: InstanceExtensionKind,
) -> Result<Vec<InstanceExtension>, ResourceManagerError> {
    let mut entries = match tokio::fs::read_dir(target_dir).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };

    let mut extensions = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_file() {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();
        if !naming::is_resource_file(&file_name) {
            continue;
        }

        let size_bytes = match entry.metadata().await {
            Ok(metadata) => metadata.len(),
            Err(error) => {
                // 元数据失败不阻断扫描，但必须留痕，避免静默记成 0 字节。
                observability::resource_metadata_unreadable(&entry.path(), &error);
                0
            }
        };
        let enabled = !naming::is_disabled(&file_name);

        extensions.push(InstanceExtension::new(
            kind,
            file_name,
            entry.path(),
            enabled,
            size_bytes,
        )?);
    }

    extensions.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    Ok(extensions)
}

/// 扫描实例中所有目标资源目录，结果按文件名排序。
pub async fn scan_instance(
    instance_dir: &Path,
    targets: &[ResourceTarget],
) -> Result<Vec<InstanceExtension>, ResourceManagerError> {
    let mut extensions = Vec::new();
    for target in targets {
        extensions.extend(scan_target(&instance_dir.join(&target.relative), target.kind).await?);
    }
    extensions.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    Ok(extensions)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir()
            .join(format!("sealantern-resource-scan-{label}-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&path).expect("temp dir should be created");
        path
    }

    #[tokio::test]
    async fn scan_reads_only_resource_files() {
        let root = temp_dir("read");
        fs::create_dir_all(root.join("mods")).unwrap();
        fs::write(root.join("mods/sodium.jar"), b"jar").unwrap();
        fs::write(root.join("mods/config.txt"), b"ignored").unwrap();
        fs::write(root.join("mods/disabled.jar.disabled"), b"jar").unwrap();
        fs::create_dir_all(root.join("mods/subdir")).unwrap();

        let extensions = scan_target(&root.join("mods"), InstanceExtensionKind::Mod)
            .await
            .unwrap();
        assert_eq!(extensions.len(), 2);
        assert_eq!(extensions[0].file_name, "disabled.jar.disabled");
        assert!(!extensions[0].enabled);
        assert_eq!(extensions[1].file_name, "sodium.jar");
        assert!(extensions[1].enabled);

        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn missing_directory_yields_empty_list() {
        let root = temp_dir("missing");
        let extensions = scan_target(&root.join("mods"), InstanceExtensionKind::Mod)
            .await
            .unwrap();
        assert!(extensions.is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn scan_instance_walks_all_targets() {
        let root = temp_dir("instance");
        fs::create_dir_all(root.join("mods")).unwrap();
        fs::create_dir_all(root.join("plugins")).unwrap();
        fs::write(root.join("mods/sodium.jar"), b"jar").unwrap();
        fs::write(root.join("plugins/essentials.jar"), b"jar").unwrap();

        let targets = vec![
            ResourceTarget::new(InstanceExtensionKind::Plugin, "plugins"),
            ResourceTarget::new(InstanceExtensionKind::Mod, "mods"),
        ];
        let extensions = scan_instance(&root, &targets).await.unwrap();
        assert_eq!(extensions.len(), 2);

        fs::remove_dir_all(root).unwrap();
    }
}
