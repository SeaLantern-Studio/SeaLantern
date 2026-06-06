use super::state::open_or_create_log_db;
use rusqlite::params;
use std::path::Path;

pub fn get_logs_checked(
    server_id: &str,
    since: usize,
    recent_limit: Option<usize>,
) -> Result<Vec<String>, String> {
    let server_path = super::state::resolve_server_path(server_id)?;
    read_logs(&server_path, since as u64, recent_limit)
}

pub fn get_all_logs_checked() -> Result<Vec<(String, Vec<String>)>, String> {
    let server_ids = crate::services::global::server_manager()
        .get_server_list_checked()?
        .into_iter()
        .map(|server| server.id)
        .collect::<Vec<String>>();

    collect_logs_for_server_ids(&server_ids, |server_id| get_logs_checked(server_id, 0, None))
}

fn collect_logs_for_server_ids<F>(
    server_ids: &[String],
    mut read_logs_for_server: F,
) -> Result<Vec<(String, Vec<String>)>, String>
where
    F: FnMut(&str) -> Result<Vec<String>, String>,
{
    let mut result = Vec::with_capacity(server_ids.len());
    for server_id in server_ids {
        let logs = read_logs_for_server(server_id)?;
        result.push((server_id.clone(), logs));
    }
    Ok(result)
}

pub fn read_logs(
    server_path: &Path,
    since: u64,
    recent_limit: Option<usize>,
) -> Result<Vec<String>, String> {
    let conn = open_or_create_log_db(server_path)?;
    let mut logs = Vec::new();

    if let Some(limit) = recent_limit.filter(|value| *value > 0) {
        let mut stmt = conn
            .prepare(
                r#"SELECT line FROM (
                       SELECT rowid, line FROM log_lines ORDER BY rowid DESC LIMIT ?1
                   ) recent
                   ORDER BY rowid ASC LIMIT -1 OFFSET ?2"#,
            )
            .map_err(|e| format!("准备日志读取失败: {}", e))?;
        let rows = stmt
            .query_map(params![limit as i64, since as i64], |row| row.get::<_, String>(0))
            .map_err(|e| format!("读取日志失败: {}", e))?;
        for line in rows {
            logs.push(line.map_err(|e| format!("解析日志失败: {}", e))?);
        }
    } else {
        let mut stmt = conn
            .prepare("SELECT line FROM log_lines ORDER BY rowid ASC LIMIT -1 OFFSET ?1")
            .map_err(|e| format!("准备日志读取失败: {}", e))?;
        let rows = stmt
            .query_map(params![since as i64], |row| row.get::<_, String>(0))
            .map_err(|e| format!("读取日志失败: {}", e))?;
        for line in rows {
            logs.push(line.map_err(|e| format!("解析日志失败: {}", e))?);
        }
    }
    Ok(logs)
}

#[cfg(test)]
mod tests {
    use super::{collect_logs_for_server_ids, read_logs};

    #[test]
    fn read_logs_surfaces_database_open_failures() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let blocked_root = temp_dir.path().join("blocked-root");
        std::fs::write(&blocked_root, b"not a directory")
            .expect("file-backed log root should exist");
        let blocked_server_path = blocked_root.join("server-a");

        let error = read_logs(&blocked_server_path, 0, None)
            .expect_err("log DB open failure should not be downgraded to empty logs");

        assert!(
            error.contains("打开日志数据库失败") || error.contains("重建日志数据库失败"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn collect_logs_for_server_ids_surfaces_first_read_failure() {
        let server_ids = vec!["server-a".to_string(), "server-b".to_string()];

        let error = collect_logs_for_server_ids(&server_ids, |server_id| match server_id {
            "server-a" => Ok(vec!["line-1".to_string()]),
            "server-b" => Err("db broken".to_string()),
            other => Err(format!("unexpected server id: {other}")),
        })
        .expect_err("aggregate log loading should not silently skip failed servers");

        assert_eq!(error, "db broken");
    }
}
