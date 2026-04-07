use super::common::{
    lua_value_from_storage, map_storage_err, read_storage, set_storage_function, with_storage_lock,
    StorageContext,
};
use mlua::{Function, Lua, Table, Value};

pub(super) fn register(
    lua: &Lua,
    storage: &mlua::Table,
    ctx: &StorageContext,
) -> Result<(), String> {
    set_storage_function(storage, "get", get(lua, ctx)?, "storage.set_get_failed")?;
    set_storage_function(storage, "keys", keys(lua, ctx)?, "storage.set_keys_failed")?;
    Ok(())
}

fn get(lua: &Lua, ctx: &StorageContext) -> Result<Function, String> {
    let path = ctx.storage_path.clone();
    let lock = ctx.storage_lock.clone();
    lua.create_function(move |lua, key: String| {
        with_storage_lock(&lock, || {
            let data = read_storage(&path)?;
            match data.get(&key) {
                Some(value) => lua_value_from_storage(lua, value),
                None => Ok(Value::Nil),
            }
        })
    })
    .map_err(|e| map_storage_err("storage.create_get_failed", e))
}

fn keys(lua: &Lua, ctx: &StorageContext) -> Result<Function, String> {
    let path = ctx.storage_path.clone();
    let lock = ctx.storage_lock.clone();
    lua.create_function(move |lua, ()| {
        with_storage_lock(&lock, || {
            let data = read_storage(&path)?;
            let table: Table = lua.create_table()?;
            let mut keys: Vec<_> = data.keys().cloned().collect();
            keys.sort_unstable();
            for (i, key) in keys.into_iter().enumerate() {
                table.set(i + 1, key)?;
            }
            Ok(table)
        })
    })
    .map_err(|e| map_storage_err("storage.create_keys_failed", e))
}
