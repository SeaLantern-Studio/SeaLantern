use super::common::{
    emit_permission_log_api, ensure_safe_directory_tree, ensure_safe_path_for_access,
    reject_dangerous_remove_target, resolve_scope_action, validate_fs_path, FsContext,
};
use mlua::{Function, Lua};
use std::fs;

pub(super) fn write(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (scope, path, content): (String, String, String)| {
        let (base_dir, _) = resolve_scope_action(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
            &scope,
            "write",
        )?;
        let full_path = validate_fs_path(&base_dir, &path)?;
        ensure_safe_directory_tree(&base_dir)?;
        if let Some(parent) = full_path.parent() {
            ensure_safe_directory_tree(parent)?;
        }

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.write", &format!("{}:{}", scope, path));

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| mlua::Error::runtime(format!("Failed to create directory: {}", e)))?;
        }

        fs::write(&full_path, content)
            .map_err(|e| mlua::Error::runtime(format!("Failed to write file: {}", e)))
    })
    .map_err(|e| format!("Failed to create fs.write: {}", e))
}

pub(super) fn mkdir(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (scope, path): (String, String)| {
        let (base_dir, _) = resolve_scope_action(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
            &scope,
            "write",
        )?;
        let full_path = validate_fs_path(&base_dir, &path)?;
        ensure_safe_directory_tree(&base_dir)?;
        if let Some(parent) = full_path.parent() {
            ensure_safe_directory_tree(parent)?;
        }

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.mkdir", &format!("{}:{}", scope, path));

        fs::create_dir_all(&full_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to create directory: {}", e)))
    })
    .map_err(|e| format!("Failed to create fs.mkdir: {}", e))
}

pub(super) fn remove(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (scope, path): (String, String)| {
        let (base_dir, _) = resolve_scope_action(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
            &scope,
            "delete",
        )?;
        let full_path = validate_fs_path(&base_dir, &path)?;
        ensure_safe_path_for_access(&full_path)?;
        reject_dangerous_remove_target(&base_dir, &full_path)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.remove", &format!("{}:{}", scope, path));

        if full_path.is_dir() {
            let mut entries = fs::read_dir(&full_path)
                .map_err(|e| mlua::Error::runtime(format!("Failed to inspect directory: {}", e)))?;
            if entries.next().is_some() {
                return Err(mlua::Error::runtime(
                    "Refusing to recursively remove a non-empty directory".to_string(),
                ));
            }
            fs::remove_dir(&full_path)
                .map_err(|e| mlua::Error::runtime(format!("Failed to remove directory: {}", e)))
        } else {
            fs::remove_file(&full_path)
                .map_err(|e| mlua::Error::runtime(format!("Failed to remove file: {}", e)))
        }
    })
    .map_err(|e| format!("Failed to create fs.remove: {}", e))
}
