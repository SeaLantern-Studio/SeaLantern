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
    async fn read(&self, server_path: &str) -> Result<ServerProperties, ServerConfigServiceError> {
        let path = server_path.to_owned();
        run_blocking(move || ServerPropertiesManager::new(path).read())
            .await
            .map_err(Into::into)
    }

    async fn write(
        &self,
        server_path: &str,
        values: &BTreeMap<String, String>,
    ) -> Result<(), ServerConfigServiceError> {
        let path = server_path.to_owned();
        let values = values.clone();
        run_blocking(move || ServerPropertiesManager::new(path).write(&values))
            .await
            .map_err(Into::into)
    }

    async fn read_source(&self, server_path: &str) -> Result<String, ServerConfigServiceError> {
        let path = server_path.to_owned();
        run_blocking(move || ServerPropertiesManager::new(path).read_source())
            .await
            .map_err(Into::into)
    }

    async fn write_source(
        &self,
        server_path: &str,
        source: &str,
    ) -> Result<(), ServerConfigServiceError> {
        let path = server_path.to_owned();
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
        server_path: &str,
        values: &BTreeMap<String, String>,
    ) -> Result<String, ServerConfigServiceError> {
        let path = server_path.to_owned();
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

    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn read_returns_empty_properties_for_missing_file() {
        let dir = tempdir().expect("临时目录应创建成功");
        let service = CoreServerConfigService;
        let properties = service
            .read(&dir.path().to_string_lossy())
            .await
            .expect("读取应成功");
        assert!(properties.entries.is_empty());
        assert!(properties.raw.is_empty());
    }

    #[tokio::test]
    async fn write_then_read_round_trips_values() {
        let dir = tempdir().expect("临时目录应创建成功");
        let service = CoreServerConfigService;
        let mut values = BTreeMap::new();
        values.insert("server-port".to_string(), "25566".to_string());
        values.insert("motd".to_string(), "hello".to_string());

        service
            .write(&dir.path().to_string_lossy(), &values)
            .await
            .expect("写入应成功");

        let properties = service
            .read(&dir.path().to_string_lossy())
            .await
            .expect("读取应成功");
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
            .expect("解析应成功");
        assert_eq!(properties.raw.len(), 1);
        assert_eq!(properties.raw.get("server-port").map(String::as_str), Some("25565"));
    }
}
