//! 服务器控制台日志服务实现。
//!
//! 实现 [`crate::port::ConsoleService`] 能力端口，组合
//! [`CoreInstanceService`]（按实例定位日志目录）与日志存储能力
//! （`feature::server::log`），向宿主提供服务器控制台日志的增量读取。
//!
//! 错误分层：内部以应用层主错误 [`ConsoleError`] 为源头，暴露
//! [`ConsoleService`] 时统一转为接口契约错误 [`ConsoleServiceError`]。

use std::sync::Arc;

use async_trait::async_trait;
use sealantern_contract::ConsoleServiceError;
use sealantern_contract::console::ConsoleLogLine;
use sealantern_core::instance::InstanceId;
use sealantern_feature::server::log::{open_log_database, read_logs};

use super::CoreInstanceService;
use crate::error::ConsoleError;
use crate::port::{ConsoleService, InstanceService};

/// 基于实例目录日志存储的控制台日志服务实现。
pub struct CoreConsoleService {
    instance_service: Arc<CoreInstanceService>,
}

impl CoreConsoleService {
    /// 创建使用指定实例服务的控制台日志服务。
    pub fn new(instance_service: Arc<CoreInstanceService>) -> Self {
        Self { instance_service }
    }
}

#[async_trait]
impl ConsoleService for CoreConsoleService {
    async fn logs(
        &self,
        id: &InstanceId,
        since: i64,
        recent_limit: Option<i64>,
    ) -> Result<Vec<ConsoleLogLine>, ConsoleServiceError> {
        // 游标与窗口大小均须非负（窗口至少为 1 行），避免非正窗口被
        // 底层静默解释为"无限制"。
        if since < 0 || recent_limit.is_some_and(|limit| limit <= 0) {
            return Err(ConsoleError::InvalidInput.into());
        }

        // 按实例定位日志目录；日志库不存在时自动创建（幂等）。
        let instance = self
            .instance_service
            .find(id)
            .await
            .map_err(ConsoleError::from)?
            .ok_or(ConsoleError::InstanceNotFound)?;
        let database = open_log_database(&instance.directory)
            .await
            .map_err(ConsoleError::from)?;
        let lines = read_logs(&database, since, recent_limit)
            .await
            .map_err(ConsoleError::from)?;

        Ok(lines
            .into_iter()
            .map(|line| ConsoleLogLine {
                sequence: line.id,
                timestamp: line.timestamp,
                source: line.source,
                line: line.line,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use sealantern_core::instance::{InstanceSpec, LocalLaunch, StartupMode};
    use sealantern_feature::server::log::{LogSource, open_log_database};
    use sealantern_infra::persistence::SqlValue;

    use super::*;

    fn sample_spec(id: &str, directory: PathBuf) -> InstanceSpec {
        InstanceSpec {
            id: InstanceId::new(id).expect("valid id"),
            name: format!("服务器-{id}"),
            aliases: Vec::new(),
            core_type: "paper".into(),
            core_version: "1.20.4".into(),
            game_version: "1.20.4".into(),
            directory: directory.clone(),
            port: 25565,
            max_memory_mib: 2048,
            min_memory_mib: 512,
            created_at_unix_secs: 0,
            last_started_at_unix_secs: None,
            server_metadata: None,
            launch: LocalLaunch {
                startup_mode: StartupMode::Jar,
                startup_target: Some(directory.join("server.jar")),
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: None,
                jvm_arguments: Vec::new(),
            },
        }
    }

    #[tokio::test]
    async fn logs_reads_persisted_lines_from_instance_directory() {
        let temp = tempfile::tempdir().expect("临时目录应创建成功");
        let instance_dir = temp.path().join("server-a");
        std::fs::create_dir_all(&instance_dir).expect("实例目录应创建成功");

        let instance_service = Arc::new(
            CoreInstanceService::with_path(temp.path().join("instances.json"))
                .await
                .expect("实例服务应创建成功"),
        );
        instance_service
            .create(sample_spec("a", instance_dir.clone()))
            .await
            .expect("实例应创建成功");

        // 预置日志数据（模拟输出管线写入）。
        let database = open_log_database(&instance_dir)
            .await
            .expect("日志库应初始化");
        database
            .insert(
                "INSERT INTO log_lines (timestamp, source, line) VALUES (?1, ?2, ?3)",
                [
                    SqlValue::Integer(1000),
                    SqlValue::Text(LogSource::Server.as_str().to_owned()),
                    SqlValue::Text("line one".to_owned()),
                ],
            )
            .await
            .expect("日志行应写入成功");

        let console = CoreConsoleService::new(instance_service);
        let lines = console
            .logs(&InstanceId::new("a".to_owned()).expect("valid id"), 0, None)
            .await
            .expect("读取应成功");

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].sequence, 1);
        assert_eq!(lines[0].source, "server");
        assert_eq!(lines[0].line, "line one");
    }

    #[tokio::test]
    async fn logs_rejects_negative_since() {
        let instance_service = Arc::new(
            CoreInstanceService::with_path(
                std::env::temp_dir().join("sealantern-console-neg-since.json"),
            )
            .await
            .expect("实例服务应创建成功"),
        );
        let console = CoreConsoleService::new(instance_service);

        let result = console
            .logs(&InstanceId::new("a".to_owned()).expect("valid id"), -1, None)
            .await;
        assert!(matches!(result, Err(ConsoleServiceError::InvalidInput)));
    }

    #[tokio::test]
    async fn logs_rejects_non_positive_recent_limit() {
        let instance_service = Arc::new(
            CoreInstanceService::with_path(
                std::env::temp_dir().join("sealantern-console-neg-limit.json"),
            )
            .await
            .expect("实例服务应创建成功"),
        );
        let console = CoreConsoleService::new(instance_service);

        for invalid in [Some(0), Some(-3)] {
            let result = console
                .logs(&InstanceId::new("a".to_owned()).expect("valid id"), 0, invalid)
                .await;
            assert!(matches!(result, Err(ConsoleServiceError::InvalidInput)));
        }
    }

    #[tokio::test]
    async fn logs_reports_missing_instance() {
        let instance_service = Arc::new(
            CoreInstanceService::with_path(
                std::env::temp_dir().join("sealantern-console-missing.json"),
            )
            .await
            .expect("实例服务应创建成功"),
        );
        let console = CoreConsoleService::new(instance_service);

        let result = console
            .logs(&InstanceId::new("missing".to_owned()).expect("valid id"), 0, None)
            .await;
        assert!(matches!(result, Err(ConsoleServiceError::InstanceNotFound)));
    }
}
