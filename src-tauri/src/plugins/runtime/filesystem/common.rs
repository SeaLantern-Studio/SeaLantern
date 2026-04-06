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

pub(super) fn check_fs_permission(
    perms: &[String],
    required_perm: &str,
) -> Result<(), mlua::Error> {
    if !perms.iter().any(|p| p == required_perm) {
        return Err(mlua::Error::runtime(format!(
            "Permission denied: '{}' permission is required for this operation",
            required_perm
        )));
    }
    Ok(())
}

pub(super) fn get_base_dir_for_permission(
    data_dir: &Path,
    server_dir: &Path,
    global_dir: &Path,
    perms: &[String],
) -> Result<(PathBuf, String), mlua::Error> {
    if perms.iter().any(|p| p == "fs.global") {
        Ok((global_dir.to_path_buf(), "fs.global".to_string()))
    } else if perms.iter().any(|p| p == "fs.server") {
        Ok((server_dir.to_path_buf(), "fs.server".to_string()))
    } else if perms.iter().any(|p| p == "fs.data") {
        Ok((data_dir.to_path_buf(), "fs.data".to_string()))
    } else {
        Err(mlua::Error::runtime(
            "Permission denied: 'fs.data', 'fs.server', or 'fs.global' permission is required",
        ))
    }
}

pub(super) fn validate_fs_path(base_dir: &Path, path: &str) -> Result<PathBuf, mlua::Error> {
    validate_path_static(base_dir, path)
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
        fs::remove_dir_all(dst)?;
    }
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
