use super::helpers::{json_value_from_lua, lua_value_from_json};
use super::PluginRuntime;
use mlua::{Table, Value};
use serde_json::{Map, Value as JsonValue};
use std::fs;
use std::path::Path;
use std::sync::Arc;

impl PluginRuntime {
    pub(super) fn setup_storage_namespace(&self, sl: &Table) -> Result<(), String> {
        let storage = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create storage table: {}", e))?;

        let storage_path = self.data_dir.join("storage.json");
        let storage_lock = Arc::new(self.storage_lock.clone());

        let path = storage_path.clone();
        let lock = storage_lock.clone();
        let get_fn = self
            .lua
            .create_function(move |lua, key: String| {
                let _guard = lock.lock().unwrap();
                let data = read_storage(&path);
                match data.get(&key) {
                    Some(value) => lua_value_from_json(lua, value, 0),
                    None => Ok(Value::Nil),
                }
            })
            .map_err(|e| format!("Failed to create storage.get: {}", e))?;
        storage
            .set("get", get_fn)
            .map_err(|e| format!("Failed to set storage.get: {}", e))?;

        let path = storage_path.clone();
        let lock = storage_lock.clone();
        let set_fn = self
            .lua
            .create_function(move |_, (key, value): (String, Value)| {
                if key.len() > 256 {
                    return Err(mlua::Error::runtime("Storage key exceeds 256 bytes limit"));
                }
                let _guard = lock.lock().unwrap();
                let mut data = read_storage(&path);
                let json_value = json_value_from_lua(&value, 0)?;

                let value_str = serde_json::to_string(&json_value)
                    .map_err(|e| mlua::Error::runtime(e.to_string()))?;
                if value_str.len() > 1_048_576 {
                    return Err(mlua::Error::runtime("Storage value exceeds 1MB limit"));
                }
                data.insert(key, json_value);

                let total_str = serde_json::to_string(&data)
                    .map_err(|e| mlua::Error::runtime(e.to_string()))?;
                if total_str.len() > 10_485_760 {
                    return Err(mlua::Error::runtime("Storage total size exceeds 10MB limit"));
                }
                if let Err(e) = write_storage(&path, &data) {
                    eprintln!("Storage write error: {}", e);
                    return Err(mlua::Error::runtime(e));
                }
                Ok(())
            })
            .map_err(|e| format!("Failed to create storage.set: {}", e))?;
        storage
            .set("set", set_fn)
            .map_err(|e| format!("Failed to set storage.set: {}", e))?;

        let path = storage_path.clone();
        let lock = storage_lock.clone();
        let remove_fn = self
            .lua
            .create_function(move |_, key: String| {
                let _guard = lock.lock().unwrap();
                let mut data = read_storage(&path);
                data.remove(&key);
                if let Err(e) = write_storage(&path, &data) {
                    eprintln!("Storage write error: {}", e);
                    return Err(mlua::Error::runtime(e));
                }
                Ok(())
            })
            .map_err(|e| format!("Failed to create storage.remove: {}", e))?;
        storage
            .set("remove", remove_fn)
            .map_err(|e| format!("Failed to set storage.remove: {}", e))?;

        let path = storage_path.clone();
        let lock = storage_lock.clone();
        let keys_fn = self
            .lua
            .create_function(move |lua, ()| {
                let _guard = lock.lock().unwrap();
                let data = read_storage(&path);
                let keys: Vec<String> = data.keys().cloned().collect();
                let table = lua.create_table()?;
                for (i, key) in keys.iter().enumerate() {
                    table.set(i + 1, key.clone())?;
                }
                Ok(table)
            })
            .map_err(|e| format!("Failed to create storage.keys: {}", e))?;
        storage
            .set("keys", keys_fn)
            .map_err(|e| format!("Failed to set storage.keys: {}", e))?;

        sl.set("storage", storage)
            .map_err(|e| format!("Failed to set sl.storage: {}", e))?;

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
