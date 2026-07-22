use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rusqlite::{Connection, Row, Transaction, TransactionBehavior};

use crate::observability;

use super::{process_lock_registry, PersistenceError};

/// 可安全传入 SQLite 参数绑定的动态值。
pub use rusqlite::types::Value as SqlValue;

/// SQLite 连接的底层运行参数。
#[derive(Debug, Clone)]
pub struct SqliteOptions {
    /// 发生跨进程锁竞争时等待的最长时间。
    pub busy_timeout: Duration,
    /// 是否为数据库启用外键约束。
    pub foreign_keys: bool,
    /// 是否使用 WAL 日志模式以允许并发读。
    pub wal: bool,
}

impl Default for SqliteOptions {
    fn default() -> Self {
        Self {
            busy_timeout: Duration::from_secs(5),
            foreign_keys: true,
            wal: true,
        }
    }
}

/// 调用方提供的版本化数据库迁移。
///
/// `sql` 必须是受信任的静态 SQL；运行时值应使用 `execute` 或
/// `query` 的参数绑定传递，不能拼接到此处。
#[derive(Debug, Clone, Copy)]
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

/// 进程内串行、跨进程可协调的 SQLite 数据库访问接口。
///
/// 不承载业务表定义或业务数据类型。上层仅通过参数绑定、行映射闭包、
/// 事务闭包和迁移清单传入自己的数据模型。
#[derive(Clone)]
pub struct SqliteDatabase {
    path: PathBuf,
    connection: Arc<Mutex<Connection>>,
}

impl SqliteDatabase {
    /// 使用默认选项打开或创建数据库。
    pub async fn open(path: impl Into<PathBuf>) -> Result<Self, PersistenceError> {
        Self::open_with_options(path, SqliteOptions::default()).await
    }

    /// 使用显式选项打开或创建数据库。
    pub async fn open_with_options(
        path: impl Into<PathBuf>,
        options: SqliteOptions,
    ) -> Result<Self, PersistenceError> {
        let path = path.into();
        let _guard = process_lock_registry().lock(&path).await?;
        let operation_path = path.clone();
        let opened = tokio::task::spawn_blocking(move || open_connection(&path, &options))
            .await
            .map_err(|error| PersistenceError::Task {
                operation: "open SQLite database",
                message: error.to_string(),
            })?;
        match opened {
            Ok(connection) => Ok(Self {
                path: operation_path,
                connection: Arc::new(Mutex::new(connection)),
            }),
            Err(error) => {
                observability::persistence_operation_failed("open", &operation_path, &error);
                Err(error)
            }
        }
    }

    /// 返回数据库文件路径。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 执行单条使用参数绑定的写入或控制语句。
    pub async fn execute(
        &self,
        sql: impl Into<String>,
        params: Vec<SqlValue>,
    ) -> Result<usize, PersistenceError> {
        let sql = sql.into();
        self.with_connection("execute", move |connection| {
            connection.execute(&sql, rusqlite::params_from_iter(params))
        })
        .await
    }

    /// 执行仅由受信任代码构造的多条 SQL 语句。
    pub async fn execute_batch(&self, sql: impl Into<String>) -> Result<(), PersistenceError> {
        let sql = sql.into();
        self.with_connection("execute batch", move |connection| connection.execute_batch(&sql))
            .await
    }

