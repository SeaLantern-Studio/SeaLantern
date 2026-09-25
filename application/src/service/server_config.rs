//! 服务器配置（server.properties）服务实现。
//!
//! 实现 [`crate::port::ServerConfigService`] 能力端口，组合
//! `feature::config::server` 的配置文件读写能力，向宿主提供
//! server.properties 的可视化结构读写、原始文本读写、解析与写入预览。
//!
//! 错误分层：内部以应用层主错误 [`ServerConfigError`] 为源头，暴露
//! [`ServerConfigService`] 时统一转为接口契约错误
//! [`ServerConfigServiceError`]。

use std::collections::BTreeMap;
use std::path::Path;

use async_trait::async_trait;
use sealantern_contract::ServerConfigServiceError;
use sealantern_contract::server_config::ServerProperties;
use sealantern_feature::config::server::{ServerPropertiesError, ServerPropertiesManager};

use crate::error::ServerConfigError;
use crate::port::ServerConfigService;

/// 将阻塞的文件操作调度到阻塞线程池，统一收敛错误。
async fn run_blocking<T, F>(operation: F) -> Result<T, ServerConfigError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, ServerPropertiesError> + Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(ServerConfigError::from)?
        .map_err(ServerConfigError::from)
}

/// 基于 `feature` 配置读写能力的 server.properties 服务实现。
#[derive(Debug, Default)]
pub struct CoreServerConfigService;

#[async_trait]
impl ServerConfigService for CoreServerConfigService {
    async fn read(&self, server_path: &Path) -> Result<ServerProperties, ServerConfigServiceError> {
        // 用 `to_path_buf` 而非字符串转换：非 UTF-8 的目录名必须原样传给文件系统。
        let path = server_path.to_path_buf();
        run_blocking(move || ServerPropertiesManager::new(path).read())
            .await
            .map_err(Into::into)
    }

    async fn write(
        &self,
        server_path: &Path,
        values: &BTreeMap<String, String>,
    ) -> Result<(), ServerConfigServiceError> {
        let path = server_path.to_path_buf();
        let values = values.clone();
        run_blocking(move || ServerPropertiesManager::new(path).write(&values))
            .await
            .map_err(Into::into)
    }

    async fn read_source(&self, server_path: &Path) -> Result<String, ServerConfigServiceError> {
        let path = server_path.to_path_buf();
        run_blocking(move || ServerPropertiesManager::new(path).read_source())
            .await
            .map_err(Into::into)
    }

    async fn write_source(
        &self,
        server_path: &Path,
        source: &str,
    ) -> Result<(), ServerConfigServiceError> {
        let path = server_path.to_path_buf();
        let source = source.to_owned();
        run_blocking(move || ServerPropertiesManager::new(path).write_source(&source))
            .await
            .map_err(Into::into)
    }

    async fn parse_source(
        &self,
        source: &str,
    ) -> Result<ServerProperties, ServerConfigServiceError> {
        let source = source.to_owned();
        run_blocking(move || ServerPropertiesManager::parse_source(&source))
            .await
            .map_err(Into::into)
    }

    async fn preview_write(
        &self,
        server_path: &Path,
        values: &BTreeMap<String, String>,
    ) -> Result<String, ServerConfigServiceError> {
        let path = server_path.to_path_buf();
        let values = values.clone();
        run_blocking(move || ServerPropertiesManager::new(path).preview_write(&values))
            .await
            .map_err(Into::into)
    }

