use super::helpers::validate_path_static;
use super::PluginRuntime;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use mlua::Table;
use std::fs;

fn check_fs_permission(perms: &[String], required_perm: &str) -> Result<(), mlua::Error> {
    if !perms.iter().any(|p| p == required_perm) {
        return Err(mlua::Error::runtime(format!(
            "Permission denied: '{}' permission is required for this operation",
            required_perm
        )));
    }
    Ok(())
}

fn get_base_dir_for_permission(
    data_dir: &std::path::Path,
    server_dir: &std::path::Path,
    global_dir: &std::path::Path,
    perms: &[String],
) -> Result<(std::path::PathBuf, String), mlua::Error> {
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

fn validate_fs_path(
    base_dir: &std::path::Path,
    path: &str,
) -> Result<std::path::PathBuf, mlua::Error> {
    validate_path_static(base_dir, path)
}

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

impl PluginRuntime {
    pub(super) fn setup_fs_namespace(&self, sl: &Table) -> Result<(), String> {
        use crate::plugins::api::emit_permission_log;

        let fs_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create fs table: {}", e))?;

        let data_dir = self.data_dir.clone();
        let server_dir = self.server_dir.clone();
        let global_dir = self.global_dir.clone();
        let plugin_id = self.plugin_id.clone();
        let permissions = self.permissions.clone();

        let data_dir_fn = data_dir.clone();
        let server_dir_fn = server_dir.clone();
        let global_dir_fn = global_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let read_fn = self
            .lua
            .create_function(move |_, path: String| {
                let (base_dir, perm) = get_base_dir_for_permission(
                    &data_dir_fn,
                    &server_dir_fn,
                    &global_dir_fn,
                    &perms,
                )?;
                check_fs_permission(&perms, &perm)?;
                let full_path = validate_fs_path(&base_dir, &path)?;

                if let Err(e) = emit_permission_log(&pid, "api_call", "sl.fs.read", &path) {
                    eprintln!("Failed to emit permission log: {}", e);
                }

                let metadata = fs::metadata(&full_path).map_err(|e| {
                    mlua::Error::runtime(format!("Failed to get file metadata: {}", e))
                })?;
                if metadata.len() > MAX_FILE_SIZE {
                    return Err(mlua::Error::runtime("File too large (max 10MB)"));
                }
                fs::read_to_string(&full_path)
                    .map_err(|e| mlua::Error::runtime(format!("Failed to read file: {}", e)))
            })
            .map_err(|e| format!("Failed to create fs.read: {}", e))?;
        fs_table
            .set("read", read_fn)
            .map_err(|e| format!("Failed to set fs.read: {}", e))?;

        let data_dir_fn = data_dir.clone();
        let server_dir_fn = server_dir.clone();
        let global_dir_fn = global_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let read_binary_fn = self
            .lua
            .create_function(move |_, path: String| {
                let (base_dir, perm) = get_base_dir_for_permission(
                    &data_dir_fn,
                    &server_dir_fn,
                    &global_dir_fn,
                    &perms,
                )?;
                check_fs_permission(&perms, &perm)?;
                let full_path = validate_fs_path(&base_dir, &path)?;

                if let Err(e) = emit_permission_log(&pid, "api_call", "sl.fs.read_binary", &path) {
                    eprintln!("Failed to emit permission log: {}", e);
                }

                let metadata = fs::metadata(&full_path).map_err(|e| {
                    mlua::Error::runtime(format!("Failed to get file metadata: {}", e))
                })?;
                if metadata.len() > MAX_FILE_SIZE {
                    return Err(mlua::Error::runtime("File too large (max 10MB)"));
                }
                let bytes = fs::read(&full_path)
                    .map_err(|e| mlua::Error::runtime(format!("Failed to read file: {}", e)))?;
                Ok(BASE64.encode(&bytes))
            })
            .map_err(|e| format!("Failed to create fs.read_binary: {}", e))?;
        fs_table
            .set("read_binary", read_binary_fn)
            .map_err(|e| format!("Failed to set fs.read_binary: {}", e))?;

        let data_dir_fn = data_dir.clone();
        let server_dir_fn = server_dir.clone();
        let global_dir_fn = global_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let write_fn = self
            .lua
            .create_function(move |_, (path, content): (String, String)| {
                let (base_dir, perm) = get_base_dir_for_permission(
                    &data_dir_fn,
                    &server_dir_fn,
                    &global_dir_fn,
                    &perms,
                )?;
                check_fs_permission(&perms, &perm)?;

                if content.len() > 10 * 1024 * 1024 {
                    return Err(mlua::Error::runtime("Content too large (max 10MB)"));
                }
                let full_path = validate_fs_path(&base_dir, &path)?;

                if let Err(e) = emit_permission_log(&pid, "api_call", "sl.fs.write", &path) {
                    eprintln!("Failed to emit permission log: {}", e);
                }

                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        mlua::Error::runtime(format!("Failed to create directory: {}", e))
                    })?;
                }
                fs::write(&full_path, content)
                    .map_err(|e| mlua::Error::runtime(format!("Failed to write file: {}", e)))
            })
            .map_err(|e| format!("Failed to create fs.write: {}", e))?;
        fs_table
            .set("write", write_fn)
            .map_err(|e| format!("Failed to set fs.write: {}", e))?;

        let data_dir_fn = data_dir.clone();
        let server_dir_fn = server_dir.clone();
        let global_dir_fn = global_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let exists_fn = self
            .lua
            .create_function(move |_, path: String| {
                let (base_dir, perm) = get_base_dir_for_permission(
                    &data_dir_fn,
                    &server_dir_fn,
                    &global_dir_fn,
                    &perms,
                )?;
                check_fs_permission(&perms, &perm)?;
                let full_path = validate_fs_path(&base_dir, &path)?;

                if let Err(e) = emit_permission_log(&pid, "api_call", "sl.fs.exists", &path) {
                    eprintln!("Failed to emit permission log: {}", e);
                }
                Ok(full_path.exists())
            })
            .map_err(|e| format!("Failed to create fs.exists: {}", e))?;
        fs_table
            .set("exists", exists_fn)
            .map_err(|e| format!("Failed to set fs.exists: {}", e))?;

        let data_dir_fn = data_dir.clone();
        let server_dir_fn = server_dir.clone();
        let global_dir_fn = global_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let list_fn = self
            .lua
            .create_function(move |lua, path: String| {
                let (base_dir, perm) = get_base_dir_for_permission(
                    &data_dir_fn,
                    &server_dir_fn,
                    &global_dir_fn,
                    &perms,
                )?;
                check_fs_permission(&perms, &perm)?;
                let full_path = validate_fs_path(&base_dir, &path)?;

                if let Err(e) = emit_permission_log(&pid, "api_call", "sl.fs.list", &path) {
                    eprintln!("Failed to emit permission log: {}", e);
                }
                let entries = fs::read_dir(&full_path).map_err(|e| {
                    mlua::Error::runtime(format!("Failed to read directory: {}", e))
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
            .map_err(|e| format!("Failed to create fs.list: {}", e))?;
        fs_table
            .set("list", list_fn)
            .map_err(|e| format!("Failed to set fs.list: {}", e))?;

        let data_dir_fn = data_dir.clone();
        let server_dir_fn = server_dir.clone();
        let global_dir_fn = global_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let mkdir_fn = self
            .lua
            .create_function(move |_, path: String| {
                let (base_dir, perm) = get_base_dir_for_permission(
                    &data_dir_fn,
                    &server_dir_fn,
                    &global_dir_fn,
                    &perms,
                )?;
                check_fs_permission(&perms, &perm)?;
                let full_path = validate_fs_path(&base_dir, &path)?;

                if let Err(e) = emit_permission_log(&pid, "api_call", "sl.fs.mkdir", &path) {
                    eprintln!("Failed to emit permission log: {}", e);
                }
                fs::create_dir_all(&full_path)
                    .map_err(|e| mlua::Error::runtime(format!("Failed to create directory: {}", e)))
            })
            .map_err(|e| format!("Failed to create fs.mkdir: {}", e))?;
        fs_table
            .set("mkdir", mkdir_fn)
            .map_err(|e| format!("Failed to set fs.mkdir: {}", e))?;

        let data_dir_fn = data_dir.clone();
        let server_dir_fn = server_dir.clone();
        let global_dir_fn = global_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let remove_fn = self
            .lua
            .create_function(move |_, path: String| {
                let (base_dir, perm) = get_base_dir_for_permission(
                    &data_dir_fn,
                    &server_dir_fn,
                    &global_dir_fn,
                    &perms,
                )?;
                check_fs_permission(&perms, &perm)?;
                let full_path = validate_fs_path(&base_dir, &path)?;

                if let Err(e) = emit_permission_log(&pid, "api_call", "sl.fs.remove", &path) {
                    eprintln!("Failed to emit permission log: {}", e);
                }
                if full_path.is_dir() {
                    fs::remove_dir_all(&full_path).map_err(|e| {
                        mlua::Error::runtime(format!("Failed to remove directory: {}", e))
                    })
                } else {
                    fs::remove_file(&full_path)
                        .map_err(|e| mlua::Error::runtime(format!("Failed to remove file: {}", e)))
                }
            })
            .map_err(|e| format!("Failed to create fs.remove: {}", e))?;
        fs_table
            .set("remove", remove_fn)
            .map_err(|e| format!("Failed to set fs.remove: {}", e))?;

        let data_dir_fn = data_dir.clone();
        let server_dir_fn = server_dir.clone();
        let global_dir_fn = global_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let info_fn = self
            .lua
            .create_function(move |lua, path: String| {
                let (base_dir, perm) = get_base_dir_for_permission(
                    &data_dir_fn,
                    &server_dir_fn,
                    &global_dir_fn,
                    &perms,
                )?;
                check_fs_permission(&perms, &perm)?;
                let full_path = validate_fs_path(&base_dir, &path)?;

                if let Err(e) = emit_permission_log(&pid, "api_call", "sl.fs.info", &path) {
                    eprintln!("Failed to emit permission log: {}", e);
                }
                let metadata = fs::metadata(&full_path).map_err(|e| {
                    mlua::Error::runtime(format!("Failed to get file metadata: {}", e))
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
            .map_err(|e| format!("Failed to create fs.info: {}", e))?;
        fs_table
            .set("info", info_fn)
            .map_err(|e| format!("Failed to set fs.info: {}", e))?;

        let data_dir_fn = data_dir.clone();
        let server_dir_fn = server_dir.clone();
        let global_dir_fn = global_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let copy_fn = self
            .lua
            .create_function(move |_, (src, dst): (String, String)| {
                let (base_dir, perm) = get_base_dir_for_permission(
                    &data_dir_fn,
                    &server_dir_fn,
                    &global_dir_fn,
                    &perms,
                )?;
                check_fs_permission(&perms, &perm)?;
                let src_path = validate_fs_path(&base_dir, &src)?;
                let dst_path = validate_fs_path(&base_dir, &dst)?;

                if let Err(e) = emit_permission_log(&pid, "api_call", "sl.fs.copy", &src) {
                    eprintln!("Failed to emit permission log: {}", e);
                }

                if src_path.is_dir() {
                    copy_dir_recursive(&src_path, &dst_path).map_err(|e| {
                        mlua::Error::runtime(format!("Failed to copy directory: {}", e))
                    })
                } else {
                    fs::copy(&src_path, &dst_path)
                        .map_err(|e| mlua::Error::runtime(format!("Failed to copy file: {}", e)))
                        .map(|_| ())
                }
            })
            .map_err(|e| format!("Failed to create fs.copy: {}", e))?;
        fs_table
            .set("copy", copy_fn)
            .map_err(|e| format!("Failed to set fs.copy: {}", e))?;

        let data_dir_fn = data_dir.clone();
        let server_dir_fn = server_dir.clone();
        let global_dir_fn = global_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let move_fn = self
            .lua
            .create_function(move |_, (src, dst): (String, String)| {
                let (base_dir, perm) = get_base_dir_for_permission(
                    &data_dir_fn,
                    &server_dir_fn,
                    &global_dir_fn,
                    &perms,
                )?;
                check_fs_permission(&perms, &perm)?;
                let src_path = validate_fs_path(&base_dir, &src)?;
                let dst_path = validate_fs_path(&base_dir, &dst)?;

                if let Err(e) = emit_permission_log(&pid, "api_call", "sl.fs.move", &src) {
                    eprintln!("Failed to emit permission log: {}", e);
                }

                fs::rename(&src_path, &dst_path).map_err(|e| {
                    mlua::Error::runtime(format!("Failed to move file/directory: {}", e))
                })
            })
            .map_err(|e| format!("Failed to create fs.move: {}", e))?;
        fs_table
            .set("move", move_fn)
            .map_err(|e| format!("Failed to set fs.move: {}", e))?;

        let data_dir_fn = data_dir.clone();
        let server_dir_fn = server_dir.clone();
        let global_dir_fn = global_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let rename_fn = self
            .lua
            .create_function(move |_, (old_path, new_path): (String, String)| {
                let (base_dir, perm) = get_base_dir_for_permission(
                    &data_dir_fn,
                    &server_dir_fn,
                    &global_dir_fn,
                    &perms,
                )?;
                check_fs_permission(&perms, &perm)?;
                let old_full_path = validate_fs_path(&base_dir, &old_path)?;
                let new_full_path = validate_fs_path(&base_dir, &new_path)?;

                if let Err(e) = emit_permission_log(&pid, "api_call", "sl.fs.rename", &old_path) {
                    eprintln!("Failed to emit permission log: {}", e);
                }

                fs::rename(&old_full_path, &new_full_path).map_err(|e| {
                    mlua::Error::runtime(format!("Failed to rename file/directory: {}", e))
                })
            })
            .map_err(|e| format!("Failed to create fs.rename: {}", e))?;
        fs_table
            .set("rename", rename_fn)
            .map_err(|e| format!("Failed to set fs.rename: {}", e))?;

        let data_dir_get = data_dir.clone();
        let server_dir_get = server_dir.clone();
        let global_dir_get = global_dir.clone();
        let pid_get = plugin_id.clone();
        let perms_get = permissions.clone();
        let get_path_fn = self
            .lua
            .create_function(move |_, scope: String| {
                let path = match scope.as_str() {
                    "data" => {
                        check_fs_permission(&perms_get, "fs.data")?;
                        data_dir_get.to_string_lossy().to_string()
                    }
                    "server" => {
                        check_fs_permission(&perms_get, "fs.server")?;
                        server_dir_get.to_string_lossy().to_string()
                    }
                    "global" => {
                        check_fs_permission(&perms_get, "fs.global")?;
                        global_dir_get.to_string_lossy().to_string()
                    }
                    _ => {
                        return Err(mlua::Error::runtime(
                            "Invalid scope. Must be 'data', 'server', or 'global'",
                        ))
                    }
                };

                if let Err(e) = emit_permission_log(&pid_get, "api_call", "sl.fs.get_path", &scope)
                {
                    eprintln!("Failed to emit permission log: {}", e);
                }

                Ok(path)
            })
            .map_err(|e| format!("Failed to create fs.get_path: {}", e))?;
        fs_table
            .set("get_path", get_path_fn)
            .map_err(|e| format!("Failed to set fs.get_path: {}", e))?;

        sl.set("fs", fs_table)
            .map_err(|e| format!("Failed to set sl.fs: {}", e))?;

        Ok(())
    }
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
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
