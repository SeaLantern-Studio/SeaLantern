use super::common::{
    check_fs_permission, emit_permission_log_api, get_base_dir_for_permission, validate_fs_path,
    FsContext,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use mlua::{Function, Lua};
use std::fs;

pub(super) fn read(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, path: String| {
        let (base_dir, perm) = get_base_dir_for_permission(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
        )?;
        check_fs_permission(&ctx.permissions, &perm)?;
        let full_path = validate_fs_path(&base_dir, &path)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.read", &path);

        fs::read_to_string(&full_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to read file: {}", e)))
    })
    .map_err(|e| format!("Failed to create fs.read: {}", e))
}

pub(super) fn read_binary(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, path: String| {
        let (base_dir, perm) = get_base_dir_for_permission(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
        )?;
        check_fs_permission(&ctx.permissions, &perm)?;
        let full_path = validate_fs_path(&base_dir, &path)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.read_binary", &path);

        let bytes = fs::read(&full_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to read file: {}", e)))?;
        Ok(BASE64.encode(&bytes))
    })
    .map_err(|e| format!("Failed to create fs.read_binary: {}", e))
}

pub(super) fn exists(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, path: String| {
        let (base_dir, perm) = get_base_dir_for_permission(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
        )?;
        check_fs_permission(&ctx.permissions, &perm)?;
        let full_path = validate_fs_path(&base_dir, &path)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.exists", &path);
        Ok(full_path.exists())
    })
    .map_err(|e| format!("Failed to create fs.exists: {}", e))
}

pub(super) fn list(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, path: String| {
        let (base_dir, perm) = get_base_dir_for_permission(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
        )?;
        check_fs_permission(&ctx.permissions, &perm)?;
        let full_path = validate_fs_path(&base_dir, &path)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.list", &path);

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
    lua.create_function(move |lua, path: String| {
        let (base_dir, perm) = get_base_dir_for_permission(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
        )?;
        check_fs_permission(&ctx.permissions, &perm)?;
        let full_path = validate_fs_path(&base_dir, &path)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.info", &path);

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
        let path = match scope.as_str() {
            "data" => {
                check_fs_permission(&ctx.permissions, "fs.data")?;
                ctx.data_dir.to_string_lossy().to_string()
            }
            "server" => {
                check_fs_permission(&ctx.permissions, "fs.server")?;
                ctx.server_dir.to_string_lossy().to_string()
            }
            "global" => {
                check_fs_permission(&ctx.permissions, "fs.global")?;
                ctx.global_dir.to_string_lossy().to_string()
            }
            _ => {
                return Err(mlua::Error::runtime(
                    "Invalid scope. Must be 'data', 'server', or 'global'",
                ))
            }
        };

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.get_path", &scope);

        Ok(path)
    })
    .map_err(|e| format!("Failed to create fs.get_path: {}", e))
}
