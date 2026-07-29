use crate::plugins::runtime::shared::{json_value_from_lua, lua_value_from_json};
use crate::services::global::i18n_service;
use mlua::{Lua, Table, Value};
use serde_json::{Map, Value as JsonValue};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 存储限制常量
pub(super) const MAX_KEY_LENGTH: usize = 256;
pub(super) const MAX_VALUE_SIZE: usize = 1024 * 1024;
pub(super) const MAX_TOTAL_SIZE: usize = 10 * 1024 * 1024;

#[derive(Clone)]
pub(super) struct StorageContext {
    pub(super) storage_path: Arc<PathBuf>,
    pub(super) storage_lock: Arc<std::sync::Mutex<()>>,
}

impl StorageContext {
    pub(super) fn new(data_dir: &Path, storage_lock: Arc<std::sync::Mutex<()>>) -> Self {
        Self {
            storage_path: Arc::new(data_dir.join("storage.json")),
            storage_lock,
        }
    }
}

pub(super) fn map_storage_err(key: &str, e: mlua::Error) -> String {
    format!("{}: {}", i18n_service().t(key), e)
}

pub(super) fn storage_runtime_err(key: &str) -> mlua::Error {
    mlua::Error::runtime(i18n_service().t(key))
}

pub(super) fn with_storage_lock<T>(
    lock: &Arc<std::sync::Mutex<()>>,
    f: impl FnOnce() -> Result<T, mlua::Error>,
) -> Result<T, mlua::Error> {
    let _guard = lock
        .lock()
        .map_err(|_| storage_runtime_err("storage.lock_failed"))?;
    f()
}

pub(super) fn create_storage_table(lua: &Lua) -> Result<Table, String> {
    lua.create_table()
        .map_err(|e| map_storage_err("storage.create_table_failed", e))
}

pub(super) fn set_storage_function(
    storage: &Table,
    name: &str,
    function: mlua::Function,
    error_key: &str,
) -> Result<(), String> {
    storage
        .set(name, function)
        .map_err(|e| map_storage_err(error_key, e))
}

pub(super) fn set_storage_table(sl: &Table, storage: Table) -> Result<(), String> {
    sl.set("storage", storage)
        .map_err(|e| map_storage_err("storage.set_storage_failed", e))
}

pub(super) fn read_storage(path: &Path) -> Result<Map<String, JsonValue>, mlua::Error> {
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).map_err(|e| {
            mlua::Error::runtime(format!("{}: {}", i18n_service().t("storage.invalid_json"), e))
        }),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(Map::new()),
        Err(err) => Err(mlua::Error::runtime(format!(
            "{}: {}",
            i18n_service().t("storage.read_failed"),
            err
        ))),
    }
}

pub(super) fn write_storage(path: &Path, data: &Map<String, JsonValue>) -> Result<(), mlua::Error> {
    let content = serde_json::to_vec_pretty(data).map_err(|e| {
        mlua::Error::runtime(format!("{}: {}", i18n_service().t("storage.serialize_failed"), e))
    })?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            mlua::Error::runtime(format!("{}: {}", i18n_service().t("storage.write_failed"), e))
        })?;
    }

    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, content).map_err(|e| {
        mlua::Error::runtime(format!("{}: {}", i18n_service().t("storage.write_failed"), e))
    })?;

    fs::rename(&tmp_path, path).map_err(|e| {
        let _ = fs::remove_file(&tmp_path);
        mlua::Error::runtime(format!("{}: {}", i18n_service().t("storage.write_failed"), e))
    })
}

pub(super) fn lua_value_from_storage(lua: &Lua, value: &JsonValue) -> Result<Value, mlua::Error> {
    lua_value_from_json(lua, value, 0)
}

pub(super) fn storage_value_from_lua(value: &Value) -> Result<JsonValue, mlua::Error> {
    json_value_from_lua(value, 0)
}