    /// 查询记录，并通过调用方提供的闭包映射每一行。
    pub async fn query<T, F>(
        &self,
        sql: impl Into<String>,
        params: Vec<SqlValue>,
        mut map_row: F,
    ) -> Result<Vec<T>, PersistenceError>
    where
        T: Send + 'static,
        F: FnMut(&Row<'_>) -> rusqlite::Result<T> + Send + 'static,
    {
        let sql = sql.into();
        self.with_connection("query", move |connection| {
            let mut statement = connection.prepare(&sql)?;
            let rows =
                statement.query_map(rusqlite::params_from_iter(params), |row| map_row(row))?;
            rows.collect()
        })
        .await
    }

    /// 在 `BEGIN IMMEDIATE` 事务中运行上层定义的写入操作。
    pub async fn write<T, F>(&self, operation: &'static str, work: F) -> Result<T, PersistenceError>
    where
        T: Send + 'static,
        F: FnOnce(&Transaction<'_>) -> rusqlite::Result<T> + Send + 'static,
    {
        self.with_mut_connection(operation, move |connection| {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let result = work(&transaction)?;
            transaction.commit()?;
            Ok(result)
        })
        .await
    }

    /// 以单个原子事务应用未执行过的迁移。
    pub async fn migrate(&self, mut migrations: Vec<Migration>) -> Result<(), PersistenceError> {
        migrations.sort_by_key(|migration| migration.version);
        if let Some(migration) = migrations.iter().find(|migration| migration.version < 0) {
            return Err(PersistenceError::InvalidMigration {
                version: migration.version,
                reason: "migration versions must not be negative",
            });
        }
        for pair in migrations.windows(2) {
            if pair[0].version == pair[1].version {
                return Err(PersistenceError::InvalidMigration {
                    version: pair[0].version,
                    reason: "migration versions must be unique",
                });
            }
        }

        self.with_mut_connection("migrate", move |connection| {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            transaction.execute_batch(
                "CREATE TABLE IF NOT EXISTS _sealantern_schema_migrations (\
                    version INTEGER PRIMARY KEY NOT NULL,\
                    name TEXT NOT NULL,\
                    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP\
                 )",
            )?;
            for migration in migrations {
                let already_applied: bool = transaction.query_row(
                    "SELECT EXISTS(SELECT 1 FROM _sealantern_schema_migrations WHERE version = ?1)",
                    [migration.version],
                    |row| row.get(0),
                )?;
                if !already_applied {
                    transaction.execute_batch(migration.sql)?;
                    transaction.execute(
                        "INSERT INTO _sealantern_schema_migrations (version, name) VALUES (?1, ?2)",
                        (migration.version, migration.name),
                    )?;
                }
            }
            transaction.commit()
        })
        .await
    }

    async fn with_connection<T, F>(
        &self,
        operation: &'static str,
        work: F,
    ) -> Result<T, PersistenceError>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> rusqlite::Result<T> + Send + 'static,
    {
        let path = self.path.clone();
        let process_guard = process_lock_registry().lock(&path).await?;
        let connection = Arc::clone(&self.connection);
        let result = tokio::task::spawn_blocking(move || {
            let _process_guard = process_guard;
            let connection = connection
                .lock()
                .map_err(|error| PersistenceError::Coordination {
                    resource: path.clone(),
                    message: error.to_string(),
                })?;
            work(&connection).map_err(|error| PersistenceError::Sqlite {
                operation,
                path,
                message: error.to_string(),
            })
        })
        .await
        .map_err(|error| PersistenceError::Task { operation, message: error.to_string() })?;
        report_operation_error(operation, &self.path, &result);
        result
    }

    async fn with_mut_connection<T, F>(
        &self,
        operation: &'static str,
        work: F,
    ) -> Result<T, PersistenceError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> rusqlite::Result<T> + Send + 'static,
    {
        let path = self.path.clone();
        let process_guard = process_lock_registry().lock(&path).await?;
        let connection = Arc::clone(&self.connection);
        let result = tokio::task::spawn_blocking(move || {
            let _process_guard = process_guard;
            let mut connection =
                connection
                    .lock()
                    .map_err(|error| PersistenceError::Coordination {
                        resource: path.clone(),
                        message: error.to_string(),
                    })?;
            work(&mut connection).map_err(|error| PersistenceError::Sqlite {
                operation,
                path,
                message: error.to_string(),
            })
        })
        .await
        .map_err(|error| PersistenceError::Task { operation, message: error.to_string() })?;
        report_operation_error(operation, &self.path, &result);
        result
    }
}

fn open_connection(path: &Path, options: &SqliteOptions) -> Result<Connection, PersistenceError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| PersistenceError::CreateParent {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let connection = Connection::open(path).map_err(|error| PersistenceError::Sqlite {
        operation: "open",
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    connection
        .busy_timeout(options.busy_timeout)
        .map_err(|error| PersistenceError::Sqlite {
            operation: "configure busy timeout",
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
    connection
        .pragma_update(None, "foreign_keys", options.foreign_keys)
        .map_err(|error| PersistenceError::Sqlite {
            operation: "enable foreign keys",
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
    if options.wal {
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|error| PersistenceError::Sqlite {
                operation: "enable WAL",
                path: path.to_path_buf(),
                message: error.to_string(),
            })?;
        connection
            .pragma_update(None, "synchronous", "NORMAL")
            .map_err(|error| PersistenceError::Sqlite {
                operation: "configure synchronous mode",
                path: path.to_path_buf(),
                message: error.to_string(),
            })?;
    }
    Ok(connection)
}

fn report_operation_error<T>(
    operation: &'static str,
    path: &Path,
    result: &Result<T, PersistenceError>,
) {
    if let Err(error) = result {
        observability::persistence_operation_failed(operation, path, error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database_path(label: &str) -> PathBuf {
        crate::fs::test_dir(label).join("state.sqlite")
    }

    #[tokio::test]
    async fn executes_queries_and_binds_values() {
        let path = database_path("sqlite-query");
        let database = SqliteDatabase::open(&path).await.unwrap();
        database
            .execute_batch("CREATE TABLE records (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .await
            .unwrap();
        database
            .execute(
                "INSERT INTO records (id, name) VALUES (?1, ?2)",
                vec![SqlValue::Integer(7), SqlValue::Text("O'Reilly".to_owned())],
            )
            .await
            .unwrap();

        let names = database
            .query("SELECT name FROM records WHERE id = ?1", vec![SqlValue::Integer(7)], |row| {
                row.get::<_, String>(0)
            })
            .await
            .unwrap();
        assert_eq!(names, ["O'Reilly"]);
        drop(database);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn applies_migrations_only_once() {
        let path = database_path("sqlite-migration");
        let database = SqliteDatabase::open(&path).await.unwrap();
        let migrations = vec![Migration {
            version: 1,
            name: "create records",
            sql: "CREATE TABLE records (id INTEGER PRIMARY KEY)",
        }];
        database.migrate(migrations.clone()).await.unwrap();
        database.migrate(migrations).await.unwrap();

        let versions = database
            .query("SELECT version FROM _sealantern_schema_migrations", vec![], |row| {
                row.get::<_, i64>(0)
            })
            .await
            .unwrap();
        assert_eq!(versions, [1]);
        drop(database);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn write_rolls_back_when_the_work_fails() {
        let path = database_path("sqlite-transaction");
        let database = SqliteDatabase::open(&path).await.unwrap();
        database
            .execute_batch("CREATE TABLE records (id INTEGER PRIMARY KEY)")
            .await
            .unwrap();
        let result = database
            .write("insert duplicate records", |transaction| {
                transaction.execute("INSERT INTO records (id) VALUES (1)", [])?;
                transaction.execute("INSERT INTO records (id) VALUES (1)", [])?;
                Ok(())
            })
            .await;
        assert!(matches!(result, Err(PersistenceError::Sqlite { .. })));
        let count = database
            .query("SELECT COUNT(*) FROM records", vec![], |row| row.get::<_, i64>(0))
            .await
            .unwrap();
        assert_eq!(count, [0]);
        drop(database);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn serializes_separately_opened_database_handles() {
        let path = database_path("sqlite-coordination");
        let first = SqliteDatabase::open(&path).await.unwrap();
        let second = SqliteDatabase::open(&path).await.unwrap();
        first
            .execute_batch("CREATE TABLE records (id INTEGER PRIMARY KEY)")
            .await
            .unwrap();

        let (entered_tx, entered_rx) = tokio::sync::oneshot::channel();
        let first_write = tokio::spawn(async move {
            first
                .write("hold transaction", move |transaction| {
                    let _ = entered_tx.send(());
                    std::thread::sleep(Duration::from_millis(50));
                    transaction.execute("INSERT INTO records (id) VALUES (1)", [])?;
                    Ok(())
                })
                .await
        });
        entered_rx.await.unwrap();

        let mut second_write = tokio::spawn(async move {
            second
                .execute("INSERT INTO records (id) VALUES (?1)", vec![SqlValue::Integer(2)])
                .await
        });
        assert!(tokio::time::timeout(Duration::from_millis(10), &mut second_write)
            .await
            .is_err());
        first_write.await.unwrap().unwrap();
        second_write.await.unwrap().unwrap();

        let reopened = SqliteDatabase::open(&path).await.unwrap();
        let ids = reopened
            .query("SELECT id FROM records ORDER BY id", vec![], |row| row.get::<_, i64>(0))
            .await
            .unwrap();
        assert_eq!(ids, [1, 2]);
        drop(reopened);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
