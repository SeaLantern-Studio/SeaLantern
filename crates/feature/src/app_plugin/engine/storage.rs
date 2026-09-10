use std::{
    collections::BTreeMap,
    fs,
    io::ErrorKind,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use mlua::{Lua, Value};
use serde_json::Value as JsonValue;

use crate::app_plugin::AppPluginError;

const MAX_DEPTH: usize = 64;
const MAX_STORAGE_VALUE_BYTES: usize = 256 * 1024;
const MAX_STORAGE_FILE_BYTES: usize = 1024 * 1024;

#[derive(Clone)]
pub(super) struct PluginStorage {
    plugin_id: Arc<str>,
    path: Arc<PathBuf>,
    lock: Arc<Mutex<()>>,
}

impl PluginStorage {
    pub(super) fn new(plugin_id: &str, data_dir: &Path) -> Self {
        Self {
            plugin_id: Arc::from(plugin_id),
            path: Arc::new(data_dir.join("storage.json")),
            lock: Arc::new(Mutex::new(())),
        }
    }

    pub(super) fn get(&self, lua: &Lua, key: String) -> mlua::Result<Value> {
        self.report_failure(
            "get",
            self.with_lock(|| {
                let values = self.read()?;
                values
                    .get(&key)
                    .map(|value| json_to_lua(lua, value, 0))
                    .unwrap_or(Ok(Value::Nil))
            }),
        )
    }

    pub(super) fn keys(&self, lua: &Lua) -> mlua::Result<Value> {
        self.report_failure(
            "keys",
            self.with_lock(|| {
                let table = lua.create_table()?;
                for (index, key) in self.read()?.keys().enumerate() {
                    table.set(index + 1, key.as_str())?;
                }
                Ok(Value::Table(table))
            }),
        )
    }

    pub(super) fn set(&self, key: String, value: Value) -> mlua::Result<()> {
        self.report_failure(
            "set",
            (|| {
                let key = validate_key(key)?;
                let value = lua_to_json(&value, 0)?;
                let value_size = serde_json::to_vec(&value)
                    .map_err(|error| storage_error("serialize value", error))?
                    .len();
                if value_size > MAX_STORAGE_VALUE_BYTES {
                    return Err(storage_limit("value", MAX_STORAGE_VALUE_BYTES));
                }
                self.with_lock(|| {
                    let mut values = self.read()?;
                    values.insert(key, value);
                    self.write(&values)
                })
            })(),
        )
    }

    pub(super) fn remove(&self, key: String) -> mlua::Result<()> {
        self.report_failure(
            "remove",
            (|| {
                let key = validate_key(key)?;
                self.with_lock(|| {
                    let mut values = self.read()?;
                    values.remove(&key);
                    self.write(&values)
                })
            })(),
        )
    }

    fn with_lock<T>(&self, operation: impl FnOnce() -> mlua::Result<T>) -> mlua::Result<T> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| mlua::Error::runtime("plugin storage lock is poisoned"))?;
        operation()
    }

    fn report_failure<T>(&self, operation: &str, result: mlua::Result<T>) -> mlua::Result<T> {
        if let Err(_error) = &result {
            crate::observability::app_plugin_storage_failed(&self.plugin_id, operation);
        }
        result
    }

    fn read(&self) -> mlua::Result<BTreeMap<String, JsonValue>> {
        let file = match fs::File::open(&*self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(BTreeMap::new()),
            Err(error) => return Err(storage_error("open storage", error)),
        };
        let mut bytes = Vec::with_capacity(MAX_STORAGE_FILE_BYTES.min(8 * 1024));
        file.take((MAX_STORAGE_FILE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| storage_error("read storage", error))?;
        if bytes.len() > MAX_STORAGE_FILE_BYTES {
            return Err(storage_limit("file", MAX_STORAGE_FILE_BYTES));
        }
        let content = String::from_utf8(bytes)
            .map_err(|error| storage_error("decode storage as UTF-8", error))?;
        serde_json::from_str(&content).map_err(|error| {
            mlua::Error::runtime(format!("failed to parse plugin storage: {error}"))
        })
    }

    fn write(&self, values: &BTreeMap<String, JsonValue>) -> mlua::Result<()> {
        let content = serde_json::to_vec_pretty(values)
            .map_err(|error| storage_error("serialize storage", error))?;
        if content.len() > MAX_STORAGE_FILE_BYTES {
            return Err(storage_limit("file", MAX_STORAGE_FILE_BYTES));
        }
        sealantern_infra::fs::write_atomic_blocking(&*self.path, &content)
            .map_err(|error| storage_error("replace storage", error))
    }
}

