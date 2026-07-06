use super::common::{
    emit_permission_log_api, ensure_safe_directory_tree, ensure_safe_path_for_access, fs_t, fs_t1,
    fs_t2, reject_dangerous_remove_target, resolve_scope_action, validate_fs_path, FsContext,
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
            fs::create_dir_all(parent).map_err(|e| {
                mlua::Error::runtime(fs_t1(
                    "plugins.runtime.filesystem.create_directory_failed",
                    e.to_string(),
                ))
            })?;
        }

        fs::write(&full_path, content).map_err(|e| {
            mlua::Error::runtime(fs_t1(
                "plugins.runtime.filesystem.write_file_failed",
                e.to_string(),
            ))
        })
    })
    .map_err(|e| fs_t2("plugins.runtime.filesystem.create_api_failed", "fs.write", e.to_string()))
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

        fs::create_dir_all(&full_path).map_err(|e| {
            mlua::Error::runtime(fs_t1(
                "plugins.runtime.filesystem.create_directory_failed",
                e.to_string(),
            ))
        })
    })
    .map_err(|e| fs_t2("plugins.runtime.filesystem.create_api_failed", "fs.mkdir", e.to_string()))
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
            let mut entries = fs::read_dir(&full_path).map_err(|e| {
                mlua::Error::runtime(fs_t1(
                    "plugins.runtime.filesystem.inspect_dir_failed",
                    e.to_string(),
                ))
            })?;
            if entries.next().is_some() {
                return Err(mlua::Error::runtime(fs_t(
                    "plugins.runtime.filesystem.remove_non_empty_dir_forbidden",
                )));
            }
            fs::remove_dir(&full_path).map_err(|e| {
                mlua::Error::runtime(fs_t1(
                    "plugins.runtime.filesystem.remove_dir_failed",
                    e.to_string(),
                ))
            })
        } else {
            fs::remove_file(&full_path).map_err(|e| {
                mlua::Error::runtime(fs_t1(
                    "plugins.runtime.filesystem.remove_file_failed",
                    e.to_string(),
                ))
            })
        }
    })
    .map_err(|e| fs_t2("plugins.runtime.filesystem.create_api_failed", "fs.remove", e.to_string()))
}
