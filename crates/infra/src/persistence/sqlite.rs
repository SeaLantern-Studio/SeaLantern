use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, Row, Transaction, TransactionBehavior};

use crate::observability;

use super::{PersistenceError, ProcessResourceLock, process_lock_registry};

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
    /// 提交事务时的崩溃耐久性策略。
    pub synchronous: SqliteSynchronousMode,
    /// 数据库文件锁定的粒度和持续时间。
    pub locking_mode: SqliteLockingMode,
    /// WAL 自动 checkpoint 的页数阈值；`None` 表示使用 SQLite 默认值。
    pub wal_autocheckpoint: Option<u32>,
}

/// SQLite 的数据库锁定模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqliteLockingMode {
    /// 每次事务结束时释放文件锁，允许其他进程并发访问。
    Normal,
    /// 打开连接期间持有文件锁，读写吞吐更高但排斥其他进程。
    Exclusive,
}

impl SqliteLockingMode {
    const fn as_pragma(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Exclusive => "EXCLUSIVE",
        }
    }
}

/// SQLite 的同步写入策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqliteSynchronousMode {
    Off,
    Normal,
    Full,
    Extra,
}

impl SqliteSynchronousMode {
    const fn as_pragma(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Normal => "NORMAL",
            Self::Full => "FULL",
            Self::Extra => "EXTRA",
        }
    }
}