fn validate_key(key: String) -> mlua::Result<String> {
    let key = key.trim().to_owned();
    if key.is_empty() {
        return Err(mlua::Error::runtime("storage key must not be empty"));
    }
    if key.len() > 256 {
        return Err(mlua::Error::runtime("storage key exceeds the 256-byte limit"));
    }
    Ok(key)
}

fn storage_error(operation: &'static str, error: impl std::fmt::Display) -> mlua::Error {
    mlua::Error::runtime(
        AppPluginError::Storage { operation, message: error.to_string() }.to_string(),
    )
}

fn storage_limit(subject: &str, limit: usize) -> mlua::Error {
    mlua::Error::runtime(format!("plugin storage {subject} exceeds the {limit}-byte limit"))
}

pub(super) fn lua_to_json(value: &Value, depth: usize) -> mlua::Result<JsonValue> {
    if depth >= MAX_DEPTH {
        return Err(mlua::Error::runtime("storage value exceeds the 64-level nesting limit"));
    }
    match value {
        Value::Nil => Ok(JsonValue::Null),
        Value::Boolean(value) => Ok(JsonValue::Bool(*value)),
        Value::Integer(value) => Ok(JsonValue::Number((*value).into())),
        Value::Number(value) => serde_json::Number::from_f64(*value)
            .map(JsonValue::Number)
            .ok_or_else(|| mlua::Error::runtime("storage does not support non-finite numbers")),
        Value::String(value) => Ok(JsonValue::String(
            value
                .to_str()
                .map_err(|error| {
                    mlua::Error::runtime(format!("storage string is not UTF-8: {error}"))
                })?
                .to_owned(),
        )),
        Value::Table(table) => {
            let mut array_entries = BTreeMap::new();
            let mut object = serde_json::Map::new();
            for pair in table.pairs::<Value, Value>() {
                let (key, value) = pair?;
                match key {
                    Value::String(key) => {
                        if !array_entries.is_empty() {
                            return Err(mlua::Error::runtime(
                                "storage tables cannot mix array and object keys",
                            ));
                        }
                        object.insert(
                            key.to_str()
                                .map_err(|error| {
                                    mlua::Error::runtime(format!(
                                        "storage object key is not UTF-8: {error}"
                                    ))
                                })?
                                .to_owned(),
                            lua_to_json(&value, depth + 1)?,
                        );
                    }
                    Value::Integer(index) if index > 0 => {
                        if !object.is_empty() {
                            return Err(mlua::Error::runtime(
                                "storage tables cannot mix array and object keys",
                            ));
                        }
                        array_entries.insert(index as usize, lua_to_json(&value, depth + 1)?);
                    }
                    Value::Integer(_) => {
                        return Err(mlua::Error::runtime(
                            "storage arrays must use positive Lua indexes",
                        ));
                    }
                    key => {
                        return Err(mlua::Error::runtime(format!(
                            "storage does not support {} table keys",
                            key.type_name()
                        )));
                    }
                }
            }

            if !object.is_empty() || array_entries.is_empty() {
                return Ok(JsonValue::Object(object));
            }

            let expected_length = array_entries
                .last_key_value()
                .map(|(index, _)| *index)
                .expect("non-empty map has a last key");
            if array_entries.len() != expected_length {
                return Err(mlua::Error::runtime(
                    "storage arrays must use contiguous Lua indexes starting at 1",
                ));
            }

            Ok(JsonValue::Array(array_entries.into_values().collect()))
        }
        value => Err(mlua::Error::runtime(format!(
            "storage does not support Lua {} values",
            value.type_name()
        ))),
    }
}

pub(super) fn json_to_lua(lua: &Lua, value: &JsonValue, depth: usize) -> mlua::Result<Value> {
    if depth >= MAX_DEPTH {
        return Err(mlua::Error::runtime("stored value exceeds the 64-level nesting limit"));
    }
    match value {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(value) => Ok(Value::Boolean(*value)),
        JsonValue::Number(value) => value
            .as_i64()
            .map(Value::Integer)
            .or_else(|| value.as_f64().map(Value::Number))
            .ok_or_else(|| mlua::Error::runtime("stored number cannot be represented in Lua")),
        JsonValue::String(value) => lua.create_string(value).map(Value::String),
        JsonValue::Array(values) => {
            let table = lua.create_table_with_capacity(values.len(), 0)?;
            for (index, value) in values.iter().enumerate() {
                table.set(index + 1, json_to_lua(lua, value, depth + 1)?)?;
            }
            Ok(Value::Table(table))
        }
        JsonValue::Object(values) => {
            let table = lua.create_table_with_capacity(0, values.len())?;
            for (key, value) in values {
                table.set(key.as_str(), json_to_lua(lua, value, depth + 1)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}
