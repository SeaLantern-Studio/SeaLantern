use super::common::{
    copy_dir_recursive, emit_permission_log_api, ensure_safe_directory_tree,
    ensure_safe_path_for_access, resolve_scope_action, validate_fs_path, FsContext,
};
use mlua::{Function, Lua};
use std::fs;

pub(super) fn copy(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (scope, src, dst): (String, String, String)| {
        let (base_dir, _) = resolve_scope_action(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
            &scope,
            "transfer",
        )?;
        let src_path = validate_fs_path(&base_dir, &src)?;
        let dst_path = validate_fs_path(&base_dir, &dst)?;
        ensure_safe_path_for_access(&src_path)?;
        if let Some(parent) = dst_path.parent() {
            ensure_safe_directory_tree(parent)?;
        }

        emit_permission_log_api(
            &ctx.plugin_id,
            "sl.fs.copy",
            &format!("{}:{}->{}", scope, src, dst),
        );

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)
                .map_err(|e| mlua::Error::runtime(format!("Failed to copy directory: {}", e)))
        } else {
            if dst_path.exists() {
                return Err(mlua::Error::runtime(format!("Destination already exists: {}", dst)));
            }
            fs::copy(&src_path, &dst_path)
                .map_err(|e| mlua::Error::runtime(format!("Failed to copy file: {}", e)))
                .map(|_| ())
        }
    })
    .map_err(|e| format!("Failed to create fs.copy: {}", e))
}

pub(super) fn move_entry(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (scope, src, dst): (String, String, String)| {
        let (base_dir, _) = resolve_scope_action(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
            &scope,
            "transfer",
        )?;
        let src_path = validate_fs_path(&base_dir, &src)?;
        let dst_path = validate_fs_path(&base_dir, &dst)?;
        ensure_safe_path_for_access(&src_path)?;
        if let Some(parent) = dst_path.parent() {
            ensure_safe_directory_tree(parent)?;
        }

        emit_permission_log_api(
            &ctx.plugin_id,
            "sl.fs.move",
            &format!("{}:{}->{}", scope, src, dst),
        );

        fs::rename(&src_path, &dst_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to move file/directory: {}", e)))
    })
    .map_err(|e| format!("Failed to create fs.move: {}", e))
}

pub(super) fn rename_entry(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (scope, old_path, new_path): (String, String, String)| {
        let (base_dir, _) = resolve_scope_action(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
            &scope,
            "transfer",
        )?;
        let old_full_path = validate_fs_path(&base_dir, &old_path)?;
        let new_full_path = validate_fs_path(&base_dir, &new_path)?;
        ensure_safe_path_for_access(&old_full_path)?;
        if let Some(parent) = new_full_path.parent() {
            ensure_safe_directory_tree(parent)?;
        }

        emit_permission_log_api(
            &ctx.plugin_id,
            "sl.fs.rename",
            &format!("{}:{}->{}", scope, old_path, new_path),
        );

        fs::rename(&old_full_path, &new_full_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to rename file/directory: {}", e)))
    })
    .map_err(|e| format!("Failed to create fs.rename: {}", e))
}
