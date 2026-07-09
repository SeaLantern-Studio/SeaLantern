use crate::plugins::runtime::permissions::is_any_fs_permission;
use crate::plugins::runtime::shared::validate_path_static;
use crate::services::global::i18n_service;
use crate::utils::logger::log_error_ctx;
use mlua::{Lua, Table};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(super) struct FsContext {
    pub(super) data_dir: PathBuf,
    pub(super) server_dir: PathBuf,
    pub(super) global_dir: PathBuf,
    pub(super) plugin_id: String,
    pub(super) permissions: Vec<String>,
}

impl FsContext {
    pub(super) fn new(
        data_dir: PathBuf,
        server_dir: PathBuf,
        global_dir: PathBuf,
        plugin_id: String,
        permissions: Vec<String>,
    ) -> Self {
        Self {
            data_dir,
            server_dir,
            global_dir,
            plugin_id,
            permissions,
        }
    }
}

pub(super) fn fs_t(key: &str) -> String {
    i18n_service().t(key)
}

pub(super) fn fs_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    i18n_service().t_with_options(key, &m)
}

pub(super) fn fs_t2(key: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    i18n_service().t_with_options(key, &m)
}

pub(super) fn create_fs_table(lua: &Lua) -> Result<Table, String> {
    lua.create_table()
        .map_err(|e| fs_t1("plugins.runtime.filesystem.create_table_failed", e.to_string()))
}

pub(super) fn set_fs_table(sl: &Table, fs_table: Table) -> Result<(), String> {
    sl.set("fs", fs_table)
        .map_err(|e| fs_t1("plugins.runtime.filesystem.set_namespace_failed", e.to_string()))
}

pub(super) fn set_fs_function(
    fs_table: &Table,
    name: &str,
    function: mlua::Function,
) -> Result<(), String> {
    fs_table
        .set(name, function)
        .map_err(|e| fs_t2("plugins.runtime.filesystem.set_api_failed", name, e.to_string()))
}

pub(crate) fn has_any_fs_permission(perms: &[String]) -> bool {
    perms
        .iter()
        .any(|permission| is_any_fs_permission(permission))
}

fn resolve_scope_permission(scope: &str) -> Result<&'static str, mlua::Error> {
    match scope {
        "data" => Ok("fs.data"),
        "server" => Ok("fs.server"),
        "global" => Ok("fs.global"),
        _ => Err(mlua::Error::runtime(fs_t("plugins.runtime.filesystem.invalid_scope"))),
    }
}

pub(super) fn resolve_scope_action(
    data_dir: &Path,
    server_dir: &Path,
    global_dir: &Path,
    perms: &[String],
    scope: &str,
    action: &str,
) -> Result<(PathBuf, String), mlua::Error> {
    let scope_permission = resolve_scope_permission(scope)?;
    let action_permission = format!("{}.{}", scope_permission, action);

    if !(perms.iter().any(|p| p == scope_permission)
        || perms.iter().any(|p| p == &action_permission))
    {
        return Err(mlua::Error::runtime(fs_t2(
            "plugins.runtime.filesystem.permission_required",
            scope_permission,
            action_permission,
        )));
    }

    let base_dir = match scope {
        "data" => data_dir.to_path_buf(),
        "server" => server_dir.to_path_buf(),
        "global" => global_dir.to_path_buf(),
        _ => unreachable!(),
    };

    Ok((base_dir, action_permission))
}

pub(super) fn validate_fs_path(base_dir: &Path, path: &str) -> Result<PathBuf, mlua::Error> {
    validate_path_static(base_dir, path)
}

fn ensure_not_symlink(path: &Path) -> Result<(), mlua::Error> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink() {
            return Err(mlua::Error::runtime(fs_t1(
                "plugins.runtime.filesystem.symlink_forbidden",
                path.display().to_string(),
            )));
        }
    }

    Ok(())
}

pub(super) fn ensure_safe_directory_tree(path: &Path) -> Result<(), mlua::Error> {
    let mut current = PathBuf::new();

    for component in path.components() {
        current.push(component.as_os_str());
        ensure_not_symlink(&current)?;
    }

    Ok(())
}

pub(super) fn ensure_safe_path_for_access(path: &Path) -> Result<(), mlua::Error> {
    ensure_safe_directory_tree(path)?;
    ensure_not_symlink(path)
}

pub(super) fn reject_dangerous_remove_target(
    base_dir: &Path,
    full_path: &Path,
) -> Result<(), mlua::Error> {
    let canonical_base = fs::canonicalize(base_dir).unwrap_or_else(|_| base_dir.to_path_buf());
    let canonical_target = fs::canonicalize(full_path).unwrap_or_else(|_| full_path.to_path_buf());

    if canonical_target == canonical_base {
        return Err(mlua::Error::runtime(fs_t(
            "plugins.runtime.filesystem.sandbox_root_remove_forbidden",
        )));
    }

    Ok(())
}

pub(super) fn emit_permission_log_api(plugin_id: &str, api_name: &str, detail: &str) {
    if let Err(e) =
        crate::plugins::api::emit_permission_log(plugin_id, "api_call", api_name, detail)
    {
        log_error_ctx(
            "plugins.runtime.filesystem.common",
            "emit_permission_log_api",
            &format!("failed to emit permission log: {}", e),
        );
    }
}

pub(super) fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if dst.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            fs_t1("plugins.runtime.filesystem.destination_exists", dst.display().to_string()),
        ));
    }
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let metadata = fs::symlink_metadata(&src_path)?;

        if metadata.file_type().is_symlink() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                fs_t1(
                    "plugins.runtime.filesystem.copy_symlink_forbidden",
                    src_path.display().to_string(),
                ),
            ));
        }

        if metadata.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