    async fn preview_write_from_source(
        &self,
        source: &str,
        values: &BTreeMap<String, String>,
    ) -> Result<String, ServerConfigServiceError> {
        let source = source.to_owned();
        let values = values.clone();
        run_blocking(move || ServerPropertiesManager::preview_write_from_source(&source, &values))
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn read_returns_empty_properties_for_missing_file() {
        let dir = tempdir().expect("temporary directory should be created");
        let service = CoreServerConfigService;
        let properties = service.read(dir.path()).await.expect("read should succeed");
        assert!(properties.entries.is_empty());
        assert!(properties.raw.is_empty());
    }

    #[tokio::test]
    async fn write_then_read_round_trips_values() {
        let dir = tempdir().expect("temporary directory should be created");
        let service = CoreServerConfigService;
        let mut values = BTreeMap::new();
        values.insert("server-port".to_string(), "25566".to_string());
        values.insert("motd".to_string(), "hello".to_string());

        service
            .write(dir.path(), &values)
            .await
            .expect("write should succeed");

        let properties = service.read(dir.path()).await.expect("read should succeed");
        assert_eq!(properties.raw.get("server-port").map(String::as_str), Some("25566"));
        assert_eq!(properties.raw.get("motd").map(String::as_str), Some("hello"));
    }

    #[tokio::test]
    async fn parse_source_parses_valid_source() {
        // 解析是宽松的：空行/注释被跳过，`key=value` 收集为键值对；
        // 任何文本都能解析出结果，不会产生 InvalidInput。
        let service = CoreServerConfigService;
        let properties = service
            .parse_source("server-port=25565\n# comment\n")
            .await
            .expect("parse should succeed");
        assert_eq!(properties.raw.len(), 1);
        assert_eq!(properties.raw.get("server-port").map(String::as_str), Some("25565"));
    }

    /// 非 UTF-8 的实例目录必须原样传给文件系统。
    ///
    /// 该场景只在类 Unix 系统上可能出现：Windows 的路径以 UTF-16 为底层表示，
    /// 无法表达非 UTF-8 字节。若边界处把目录转成字符串，非法字节会被替换成
    /// U+FFFD，拼接出的 `server.properties` 将落到另一个路径上。
    #[tokio::test]
    async fn read_uses_a_non_utf8_directory_verbatim() {
        let root = tempdir().expect("temporary directory should be created");
        let server_dir = root.path().join(non_utf8_directory_name());

        // 先确认夹具确实能暴露有损转换：转换后的路径与真实路径不同。
        let lossy = PathBuf::from(server_dir.to_string_lossy().into_owned());
        assert_ne!(
            lossy, server_dir,
            "lossy conversion should have changed the path, otherwise this case cannot prove the defect"
        );

        std::fs::create_dir_all(&server_dir).expect("non-UTF-8 directory should be created");
        std::fs::write(server_dir.join("server.properties"), "server-port=25565\n")
            .expect("fixture should be written");

        // 传入 `&Path` 时读取应命中真实文件，而不是被替换后的路径。
        let properties = CoreServerConfigService
            .read(&server_dir)
            .await
            .expect("read should succeed");
        assert_eq!(properties.raw.get("server-port").map(String::as_str), Some("25565"));

        // 反向确认缺陷的后果：沿有损转换后的路径读取拿不到任何配置，
        // 即修复前每次操作都会落到不同的、不存在的路径上。
        let lossy_properties = CoreServerConfigService
            .read(&lossy)
            .await
            .expect("reading a missing path should return empty properties");
        assert!(
            lossy_properties.raw.is_empty(),
            "the lossy path must not hit the real config file"
        );
    }

    /// 构造一个无法表示为 UTF-8 的目录名。
    ///
    /// 类 Unix 使用非法字节序列，Windows 使用孤立的 UTF-16 代理项：两者都不是
    /// 合法 UTF-8，`to_string_lossy` 都会把它替换成 U+FFFD。
    fn non_utf8_directory_name() -> std::ffi::OsString {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            std::ffi::OsString::from_vec(vec![b's', 0xff, b'v'])
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStringExt;
            // 0xDC00 是孤立的低位代理项，无法编码为任何 UTF-8 序列。
            std::ffi::OsString::from_wide(&[0x0073, 0xDC00, 0x0076])
        }
    }
}
