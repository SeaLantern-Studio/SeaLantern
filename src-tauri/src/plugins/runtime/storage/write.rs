use super::common::{
    map_storage_err, read_storage, set_storage_function, storage_runtime_err,
    storage_value_from_lua, with_storage_lock, write_storage, StorageContext, MAX_KEY_LENGTH,
    MAX_TOTAL_SIZE, MAX_VALUE_SIZE,
};
use mlua::{Function, Lua, Value};

pub(super) fn register(
    lua: &Lua,
    storage: &mlua::Table,
    ctx: &StorageContext,
) -> Result<(), String> {
    set_storage_function(storage, "set", set(lua, ctx)?, "storage.set_set_failed")?;
    set_storage_function(storage, "remove", remove(lua, ctx)?, "storage.set_remove_failed")?;
    Ok(())
}

fn set(lua: &Lua, ctx: &StorageContext) -> Result<Function, String> {
    let path = ctx.storage_path.clone();
    let lock = ctx.storage_lock.clone();
    lua.create_function(move |_, (key, value): (String, Value)| {
        let key = key.trim().to_string();
        if key.is_empty() {
            return Err(storage_runtime_err("storage.key_empty"));
        }
        if key.len() > MAX_KEY_LENGTH {
            return Err(storage_runtime_err("storage.key_too_long"));
        }

        let json_value = storage_value_from_lua(&value)?;
        let value_bytes =
            serde_json::to_vec(&json_value).map_err(|e| mlua::Error::runtime(e.to_string()))?;
        if value_bytes.len() > MAX_VALUE_SIZE {
            return Err(storage_runtime_err("storage.value_too_large"));
        }

        with_storage_lock(&lock, || {
            let mut data = read_storage(&path)?;
            data.insert(key, json_value);

            let total_bytes =
                serde_json::to_vec(&data).map_err(|e| mlua::Error::runtime(e.to_string()))?;
            if total_bytes.len() > MAX_TOTAL_SIZE {
                return Err(storage_runtime_err("storage.total_too_large"));
            }

            write_storage(&path, &data)?;
            Ok(())
        })
    })
    .map_err(|e| map_storage_err("storage.create_set_failed", e))
}

fn remove(lua: &Lua, ctx: &StorageContext) -> Result<Function, String> {
    let path = ctx.storage_path.clone();
    let lock = ctx.storage_lock.clone();
    lua.create_function(move |_, key: String| {
        let key = key.trim().to_string();
        if key.is_empty() {
            return Err(storage_runtime_err("storage.key_empty"));
        }

        with_storage_lock(&lock, || {
            let mut data = read_storage(&path)?;
            data.remove(&key);
            write_storage(&path, &data)?;
            Ok(())
        })
    })
    .map_err(|e| map_storage_err("storage.create_remove_failed", e))
}