impl Default for SqliteOptions {
    fn default() -> Self {
        Self {
            busy_timeout: Duration::from_secs(5),
            foreign_keys: true,
            wal: true,
            synchronous: SqliteSynchronousMode::Full,
            locking_mode: SqliteLockingMode::Normal,
            wal_autocheckpoint: None,
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
    coordination: ProcessResourceLock,
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
        let operation_path = path.clone();
        let result = async {
            let coordination = process_lock_registry().resource(&path)?;
            let _guard = coordination.write().await;
            let opened = tokio::task::spawn_blocking(move || open_connection(&path, &options))
                .await
                .map_err(|error| PersistenceError::Task {
                    operation: "open SQLite database",
                    source: error,
                })?;
            let connection = opened?;
            Ok(Self {
                path: operation_path.clone(),
                coordination,
                connection: Arc::new(Mutex::new(connection)),
            })
        }
        .await;
        report_operation_error("open", &operation_path, &result);
        result
    }

    /// 使用默认选项打开或创建数据库，并执行初始化 SQL。
    ///
    /// 数据库文件不存在时会在目标路径创建（含父目录）；`schema_sql` 在
    /// 每次打开时执行，调用方应保证其幂等（如使用 `CREATE TABLE IF NOT EXISTS`）。
    pub async fn open_with_schema(
        path: impl Into<PathBuf>,
        schema_sql: impl Into<String>,
    ) -> Result<Self, PersistenceError> {
        Self::open_with_options_and_schema(path, SqliteOptions::default(), schema_sql).await
    }

    /// 使用显式选项打开或创建数据库，并执行初始化 SQL。
    ///
    /// 在 [`Self::open_with_options`] 的基础上执行建表等初始化语句：
    /// 数据库文件不存在时会在目标路径创建（含父目录），随后按选项配置
    /// pragma 并执行 `schema_sql`。`schema_sql` 在每次打开时执行，
    /// 调用方应保证其幂等（如使用 `CREATE TABLE IF NOT EXISTS`）。
    pub async fn open_with_options_and_schema(
        path: impl Into<PathBuf>,
        options: SqliteOptions,
        schema_sql: impl Into<String>,
    ) -> Result<Self, PersistenceError> {
        let database = Self::open_with_options(path, options).await?;
        let schema_sql = schema_sql.into();
        database.execute_batch(schema_sql).await?;
        Ok(database)
    }

    /// 返回数据库文件路径。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 执行单条使用参数绑定的写入或控制语句。
    pub async fn execute<P>(
        &self,
        sql: impl Into<String>,
        params: P,
    ) -> Result<usize, PersistenceError>
    where
        P: IntoIterator<Item = SqlValue> + Send + 'static,
    {
        let sql = sql.into();
        let result = self
            .with_mut_connection("execute", move |connection| {
                connection.execute(&sql, rusqlite::params_from_iter(params))
            })
            .await;
        report_operation_error("execute", &self.path, &result);
        result
    }

    /// 执行仅由受信任代码构造的多条 SQL 语句。
    pub async fn execute_batch(&self, sql: impl Into<String>) -> Result<(), PersistenceError> {
        let sql = sql.into();
        let result = self
            .with_mut_connection("execute batch", move |connection| connection.execute_batch(&sql))
            .await;
        report_operation_error("execute batch", &self.path, &result);
        result
    }

    /// 查询记录，并通过调用方提供的闭包映射每一行。
    pub async fn query<T, P, F>(
        &self,
        sql: impl Into<String>,
        params: P,
        map_row: F,
    ) -> Result<Vec<T>, PersistenceError>
    where
        T: Send + 'static,
        P: IntoIterator<Item = SqlValue> + Send + 'static,
        F: FnMut(&Row<'_>) -> rusqlite::Result<T> + Send + 'static,
    {
        self.query_with_operation("query", sql, params, map_row)
            .await
    }

    /// 查询记录，并为错误追踪指定调用方的稳定操作名称。
    pub async fn query_with_operation<T, P, F>(
        &self,
        operation: &'static str,
        sql: impl Into<String>,
        params: P,
        mut map_row: F,
    ) -> Result<Vec<T>, PersistenceError>
    where
        T: Send + 'static,
        P: IntoIterator<Item = SqlValue> + Send + 'static,
        F: FnMut(&Row<'_>) -> rusqlite::Result<T> + Send + 'static,
    {
        let sql = sql.into();
        let result = self
            .with_connection(operation, move |connection| {
                let mut statement = connection.prepare(&sql)?;
                let rows =
                    statement.query_map(rusqlite::params_from_iter(params), |row| map_row(row))?;
                rows.collect()
            })
            .await;
        report_operation_error(operation, &self.path, &result);
        result
    }

    /// 在 `BEGIN IMMEDIATE` 事务中运行上层定义的写入操作。
    pub async fn write<T, F>(&self, operation: &'static str, work: F) -> Result<T, PersistenceError>
    where
        T: Send + 'static,
        F: FnOnce(&Transaction<'_>) -> rusqlite::Result<T> + Send + 'static,
    {
        let result = self
            .with_mut_connection(operation, move |connection| {
                let transaction =
                    connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
                let result = work(&transaction)?;
                transaction.commit()?;
                Ok(result)
            })
            .await;
        report_operation_error(operation, &self.path, &result);
        result
    }

    /// 以单个原子事务应用未执行过的迁移。
    pub async fn migrate(&self, mut migrations: Vec<Migration>) -> Result<(), PersistenceError> {
        migrations.sort_by_key(|migration| migration.version);
        if let Some(migration) = migrations.iter().find(|migration| migration.version < 0) {
            let result = Err(PersistenceError::InvalidMigration {
                version: migration.version,
                reason: "migration versions must not be negative",
            });
            report_operation_error("migrate", &self.path, &result);
            return result;
        }
        for pair in migrations.windows(2) {
            if pair[0].version == pair[1].version {
                let result = Err(PersistenceError::InvalidMigration {
                    version: pair[0].version,
                    reason: "migration versions must be unique",
                });
                report_operation_error("migrate", &self.path, &result);
                return result;
            }
        }

        let result = self
            .with_mut_connection("migrate", move |connection| {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            transaction.execute_batch(
                "CREATE TABLE IF NOT EXISTS _sealantern_schema_migrations (\
                    version INTEGER PRIMARY KEY NOT NULL,\
                    name TEXT NOT NULL,\
                    checksum TEXT NOT NULL,\
                    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP\
                 )",
            )?;
            for migration in migrations {
                let checksum = crate::fs::sha256_hex(migration.sql);
                let applied: Option<(String, String)> = transaction
                    .query_row(
                    "SELECT name, checksum FROM _sealantern_schema_migrations WHERE version = ?1",
                    [migration.version],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                    .optional()?;
                if let Some((applied_name, applied_checksum)) = applied {
                    if applied_name != migration.name || applied_checksum != checksum {
                        return Ok(Err(PersistenceError::MigrationIntegrity {
                            version: migration.version,
                            message: "applied name or SQL checksum differs from the migration manifest"
                                .to_owned(),
                        }));
                    }
                } else {
                    transaction.execute_batch(migration.sql)?;
                    transaction.execute(
                        "INSERT INTO _sealantern_schema_migrations (version, name, checksum) VALUES (?1, ?2, ?3)",
                        (migration.version, migration.name, checksum),
                    )?;
                }
            }
            transaction.commit()?;
            Ok(Ok(()))
        })
        .await
        .and_then(|result| result);
        report_operation_error("migrate", &self.path, &result);
        result
    }

    /// 执行参数化写入并返回最后插入的行 ID。
    ///
    /// 适用于 `INTEGER PRIMARY KEY AUTOINCREMENT` 等自增主键场景，
    /// 调用方可据此构造单调递增的游标（如日志序号）。
    pub async fn insert<P>(
        &self,
        sql: impl Into<String>,
        params: P,
    ) -> Result<i64, PersistenceError>
    where
        P: IntoIterator<Item = SqlValue> + Send + 'static,
    {
        let sql = sql.into();
        let result = self
            .with_mut_connection("insert", move |connection| {
                connection.execute(&sql, rusqlite::params_from_iter(params))?;
                Ok(connection.last_insert_rowid())
            })
            .await;
        report_operation_error("insert", &self.path, &result);
        result
    }

    /// 查询至多一行，通过调用方提供的闭包映射该行。
    pub async fn query_one<T, P, F>(
        &self,
        sql: impl Into<String>,
        params: P,
        map_row: F,
    ) -> Result<Option<T>, PersistenceError>
    where
        T: Send + 'static,
        P: IntoIterator<Item = SqlValue> + Send + 'static,
        F: FnMut(&Row<'_>) -> rusqlite::Result<T> + Send + 'static,
    {
        self.query_one_with_operation("query one", sql, params, map_row)
            .await
    }

    /// 查询至多一行，并为错误追踪指定调用方的稳定操作名称。
    pub async fn query_one_with_operation<T, P, F>(
        &self,
        operation: &'static str,
        sql: impl Into<String>,
        params: P,
        map_row: F,
    ) -> Result<Option<T>, PersistenceError>
    where
        T: Send + 'static,
        P: IntoIterator<Item = SqlValue> + Send + 'static,
        F: FnMut(&Row<'_>) -> rusqlite::Result<T> + Send + 'static,
    {
        let sql = sql.into();
        let result = self
            .with_connection(operation, move |connection| {
                let mut statement = connection.prepare(&sql)?;
                let mapped = statement
                    .query_row(rusqlite::params_from_iter(params), map_row)
                    .optional()?;
                Ok(mapped)
            })
            .await;
        report_operation_error(operation, &self.path, &result);
        result
    }

    /// 检查指定表是否存在（不区分普通表与视图）。
    pub async fn table_exists(&self, table: &str) -> Result<bool, PersistenceError> {
        self.query_one_with_operation(
            "check table existence",
            "SELECT 1 FROM sqlite_master WHERE type IN ('table', 'view') AND name = ?1",
            std::iter::once(SqlValue::Text(table.to_owned())),
            |row| row.get::<_, i64>(0),
        )
        .await
        .map(|found| found.is_some())
    }

    /// 检查指定表是否包含指定列（通过 `pragma_table_info` 表值函数）。
    pub async fn column_exists(&self, table: &str, column: &str) -> Result<bool, PersistenceError> {
        self.query_one_with_operation(
            "check column existence",
            "SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2",
            [SqlValue::Text(table.to_owned()), SqlValue::Text(column.to_owned())],
            |row| row.get::<_, i64>(0),
        )
        .await
        .map(|found| found.is_some())
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
        let process_guard = self.coordination.read().await;
        let connection = Arc::clone(&self.connection);

        tokio::task::spawn_blocking(move || {
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
                source: error,
            })
        })
        .await
        .map_err(|error| PersistenceError::Task { operation, source: error })?
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
        let process_guard = self.coordination.write().await;
        let connection = Arc::clone(&self.connection);

        tokio::task::spawn_blocking(move || {
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
                source: error,
            })
        })
        .await
        .map_err(|error| PersistenceError::Task { operation, source: error })?
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
        source: error,
    })?;
    connection
        .busy_timeout(options.busy_timeout)
        .map_err(|error| PersistenceError::Sqlite {
            operation: "configure busy timeout",
            path: path.to_path_buf(),
            source: error,
        })?;
    connection
        .pragma_update(None, "foreign_keys", options.foreign_keys)
        .map_err(|error| PersistenceError::Sqlite {
            operation: "enable foreign keys",
            path: path.to_path_buf(),
            source: error,
        })?;
    if options.wal {
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|error| PersistenceError::Sqlite {
                operation: "enable WAL",
                path: path.to_path_buf(),
                source: error,
            })?;
    }
    connection
        .pragma_update(None, "synchronous", options.synchronous.as_pragma())
        .map_err(|error| PersistenceError::Sqlite {
            operation: "configure synchronous mode",
            path: path.to_path_buf(),
            source: error,
        })?;
    connection
        .pragma_update(None, "locking_mode", options.locking_mode.as_pragma())
        .map_err(|error| PersistenceError::Sqlite {
            operation: "configure locking mode",
            path: path.to_path_buf(),
            source: error,
        })?;
    if let Some(checkpoint) = options.wal_autocheckpoint {
        connection
            .pragma_update(None, "wal_autocheckpoint", checkpoint)
            .map_err(|error| PersistenceError::Sqlite {
                operation: "configure WAL autocheckpoint",
                path: path.to_path_buf(),
                source: error,
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
    use std::error::Error;

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
                [SqlValue::Integer(7), SqlValue::Text("O'Reilly".to_owned())],
            )
            .await
            .unwrap();

        let names = database
            .query(
                "SELECT name FROM records WHERE id = ?1",
                std::iter::once(SqlValue::Integer(7)),
                |row| row.get::<_, String>(0),
            )
            .await
            .unwrap();
        assert_eq!(names, ["O'Reilly"]);
        drop(database);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn insert_returns_increasing_row_ids() {
        let path = database_path("sqlite-insert-rowid");
        let database = SqliteDatabase::open(&path).await.unwrap();
        database
            .execute_batch(
                "CREATE TABLE records (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL)",
            )
            .await
            .unwrap();

        let first = database
            .insert("INSERT INTO records (name) VALUES (?1)", [SqlValue::Text("first".to_owned())])
            .await
            .unwrap();
        let second = database
            .insert("INSERT INTO records (name) VALUES (?1)", [SqlValue::Text("second".to_owned())])
            .await
            .unwrap();

        assert!(first > 0);
        assert!(second > first);
        drop(database);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn query_one_returns_optional_single_row() {
        let path = database_path("sqlite-query-one");
        let database = SqliteDatabase::open(&path).await.unwrap();
        database
            .execute_batch("CREATE TABLE records (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .await
            .unwrap();
        database
            .execute(
                "INSERT INTO records (id, name) VALUES (?1, ?2)",
                [SqlValue::Integer(1), SqlValue::Text("only".to_owned())],
            )
            .await
            .unwrap();

        let found = database
            .query_one("SELECT name FROM records WHERE id = ?1", [SqlValue::Integer(1)], |row| {
                row.get::<_, String>(0)
            })
            .await
            .unwrap();
        let missing = database
            .query_one("SELECT name FROM records WHERE id = ?1", [SqlValue::Integer(999)], |row| {
                row.get::<_, String>(0)
            })
            .await
            .unwrap();

        assert_eq!(found.as_deref(), Some("only"));
        assert!(missing.is_none());
        drop(database);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn detects_table_and_column_existence() {
        let path = database_path("sqlite-schema-introspection");
        let database = SqliteDatabase::open(&path).await.unwrap();
        database
            .execute_batch("CREATE TABLE records (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .await
            .unwrap();

        assert!(database.table_exists("records").await.unwrap());
        assert!(!database.table_exists("missing").await.unwrap());
        assert!(database.column_exists("records", "name").await.unwrap());
        assert!(!database.column_exists("records", "missing").await.unwrap());
        assert!(!database.column_exists("missing", "id").await.unwrap());
        drop(database);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn open_with_schema_creates_database_and_initializes_schema() {
        let path = database_path("sqlite-open-with-schema");
        let schema = "CREATE TABLE IF NOT EXISTS records (\
             id INTEGER PRIMARY KEY AUTOINCREMENT,\
             name TEXT NOT NULL\
         )";
        let database = SqliteDatabase::open_with_schema(&path, schema)
            .await
            .unwrap();
        assert!(path.exists(), "数据库文件应在目标路径创建");

        let inserted = database
            .insert("INSERT INTO records (name) VALUES (?1)", [SqlValue::Text("first".to_owned())])
            .await
            .unwrap();
        assert!(inserted > 0);
        drop(database);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn open_with_schema_is_idempotent_across_reopens() {
        let path = database_path("sqlite-open-with-schema-idempotent");
        let schema = "CREATE TABLE IF NOT EXISTS records (id INTEGER PRIMARY KEY)";
        let first = SqliteDatabase::open_with_schema(&path, schema)
            .await
            .unwrap();
        first
            .execute("INSERT INTO records (id) VALUES (?1)", [SqlValue::Integer(1)])
            .await
            .unwrap();
        drop(first);

        // 幂等 schema 允许重复打开且不丢数据。
        let second = SqliteDatabase::open_with_schema(&path, schema)
            .await
            .unwrap();
        let count = second
            .query_one("SELECT COUNT(*) FROM records", std::iter::empty(), |row| {
                row.get::<_, i64>(0)
            })
            .await
            .unwrap();
        assert_eq!(count, Some(1));
        drop(second);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn applies_the_selected_synchronous_mode() {
        let path = database_path("sqlite-synchronous");
        let database = SqliteDatabase::open_with_options(
            &path,
            SqliteOptions {
                synchronous: SqliteSynchronousMode::Normal,
                ..SqliteOptions::default()
            },
        )
        .await
        .unwrap();

        let mode = database
            .query("PRAGMA synchronous", [], |row| row.get::<_, i64>(0))
            .await
            .unwrap();
        assert_eq!(mode, [1]);
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
    async fn rejects_migration_manifest_drift() {
        let path = database_path("sqlite-migration-drift");
        let database = SqliteDatabase::open(&path).await.unwrap();
        database
            .migrate(vec![Migration {
                version: 1,
                name: "create records",
                sql: "CREATE TABLE records (id INTEGER PRIMARY KEY)",
            }])
            .await
            .unwrap();

        let result = database
            .migrate(vec![Migration {
                version: 1,
                name: "create records",
                sql: "CREATE TABLE records (id INTEGER PRIMARY KEY, name TEXT)",
            }])
            .await;
        assert!(matches!(result, Err(PersistenceError::MigrationIntegrity { version: 1, .. })));
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
        assert!(matches!(&result, Err(PersistenceError::Sqlite { .. })));
        assert!(result.unwrap_err().source().is_some());
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
        assert!(
            tokio::time::timeout(Duration::from_millis(10), &mut second_write)
                .await
                .is_err()
        );
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
