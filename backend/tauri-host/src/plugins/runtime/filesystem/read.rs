use super::common::{
    emit_permission_log_api, ensure_safe_path_for_access, fs_t, fs_t1, fs_t2, resolve_scope_action,
    validate_fs_path, FsContext,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use mlua::{Function, Lua};
use std::fs;

fn parse_scope_path_args(
    lua: &Lua,
    args: mlua::MultiValue,
    usage_key: &str,
) -> mlua::Result<(String, String)> {
    match args.len() {
        1 => {
            let path: String = mlua::FromLuaMulti::from_lua_multi(args, lua)?;
            Ok(("data".to_string(), path))
        }
        2 => mlua::FromLuaMulti::from_lua_multi(args, lua),
        _ => Err(mlua::Error::runtime(fs_t(usage_key))),
    }
}

pub(super) fn read(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, args: mlua::MultiValue| {
        let (scope, path) =
            parse_scope_path_args(lua, args, "plugins.runtime.filesystem.read_usage")?;
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

        fs::read_to_string(&full_path).map_err(|e| {
            mlua::Error::runtime(fs_t1(
                "plugins.runtime.filesystem.read_file_failed",
                e.to_string(),
            ))
        })
    })
    .map_err(|e| fs_t2("plugins.runtime.filesystem.create_api_failed", "fs.read", e.to_string()))
}

pub(super) fn read_binary(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, args: mlua::MultiValue| {
        let (scope, path) =
            parse_scope_path_args(lua, args, "plugins.runtime.filesystem.read_binary_usage")?;
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

        let bytes = fs::read(&full_path).map_err(|e| {
            mlua::Error::runtime(fs_t1(
                "plugins.runtime.filesystem.read_file_failed",
                e.to_string(),
            ))
        })?;
        Ok(BASE64.encode(&bytes))
    })
    .map_err(|e| {
        fs_t2("plugins.runtime.filesystem.create_api_failed", "fs.read_binary", e.to_string())
    })
}

pub(super) fn exists(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, args: mlua::MultiValue| {
        let (scope, path) =
            parse_scope_path_args(lua, args, "plugins.runtime.filesystem.exists_usage")?;
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
    .map_err(|e| fs_t2("plugins.runtime.filesystem.create_api_failed", "fs.exists", e.to_string()))
}

pub(super) fn list(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, args: mlua::MultiValue| {
        let (scope, path) =
            parse_scope_path_args(lua, args, "plugins.runtime.filesystem.list_usage")?;
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

        let entries = fs::read_dir(&full_path).map_err(|e| {
            mlua::Error::runtime(fs_t1("plugins.runtime.filesystem.read_dir_failed", e.to_string()))
        })?;

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
    .map_err(|e| fs_t2("plugins.runtime.filesystem.create_api_failed", "fs.list", e.to_string()))
}

pub(super) fn info(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, args: mlua::MultiValue| {
        let (scope, path) =
            parse_scope_path_args(lua, args, "plugins.runtime.filesystem.info_usage")?;
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

        let metadata = fs::metadata(&full_path).map_err(|e| {
            mlua::Error::runtime(fs_t1(
                "plugins.runtime.filesystem.get_metadata_failed",
                e.to_string(),
            ))
        })?;

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
    .map_err(|e| fs_t2("plugins.runtime.filesystem.create_api_failed", "fs.info", e.to_string()))
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
    .map_err(|e| {
        fs_t2("plugins.runtime.filesystem.create_api_failed", "fs.get_path", e.to_string())
    })
}
