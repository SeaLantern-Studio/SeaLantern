use crate::plugins::runtime::shared::validate_path_static;
use crate::utils::logger::log_error;
use mlua::{Lua, Table};
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

pub(super) fn create_fs_table(lua: &Lua) -> Result<Table, String> {
    lua.create_table()
        .map_err(|e| format!("Failed to create fs table: {}", e))
}

pub(super) fn set_fs_table(sl: &Table, fs_table: Table) -> Result<(), String> {
    sl.set("fs", fs_table)
        .map_err(|e| format!("Failed to set sl.fs: {}", e))
}

pub(super) fn set_fs_function(
    fs_table: &Table,
    name: &str,
    function: mlua::Function,
) -> Result<(), String> {
    fs_table
        .set(name, function)
        .map_err(|e| format!("Failed to set fs.{}: {}", name, e))
}

pub(crate) fn has_any_fs_permission(perms: &[String]) -> bool {
    perms.iter().any(|p| {
        matches!(
            p.as_str(),
            "fs.data"
                | "fs.server"
                | "fs.global"
                | "fs.data.read"
                | "fs.data.write"
                | "fs.data.list"
                | "fs.data.meta"
                | "fs.data.delete"
                | "fs.data.transfer"
                | "fs.server.read"
                | "fs.server.write"
                | "fs.server.list"
                | "fs.server.meta"
                | "fs.server.delete"
                | "fs.server.transfer"
                | "fs.global.read"
                | "fs.global.write"
                | "fs.global.list"
                | "fs.global.meta"
                | "fs.global.delete"
                | "fs.global.transfer"
        )
    })
}

fn resolve_scope_permission(scope: &str) -> Result<&'static str, mlua::Error> {
    match scope {
        "data" => Ok("fs.data"),
        "server" => Ok("fs.server"),
        "global" => Ok("fs.global"),
        _ => Err(mlua::Error::runtime("Invalid scope. Must be 'data', 'server', or 'global'")),
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
        return Err(mlua::Error::runtime(format!(
            "Permission denied: '{}' or '{}' permission is required for this operation",
            scope_permission, action_permission
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
            return Err(mlua::Error::runtime(format!(
                "Symlinks and reparse points are not allowed in filesystem sandbox: {}",
                path.display()
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
        return Err(mlua::Error::runtime("Refusing to remove filesystem sandbox root".to_string()));
    }

    Ok(())
}

pub(super) fn emit_permission_log_api(plugin_id: &str, api_name: &str, detail: &str) {
    if let Err(e) =
        crate::plugins::api::emit_permission_log(plugin_id, "api_call", api_name, detail)
    {
        log_error(&format!("Failed to emit permission log: {}", e));
    }
}

pub(super) fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if dst.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Destination already exists: {}", dst.display()),
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
                format!("Symlinks and reparse points are not allowed: {}", src_path.display()),
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
