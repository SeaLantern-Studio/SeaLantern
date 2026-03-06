use mlua::{Lua, Result as LuaResult, Table, Value};
use serde_json::Value as JsonValue;
use std::path::{Path, PathBuf};

// 错误信息用英文, 中文容易出编码问题

// 最大循环深度, 后来的改的别太大, 可以调
pub(crate) const DEFAULT_MAX_RECURSION_DEPTH: usize = 64;
// 上边的常量的结构体实现
pub(crate) struct ConversionConfig {
    pub max_recursion_depth: usize,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            max_recursion_depth: DEFAULT_MAX_RECURSION_DEPTH,
        }
    }
}

// 从 Lua 值转换为 JSON 值, 递归深度默认值
pub(crate) fn json_value_from_lua(value: &Value, depth: usize) -> Result<JsonValue, mlua::Error> {
    json_value_from_lua_with_config(value, depth, &ConversionConfig::default())
}

// 从 Lua 值转换为 JSON 值, 递归深度可配置
pub(crate) fn json_value_from_lua_with_config(
    value: &Value,
    depth: usize,
    config: &ConversionConfig,
) -> Result<JsonValue, mlua::Error> {
    if depth > config.max_recursion_depth {
        return Err(mlua::Error::runtime(format!(
            "Maximum recursion depth exceeded ({})",
            config.max_recursion_depth
        )));
    }

    match value {
        Value::Nil => Ok(JsonValue::Null),
        Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
        Value::Integer(i) => Ok(JsonValue::Number((*i).into())),
        Value::Number(n) => Ok(serde_json::Number::from_f64(*n)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null)),
        Value::String(s) => {
            Ok(JsonValue::String(s.to_str().map(|s| s.to_string()).unwrap_or_default()))
        }

        // 处理数组和对象(后来的仔细看,别被绕进去了)
        Value::Table(t) => {
            let table_type = classify_table(t);
            match table_type {
                TableType::Array(max_index) => {
                    let mut arr = Vec::with_capacity(max_index);
                    for i in 1..=max_index {
                        if let Ok(v) = t.get::<Value>(i) {
                            arr.push(json_value_from_lua_with_config(&v, depth + 1, config)?);
                        } else {
                            arr.push(JsonValue::Null);
                        }
                    }
                    Ok(JsonValue::Array(arr))
                }
                TableType::Object => {
                    let mut map = serde_json::Map::new();
                    for (k, v) in t.pairs::<String, Value>().flatten() {
                        map.insert(k, json_value_from_lua_with_config(&v, depth + 1, config)?);
                    }
                    Ok(JsonValue::Object(map))
                }
            }
        }
        _ => Ok(JsonValue::Null),
    }
}

#[derive(Debug, Clone, Copy)]
enum TableType {
    Array(usize),
    Object,
}

fn classify_table(t: &Table) -> TableType {
    let mut max_index: usize = 0;
    let mut is_array = true;

    for (k, _) in t.pairs::<Value, Value>().flatten() {
        match k {
            Value::Integer(i) if i > 0 => {
                max_index = max_index.max(i as usize);
            }
            _ => {
                is_array = false;
                break;
            }
        }
    }

    if is_array && max_index > 0 {
        TableType::Array(max_index)
    } else {
        TableType::Object
    }
}

pub(crate) fn lua_value_from_json(lua: &Lua, value: &JsonValue, depth: usize) -> LuaResult<Value> {
    lua_value_from_json_with_config(lua, value, depth, &ConversionConfig::default())
}

pub(crate) fn lua_value_from_json_with_config(
    lua: &Lua,
    value: &JsonValue,
    depth: usize,
    config: &ConversionConfig,
) -> LuaResult<Value> {
    if depth > config.max_recursion_depth {
        return Err(mlua::Error::runtime(format!(
            "Maximum recursion depth exceeded ({})",
            config.max_recursion_depth
        )));
    }

    match value {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(b) => Ok(Value::Boolean(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Ok(Value::Nil)
            }
        }
        JsonValue::String(s) => Ok(Value::String(lua.create_string(s)?)),
        JsonValue::Array(arr) => {
            let table = lua.create_table_with_capacity(arr.len(), 0)?;
            for (i, v) in arr.iter().enumerate() {
                table.set(i + 1, lua_value_from_json_with_config(lua, v, depth + 1, config)?)?;
            }
            Ok(Value::Table(table))
        }
        JsonValue::Object(obj) => {
            let table = lua.create_table_with_capacity(0, obj.len())?;
            for (k, v) in obj.iter() {
                table
                    .set(k.clone(), lua_value_from_json_with_config(lua, v, depth + 1, config)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

pub(crate) fn safe_canonicalize_check(
    base_dir: &Path,
    full_path: &Path,
) -> Result<PathBuf, String> {
    let canonical_base = base_dir
        .canonicalize()
        .map_err(|e| format!("Failed to resolve base directory: {}", e))?;

    if full_path.exists() {
        let canonical = full_path
            .canonicalize()
            .map_err(|e| format!("Failed to resolve path: {}", e))?;
        if !canonical.starts_with(&canonical_base) {
            return Err("Path must be within allowed directory".to_string());
        }
        Ok(canonical)
    } else {
        let mut existing_ancestor = full_path.to_path_buf();
        let mut remaining_parts: Vec<std::ffi::OsString> = Vec::new();

        loop {
            if existing_ancestor.exists() {
                break;
            }
            match existing_ancestor.file_name() {
                Some(name) => {
                    remaining_parts.push(name.to_os_string());
                    existing_ancestor.pop();
                }
                None => {
                    return Err("Cannot find existing ancestor directory".to_string());
                }
            }
        }

        let canonical_ancestor = existing_ancestor
            .canonicalize()
            .map_err(|e| format!("Failed to resolve ancestor directory: {}", e))?;

        if !canonical_ancestor.starts_with(&canonical_base) {
            return Err("Path must be within allowed directory".to_string());
        }

        let mut result = canonical_ancestor;
        for part in remaining_parts.into_iter().rev() {
            result.push(part);
        }
        Ok(result)
    }
}

pub(super) fn validate_path_static(
    plugin_dir: &Path,
    relative_path: &str,
) -> Result<PathBuf, mlua::Error> {
    validate_path(plugin_dir, relative_path)
}

pub(super) fn validate_server_path(
    server_dir: &Path,
    relative_path: &str,
) -> Result<PathBuf, mlua::Error> {
    validate_path(server_dir, relative_path)
}

fn validate_path(base_dir: &Path, relative_path: &str) -> Result<PathBuf, mlua::Error> {
    let path = PathBuf::from(relative_path);

    if path.is_absolute() {
        return Err(mlua::Error::runtime("Absolute paths are not allowed".to_string()));
    }

    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(mlua::Error::runtime("Path cannot contain '..'".to_string()));
        }
    }

    let full_path = base_dir.join(&path);
    safe_canonicalize_check(base_dir, &full_path).map_err(mlua::Error::runtime)
}
