use rusqlite::Connection;
use std::path::Path;
use std::time::Duration;

pub fn init_sqlite_log_db(db_path: &Path) -> Result<Connection, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.busy_timeout(Duration::from_millis(2000))
        .map_err(|e| e.to_string())?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| e.to_string())?;
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| e.to_string())?;
    conn.pragma_update(None, "locking_mode", "NORMAL")
        .map_err(|e| e.to_string())?;
    conn.pragma_update(None, "wal_autocheckpoint", 1000)
        .map_err(|e| e.to_string())?;

    conn.execute_batch(
        r#"CREATE TABLE IF NOT EXISTS log_lines (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             timestamp INTEGER NOT NULL,
             source TEXT NOT NULL CHECK(source IN ('sealantern','server')),
             line TEXT NOT NULL
         );"#,
    )
    .map_err(|e| e.to_string())?;

    let has_timestamp = table_has_column(&conn, "log_lines", "timestamp")?;
    let has_source = table_has_column(&conn, "log_lines", "source")?;
    if !has_timestamp || !has_source {
        conn.execute_batch(
            r#"DROP TABLE IF EXISTS log_lines;
             CREATE TABLE log_lines (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               timestamp INTEGER NOT NULL,
               source TEXT NOT NULL CHECK(source IN ('sealantern','server')),
               line TEXT NOT NULL
             );"#,
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(conn)
}

fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool, String> {
    let sql = format!("PRAGMA table_info({})", table);
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let name: String = row.get(1).map_err(|e| e.to_string())?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}
