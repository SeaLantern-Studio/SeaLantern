use super::helpers::{json_value_from_lua, lua_value_from_json};
use super::PluginRuntime;
use crate::services::global::i18n_service;
use mlua::{Table, Value};
use serde_json::{Map, Value as JsonValue};
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// 存储限制常量
const MAX_KEY_LENGTH: usize = 256;
const MAX_VALUE_SIZE: usize = 1024 * 1024; // 1 MiB
const MAX_TOTAL_SIZE: usize = 10 * 1024 * 1024; // 10 MiB

impl PluginRuntime {
    fn map_storage_err(key: &str, e: mlua::Error) -> String {
        format!("{}: {}", i18n_service().t(key), e)
    }

    fn with_storage_lock<T>(
        lock: &Arc<std::sync::Mutex<()>>,
        f: impl FnOnce() -> Result<T, mlua::Error>,
    ) -> Result<T, mlua::Error> {
        let _guard = lock.lock().unwrap();
        f()
    }

    pub(super) fn setup_storage_namespace(&self, sl: &Table) -> Result<(), String> {
        let storage = self
            .lua
            .create_table()
            .map_err(|e| Self::map_storage_err("storage.create_table_failed", e))?;

        let storage_path = Arc::new(self.data_dir.join("storage.json"));
        let storage_lock = Arc::new(self.storage_lock.clone());

        // get
        let path = storage_path.clone();
        let lock = storage_lock.clone();
        let get_fn = self
            .lua
            .create_function(move |lua, key: String| {
                Self::with_storage_lock(&lock, || {
                    let data = read_storage(&path);
                    match data.get(&key) {
                        Some(value) => lua_value_from_json(lua, value, 0),
                        None => Ok(Value::Nil),
                    }
                })
            })
            .map_err(|e| Self::map_storage_err("storage.create_get_failed", e))?;
        storage
            .set("get", get_fn)
            .map_err(|e| Self::map_storage_err("storage.set_get_failed", e))?;

        // set
        let path = storage_path.clone();
        let lock = storage_lock.clone();
        let set_fn = self
            .lua
            .create_function(move |_, (key, value): (String, Value)| {
                if key.len() > MAX_KEY_LENGTH {
                    return Err(mlua::Error::runtime(i18n_service().t("storage.key_too_long")));
                }

                let json_value = json_value_from_lua(&value, 0)?;
                let value_str = serde_json::to_string(&json_value)
                    .map_err(|e| mlua::Error::runtime(e.to_string()))?;
                if value_str.len() > MAX_VALUE_SIZE {
                    return Err(mlua::Error::runtime(i18n_service().t("storage.value_too_large")));
                }

                Self::with_storage_lock(&lock, || {
                    let mut data = read_storage(&path);
                    data.insert(key, json_value);

                    let total_str = serde_json::to_string(&data)
                        .map_err(|e| mlua::Error::runtime(e.to_string()))?;
                    if total_str.len() > MAX_TOTAL_SIZE {
                        return Err(mlua::Error::runtime(
                            i18n_service().t("storage.total_too_large"),
                        ));
                    }

                    write_storage(&path, &data).map_err(mlua::Error::runtime)?;
                    Ok(())
                })
            })
            .map_err(|e| Self::map_storage_err("storage.create_set_failed", e))?;
        storage
            .set("set", set_fn)
            .map_err(|e| Self::map_storage_err("storage.set_set_failed", e))?;

        // remove
        let path = storage_path.clone();
        let lock = storage_lock.clone();
        let remove_fn = self
            .lua
            .create_function(move |_, key: String| {
                Self::with_storage_lock(&lock, || {
                    let mut data = read_storage(&path);
                    data.remove(&key);
                    write_storage(&path, &data).map_err(mlua::Error::runtime)?;
                    Ok(())
                })
            })
            .map_err(|e| Self::map_storage_err("storage.create_remove_failed", e))?;
        storage
            .set("remove", remove_fn)
            .map_err(|e| Self::map_storage_err("storage.set_remove_failed", e))?;

        // keys
        let path = storage_path.clone();
        let lock = storage_lock.clone();
        let keys_fn = self
            .lua
            .create_function(move |lua, ()| {
                Self::with_storage_lock(&lock, || {
                    let data = read_storage(&path);
                    let table = lua.create_table()?;
                    for (i, key) in data.keys().enumerate() {
                        table.set(i + 1, key.clone())?;
                    }
                    Ok(table)
                })
            })
            .map_err(|e| Self::map_storage_err("storage.create_keys_failed", e))?;
        storage
            .set("keys", keys_fn)
            .map_err(|e| Self::map_storage_err("storage.set_keys_failed", e))?;

        sl.set("storage", storage)
            .map_err(|e| Self::map_storage_err("storage.set_storage_failed", e))?;

        Ok(())
    }
}

fn read_storage(path: &Path) -> Map<String, JsonValue> {
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Map::new(),
    }
}

fn write_storage(path: &Path, data: &Map<String, JsonValue>) -> Result<(), String> {
    let content = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize storage: {}", e))?;
    fs::write(path, content).map_err(|e| format!("Failed to write storage: {}", e))
}
