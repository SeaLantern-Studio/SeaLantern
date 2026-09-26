//! 实例资源清单的持久化（IO）。
//!
//! 清单保存在实例根目录下的 `.sealantern/resources.json`，随实例目录
//! 一起复制、备份与迁移。写入复用 `infra::fs` 的原子写与跨进程文件锁。

use std::path::{Path, PathBuf};

use sealantern_infra::fs::{DataLimit, FileLock, read_json, write_json_atomic};

use super::ResourceManagerError;
use super::models::{MANIFEST_SCHEMA_VERSION, ResourceManifest};

/// 清单所在的实例内目录。
pub const MANIFEST_DIR: &str = ".sealantern";

/// 清单文件名。
pub const MANIFEST_FILE: &str = "resources.json";

/// 读取清单时的字节上限（1 MiB）。
const MANIFEST_READ_LIMIT: usize = 1024 * 1024;

/// 返回实例资源清单的完整路径。
pub fn path_for(instance_dir: &Path) -> PathBuf {
    instance_dir.join(MANIFEST_DIR).join(MANIFEST_FILE)
}

/// 读取实例资源清单；清单不存在时返回默认空清单。
///
/// 读操作不加锁：写入是原子的（同级临时文件 + 替换），读到的要么是旧内容、
/// 要么是新内容，不会是半截文件。
pub async fn load(instance_dir: &Path) -> Result<ResourceManifest, ResourceManagerError> {
    load_unlocked(&path_for(instance_dir)).await
}

/// 在持有跨进程锁的情况下读改写清单。
///
/// 锁覆盖"读取 → 修改 → 写回"的整个过程，避免两个并发写操作各自读取旧快照
/// 后互相覆盖。`mutate` 返回 `Err` 时放弃写入，磁盘清单保持不变；返回
/// `Ok(Some(value))` 时，该值会作为第二个返回值回传给调用方（用于把基于
/// 锁内最新数据的计算结果带出闭包）。
///
/// # 并发语义
///
/// [`FileLock::try_acquire`] 是**快速失败**（不排队）：并发时后到的写操作
/// 直接返回错误。当前单写者场景可接受；若将来出现批量安装等高频并发写，
/// 调用方需要自行增加一次重试或串行化。
pub async fn update<F, T>(
    instance_dir: &Path,
    mutate: F,
) -> Result<(ResourceManifest, Option<T>), ResourceManagerError>
where
    F: FnOnce(&mut ResourceManifest) -> Result<Option<T>, ResourceManagerError>,
{
    let path = path_for(instance_dir);
    let _lock = FileLock::try_acquire(&path).map_err(|error| manifest_error(&path, error))?;

    let mut manifest = load_unlocked(&path).await?;
    let output = mutate(&mut manifest)?;
    write_json_atomic(&path, &manifest)
        .await
        .map_err(|error| manifest_error(&path, error))?;

    Ok((manifest, output))
}

/// 不加锁读取清单；文件不存在时返回默认空清单。
///
/// 父目录由 `write_json_atomic` 内部创建，这里无需额外的 `create_dir_all`。
async fn load_unlocked(path: &Path) -> Result<ResourceManifest, ResourceManagerError> {
    if !path.exists() {
        return Ok(ResourceManifest::default());
    }

    let manifest: ResourceManifest = read_json(path, DataLimit::new(MANIFEST_READ_LIMIT))
        .await
        .map_err(|error| manifest_error(path, error))?;

    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(ResourceManagerError::Manifest {
            path: path.to_path_buf(),
            message: format!(
                "unsupported schema version {} (expected {MANIFEST_SCHEMA_VERSION})",
                manifest.schema_version
            ),
        });
    }

    Ok(manifest)
}

fn manifest_error(path: &Path, error: impl std::fmt::Display) -> ResourceManagerError {
    ResourceManagerError::Manifest {
        path: path.to_path_buf(),
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::resource::manager::models::{
        InstanceExtensionKind, ManagedResource, ResourceProvenance, ResourceSource,
    };

    fn temp_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir()
            .join(format!("sealantern-resource-manifest-{label}-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&path).expect("temp dir should be created");
        path
    }

    fn sample_manifest() -> ResourceManifest {
        ResourceManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            resources: vec![ManagedResource {
                file_name: "sodium.jar".into(),
                kind: InstanceExtensionKind::Mod,
                enabled: true,
                hash: Some("sha256:abc".into()),
                size_bytes: Some(42),
                provenance: Some(ResourceProvenance {
                    source: ResourceSource::Modrinth,
                    project_id: Some("AANobbMI".into()),
                    version_id: Some("v1".into()),
                    version_number: Some("0.5.8".into()),
                    installed_at_unix_secs: 1,
                }),
            }],
        }
    }

    #[tokio::test]
    async fn missing_manifest_loads_as_default() {
        let root = temp_dir("missing");
        let manifest = load(&root).await.unwrap();
        assert_eq!(manifest, ResourceManifest::default());
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn manifest_round_trips_through_disk() {
        let root = temp_dir("roundtrip");
        let manifest = sample_manifest();

        update(&root, |stored| {
            stored.resources = manifest.resources.clone();
            Ok(None::<()>)
        })
        .await
        .unwrap();
        assert!(path_for(&root).exists());

        let loaded = load(&root).await.unwrap();
        assert_eq!(loaded, manifest);
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn update_reads_modifies_and_writes_under_a_single_lock() {
        let root = temp_dir("update");

        update(&root, |manifest| {
            manifest.resources.push(ManagedResource {
                file_name: "first.jar".into(),
                kind: InstanceExtensionKind::Mod,
                enabled: true,
                hash: None,
                size_bytes: None,
                provenance: None,
            });
            Ok(None::<()>)
        })
        .await
        .unwrap();

        let (updated, _) = update(&root, |manifest| {
            manifest.resources.push(ManagedResource {
                file_name: "second.jar".into(),
                kind: InstanceExtensionKind::Mod,
                enabled: true,
                hash: None,
                size_bytes: None,
                provenance: None,
            });
            Ok(None::<()>)
        })
        .await
        .unwrap();

        assert_eq!(updated.resources.len(), 2);
        assert_eq!(load(&root).await.unwrap().resources.len(), 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn update_aborts_without_touching_disk_when_mutation_fails() {
        let root = temp_dir("update-abort");

        update(&root, |stored| {
            stored.resources = sample_manifest().resources.clone();
            Ok(None::<()>)
        })
        .await
        .unwrap();

        let result: Result<(ResourceManifest, Option<()>), ResourceManagerError> =
            update(&root, |manifest| {
                manifest.resources.clear();
                Err(ResourceManagerError::NotFound("abort".into()))
            })
            .await;
        assert!(result.is_err());

        assert_eq!(load(&root).await.unwrap().resources.len(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn unsupported_schema_version_is_rejected() {
        let root = temp_dir("schema");

        update(&root, |manifest| {
            manifest.schema_version = 99;
            Ok(None::<()>)
        })
        .await
        .unwrap();

        let error = load(&root).await.unwrap_err();
        assert!(matches!(error, ResourceManagerError::Manifest { .. }));
        fs::remove_dir_all(root).unwrap();
    }
}
