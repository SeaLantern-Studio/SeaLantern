use super::common::{
    emit_permission_log_api, ensure_safe_path_for_access, resolve_scope_action, validate_fs_path,
    FsContext,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use mlua::{Function, Lua};
use std::fs;

pub(super) fn read(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, args: mlua::MultiValue| {
        let (scope, path) = match args.len() {
            1 => {
                let path: String = mlua::FromLuaMulti::from_lua_multi(args, lua)?;
                ("data".to_string(), path)
            }
            2 => mlua::FromLuaMulti::from_lua_multi(args, lua)?,
            _ => {
                return Err(mlua::Error::runtime(
                    "sl.fs.read expects (path) or (scope, path)".to_string(),
                ));
            }
        };
        let (base_dir, _) = resolve_scope_action(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
            &scope,
            "read",
        )?;
        let full_path = validate_fs_path(&base_dir, &path)?;
        ensure_safe_path_for_access(&full_path)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.read", &format!("{}:{}", scope, path));

        fs::read_to_string(&full_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to read file: {}", e)))
    })
    .map_err(|e| format!("Failed to create fs.read: {}", e))
}

pub(super) fn read_binary(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, args: mlua::MultiValue| {
        let (scope, path) = match args.len() {
            1 => {
                let path: String = mlua::FromLuaMulti::from_lua_multi(args, lua)?;
                ("data".to_string(), path)
            }
            2 => mlua::FromLuaMulti::from_lua_multi(args, lua)?,
            _ => {
                return Err(mlua::Error::runtime(
                    "sl.fs.read_binary expects (path) or (scope, path)".to_string(),
                ));
            }
        };
        let (base_dir, _) = resolve_scope_action(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
            &scope,
            "read",
        )?;
        let full_path = validate_fs_path(&base_dir, &path)?;
        ensure_safe_path_for_access(&full_path)?;

        emit_permission_log_api(
            &ctx.plugin_id,
            "sl.fs.read_binary",
            &format!("{}:{}", scope, path),
        );

        let bytes = fs::read(&full_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to read file: {}", e)))?;
        Ok(BASE64.encode(&bytes))
    })
    .map_err(|e| format!("Failed to create fs.read_binary: {}", e))
}

pub(super) fn exists(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, args: mlua::MultiValue| {
        let (scope, path) = match args.len() {
            1 => {
                let path: String = mlua::FromLuaMulti::from_lua_multi(args, lua)?;
                ("data".to_string(), path)
            }
            2 => mlua::FromLuaMulti::from_lua_multi(args, lua)?,
            _ => {
                return Err(mlua::Error::runtime(
                    "sl.fs.exists expects (path) or (scope, path)".to_string(),
                ));
            }
        };
        let (base_dir, _) = resolve_scope_action(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
            &scope,
            "meta",
        )?;
        let full_path = validate_fs_path(&base_dir, &path)?;
        ensure_safe_path_for_access(&full_path)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.exists", &format!("{}:{}", scope, path));
        Ok(full_path.exists())
    })
    .map_err(|e| format!("Failed to create fs.exists: {}", e))
}

pub(super) fn list(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, args: mlua::MultiValue| {
        let (scope, path) = match args.len() {
            1 => {
                let path: String = mlua::FromLuaMulti::from_lua_multi(args, lua)?;
                ("data".to_string(), path)
            }
            2 => mlua::FromLuaMulti::from_lua_multi(args, lua)?,
            _ => {
                return Err(mlua::Error::runtime(
                    "sl.fs.list expects (path) or (scope, path)".to_string(),
                ));
            }
        };
        let (base_dir, _) = resolve_scope_action(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
            &scope,
            "list",
        )?;
        let full_path = validate_fs_path(&base_dir, &path)?;
        ensure_safe_path_for_access(&full_path)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.list", &format!("{}:{}", scope, path));

        let entries = fs::read_dir(&full_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to read directory: {}", e)))?;

        let table = lua.create_table()?;
        let mut i = 1;
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                table.set(i, name.to_string())?;
                i += 1;
            }
        }
        Ok(table)
    })
    .map_err(|e| format!("Failed to create fs.list: {}", e))
}

pub(super) fn info(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, args: mlua::MultiValue| {
        let (scope, path) = match args.len() {
            1 => {
                let path: String = mlua::FromLuaMulti::from_lua_multi(args, lua)?;
                ("data".to_string(), path)
            }
            2 => mlua::FromLuaMulti::from_lua_multi(args, lua)?,
            _ => {
                return Err(mlua::Error::runtime(
                    "sl.fs.info expects (path) or (scope, path)".to_string(),
                ));
            }
        };
        let (base_dir, _) = resolve_scope_action(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
            &scope,
            "meta",
        )?;
        let full_path = validate_fs_path(&base_dir, &path)?;
        ensure_safe_path_for_access(&full_path)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.info", &format!("{}:{}", scope, path));

        let metadata = fs::metadata(&full_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to get file metadata: {}", e)))?;

        let table = lua.create_table()?;
        table.set("size", metadata.len())?;
        table.set("is_dir", metadata.is_dir())?;

        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                table.set("modified", duration.as_secs())?;
            }
        }

        Ok(table)
    })
    .map_err(|e| format!("Failed to create fs.info: {}", e))
}

pub(super) fn get_path(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, scope: String| {
        let (_, _) = resolve_scope_action(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
            &scope,
            "meta",
        )?;
        let path = match scope.as_str() {
            "data" => "sandbox://data".to_string(),
            "server" => "sandbox://server".to_string(),
            "global" => "sandbox://global".to_string(),
            _ => unreachable!(),
        };

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.get_path", &scope);

        Ok(path)
    })
    .map_err(|e| format!("Failed to create fs.get_path: {}", e))
}
