//! 服务器日志数据库的初始化与读取。
//!
//! 提供服务器控制台日志的持久化存储访问：按服务器目录初始化 SQLite
//! 日志库（`log_lines` 表），并按行号游标增量读取。数据访问复用
//! `infra` 的调用方无关 [`SqliteDatabase`]，本模块只承载日志表结构与
//! 读写语义，供服务器进程服务（输出管线）与宿主命令消费。

use std::path::Path;

use sealantern_infra::persistence::{PersistenceError, SqlValue, SqliteDatabase};

/// 服务器日志数据库文件名（存放在服务器目录下）。
pub const LOG_DATABASE_FILE: &str = "sea_lantern_logs.sqlite";

/// 日志来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSource {
    /// Sea Lantern 自身写入的说明性日志。
    SeaLantern,
    /// 服务器进程输出（stdout / stderr）。
    Server,
}

impl LogSource {
    /// 数据库中的来源标识。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SeaLantern => "sealantern",
            Self::Server => "server",
        }
    }
}

/// 日志库建表语句（幂等）。
const LOG_SCHEMA: &str = "CREATE TABLE IF NOT EXISTS log_lines (\
     id INTEGER PRIMARY KEY AUTOINCREMENT,\
     timestamp INTEGER NOT NULL,\
     source TEXT NOT NULL CHECK(source IN ('sealantern', 'server')),\
     line TEXT NOT NULL\
 )";

/// 一条持久化的日志行（含递增游标）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLine {
    /// 行号（单调递增游标，用于增量读取）。
    pub id: i64,
    /// 写入时刻（Unix 秒）。
    pub timestamp: i64,
    /// 日志来源标识（`sealantern` / `server`）。
    pub source: String,
    /// 日志行文本。
    pub line: String,
}

/// 打开（或创建）服务器日志数据库，并确保表结构存在。
///
/// 数据库文件创建在 `server_path` 下的 [`LOG_DATABASE_FILE`]；
/// 重复打开幂等，不丢失已有数据。
pub async fn open_log_database(server_path: &Path) -> Result<SqliteDatabase, PersistenceError> {
    SqliteDatabase::open_with_schema(server_path.join(LOG_DATABASE_FILE), LOG_SCHEMA).await
}

/// 读取 `id` 大于 `since` 的日志行，按行号升序返回。
///
/// `recent_limit` 提供时，仅返回最近 `recent_limit` 行窗口内的匹配行
/// （用于前端"最近 N 行"滚动视图）；否则返回全部匹配行。
pub async fn read_logs(
    database: &SqliteDatabase,
    since: i64,
    recent_limit: Option<i64>,
) -> Result<Vec<LogLine>, PersistenceError> {
    let rows = if let Some(limit) = recent_limit.filter(|limit| *limit > 0) {
        database
            .query(
                "SELECT id, timestamp, source, line FROM (\
                     SELECT id, timestamp, source, line FROM log_lines \
                     WHERE id > ?1 ORDER BY id DESC LIMIT ?2\
                 ) recent ORDER BY id ASC",
                [SqlValue::Integer(since), SqlValue::Integer(limit)],
                map_log_line,
            )
            .await?
    } else {
        database
            .query(
                "SELECT id, timestamp, source, line FROM log_lines \
                 WHERE id > ?1 ORDER BY id ASC",
                std::iter::once(SqlValue::Integer(since)),
                map_log_line,
            )
            .await?
    };
    Ok(rows)
}

fn map_log_line(row: &rusqlite::Row<'_>) -> rusqlite::Result<LogLine> {
    Ok(LogLine {
        id: row.get(0)?,
        timestamp: row.get(1)?,
        source: row.get(2)?,
        line: row.get(3)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn open_test_database(label: &str) -> (tempfile::TempDir, SqliteDatabase) {
        let directory = tempfile::tempdir().expect("临时目录应创建成功");
        let database = open_log_database(directory.path())
            .await
            .expect("日志数据库应初始化成功");
        (directory, database)
    }

    async fn insert_line(
        database: &SqliteDatabase,
        timestamp: i64,
        source: LogSource,
        line: &str,
    ) -> i64 {
        database
            .insert(
                "INSERT INTO log_lines (timestamp, source, line) VALUES (?1, ?2, ?3)",
                [
                    SqlValue::Integer(timestamp),
                    SqlValue::Text(source.as_str().to_owned()),
                    SqlValue::Text(line.to_owned()),
                ],
            )
            .await
            .expect("日志行应写入成功")
    }

    #[tokio::test]
    async fn open_log_database_is_idempotent_and_preserves_data() {
        let (directory, first) = open_test_database("log-open-idempotent").await;
        let id = insert_line(&first, 1000, LogSource::Server, "first line").await;

        // 重复打开不报错，数据不丢失。
        let second = open_log_database(directory.path())
            .await
            .expect("重复打开应幂等");
        let lines = read_logs(&second, 0, None).await.expect("读取应成功");

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].id, id);
        assert_eq!(lines[0].timestamp, 1000);
        assert_eq!(lines[0].source, "server");
        assert_eq!(lines[0].line, "first line");
    }

    #[tokio::test]
    async fn read_logs_returns_only_lines_after_since() {
        let (_directory, database) = open_test_database("log-read-since").await;
        insert_line(&database, 1000, LogSource::SeaLantern, "sea line").await;
        let second = insert_line(&database, 2000, LogSource::Server, "server line").await;

        let lines = read_logs(&database, second - 1, None)
            .await
            .expect("读取应成功");

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].line, "server line");
    }

    #[tokio::test]
    async fn read_logs_with_recent_limit_returns_latest_window() {
        let (_directory, database) = open_test_database("log-read-limit").await;
        for index in 1..=5 {
            insert_line(&database, index * 1000, LogSource::Server, &format!("line {index}")).await;
        }

        // 最近 3 行窗口内、id > 0 的行。
        let lines = read_logs(&database, 0, Some(3)).await.expect("读取应成功");
        let texts: Vec<&str> = lines.iter().map(|line| line.line.as_str()).collect();
        assert_eq!(texts, ["line 3", "line 4", "line 5"]);

        // 窗口与 since 组合：id > 3 且最近 3 行内 → 只剩 line 4、line 5。
        let lines = read_logs(&database, 3, Some(3)).await.expect("读取应成功");
        let texts: Vec<&str> = lines.iter().map(|line| line.line.as_str()).collect();
        assert_eq!(texts, ["line 4", "line 5"]);
    }
}
