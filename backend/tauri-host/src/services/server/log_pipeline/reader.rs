use super::state::open_or_create_log_db;
use rusqlite::params;
use std::path::Path;

pub fn get_logs(server_id: &str, since: usize, recent_limit: Option<usize>) -> Vec<String> {
    super::state::resolve_server_path(server_id)
        .ok()
        .and_then(|server_path| read_logs(&server_path, since as u64, recent_limit).ok())
        .unwrap_or_default()
}

pub fn get_all_logs() -> Vec<(String, Vec<String>)> {
    let server_ids = crate::services::global::server_manager()
        .get_server_list()
        .into_iter()
        .map(|server| server.id)
        .collect::<Vec<String>>();

    let mut result = Vec::with_capacity(server_ids.len());
    for server_id in server_ids {
        result.push((server_id.clone(), get_logs(&server_id, 0, None)));
    }
    result
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
