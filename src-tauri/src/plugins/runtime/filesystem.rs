use super::helpers::validate_path_static;
use super::PluginRuntime;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use mlua::Table;
use std::fs;

impl PluginRuntime {
    pub(super) fn setup_fs_namespace(&self, sl: &Table) -> Result<(), String> {
        use crate::plugins::api::emit_permission_log;

        let fs_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create fs table: {}", e))?;

        let data_dir = self.data_dir.clone();
        let plugin_id = self.plugin_id.clone();
        let permissions = self.permissions.clone();

        const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

        let dir = data_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let read_fn = self
            .lua
            .create_function(move |_, path: String| {
                if !perms.iter().any(|p| p == "fs") {
                    return Err(mlua::Error::runtime(
                        "Permission denied: 'fs' permission required",
                    ));
                }
                let full_path = validate_path_static(&dir, &path)?;

                let _ = emit_permission_log(&pid, "api_call", "sl.fs.read", &path);

                let metadata = fs::metadata(&full_path).map_err(|e| {
                    mlua::Error::runtime(format!("Failed to get file metadata: {}", e))
                })?;
                if metadata.len() > MAX_FILE_SIZE {
                    return Err(mlua::Error::runtime("File too large (max 10MB)"));
                }
                fs::read_to_string(&full_path)
                    .map_err(|e| mlua::Error::runtime(format!("读取文件失败: {}", e)))
            })
            .map_err(|e| format!("Failed to create fs.read: {}", e))?;
        fs_table
            .set("read", read_fn)
            .map_err(|e| format!("Failed to set fs.read: {}", e))?;

        let dir = data_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let read_binary_fn = self
            .lua
            .create_function(move |_, path: String| {
                if !perms.iter().any(|p| p == "fs") {
                    return Err(mlua::Error::runtime(
                        "Permission denied: 'fs' permission required",
                    ));
                }
                let full_path = validate_path_static(&dir, &path)?;

                let _ = emit_permission_log(&pid, "api_call", "sl.fs.read_binary", &path);

                let metadata = fs::metadata(&full_path).map_err(|e| {
                    mlua::Error::runtime(format!("Failed to get file metadata: {}", e))
                })?;
                if metadata.len() > MAX_FILE_SIZE {
                    return Err(mlua::Error::runtime("File too large (max 10MB)"));
                }
                let bytes = fs::read(&full_path)
                    .map_err(|e| mlua::Error::runtime(format!("读取文件失败: {}", e)))?;
                Ok(BASE64.encode(&bytes))
            })
            .map_err(|e| format!("Failed to create fs.read_binary: {}", e))?;
        fs_table
            .set("read_binary", read_binary_fn)
            .map_err(|e| format!("Failed to set fs.read_binary: {}", e))?;

        let dir = data_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let write_fn = self
            .lua
            .create_function(move |_, (path, content): (String, String)| {
                if !perms.iter().any(|p| p == "fs") {
                    return Err(mlua::Error::runtime(
                        "Permission denied: 'fs' permission required",
                    ));
                }

                if content.len() > 10 * 1024 * 1024 {
                    return Err(mlua::Error::runtime("Content too large (max 10MB)"));
                }
                let full_path = validate_path_static(&dir, &path)?;

                let _ = emit_permission_log(&pid, "api_call", "sl.fs.write", &path);

                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| mlua::Error::runtime(format!("创建目录失败: {}", e)))?;
                }
                fs::write(&full_path, content)
                    .map_err(|e| mlua::Error::runtime(format!("写入文件失败: {}", e)))
            })
            .map_err(|e| format!("Failed to create fs.write: {}", e))?;
        fs_table
            .set("write", write_fn)
            .map_err(|e| format!("Failed to set fs.write: {}", e))?;

        let dir = data_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let exists_fn = self
            .lua
            .create_function(move |_, path: String| {
                if !perms.iter().any(|p| p == "fs") {
                    return Err(mlua::Error::runtime(
                        "Permission denied: 'fs' permission required",
                    ));
                }
                let full_path = validate_path_static(&dir, &path)?;

                let _ = emit_permission_log(&pid, "api_call", "sl.fs.exists", &path);
                Ok(full_path.exists())
            })
            .map_err(|e| format!("Failed to create fs.exists: {}", e))?;
        fs_table
            .set("exists", exists_fn)
            .map_err(|e| format!("Failed to set fs.exists: {}", e))?;

        let dir = data_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let list_fn = self
            .lua
            .create_function(move |lua, path: String| {
                if !perms.iter().any(|p| p == "fs") {
                    return Err(mlua::Error::runtime(
                        "Permission denied: 'fs' permission required",
                    ));
                }
                let full_path = validate_path_static(&dir, &path)?;

                let _ = emit_permission_log(&pid, "api_call", "sl.fs.list", &path);
                let entries = fs::read_dir(&full_path)
                    .map_err(|e| mlua::Error::runtime(format!("读取目录失败: {}", e)))?;

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

        let dir = data_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let mkdir_fn = self
            .lua
            .create_function(move |_, path: String| {
                if !perms.iter().any(|p| p == "fs") {
                    return Err(mlua::Error::runtime(
                        "Permission denied: 'fs' permission required",
                    ));
                }
                let full_path = validate_path_static(&dir, &path)?;

                let _ = emit_permission_log(&pid, "api_call", "sl.fs.mkdir", &path);
                fs::create_dir_all(&full_path)
                    .map_err(|e| mlua::Error::runtime(format!("创建目录失败: {}", e)))
            })
            .map_err(|e| format!("Failed to create fs.mkdir: {}", e))?;
        fs_table
            .set("mkdir", mkdir_fn)
            .map_err(|e| format!("Failed to set fs.mkdir: {}", e))?;

        let dir = data_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let remove_fn = self
            .lua
            .create_function(move |_, path: String| {
                if !perms.iter().any(|p| p == "fs") {
                    return Err(mlua::Error::runtime(
                        "Permission denied: 'fs' permission required",
                    ));
                }
                let full_path = validate_path_static(&dir, &path)?;

                let _ = emit_permission_log(&pid, "api_call", "sl.fs.remove", &path);
                if full_path.is_dir() {
                    fs::remove_dir_all(&full_path)
                        .map_err(|e| mlua::Error::runtime(format!("删除目录失败: {}", e)))
                } else {
                    fs::remove_file(&full_path)
                        .map_err(|e| mlua::Error::runtime(format!("删除文件失败: {}", e)))
                }
            })
            .map_err(|e| format!("Failed to create fs.remove: {}", e))?;
        fs_table
            .set("remove", remove_fn)
            .map_err(|e| format!("Failed to set fs.remove: {}", e))?;

        let dir = data_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let info_fn = self
            .lua
            .create_function(move |lua, path: String| {
                if !perms.iter().any(|p| p == "fs") {
                    return Err(mlua::Error::runtime(
                        "Permission denied: 'fs' permission required",
                    ));
                }
                let full_path = validate_path_static(&dir, &path)?;

                let _ = emit_permission_log(&pid, "api_call", "sl.fs.info", &path);
                let metadata = fs::metadata(&full_path)
                    .map_err(|e| mlua::Error::runtime(format!("获取文件信息失败: {}", e)))?;

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

        sl.set("fs", fs_table)
            .map_err(|e| format!("Failed to set sl.fs: {}", e))?;

        Ok(())
    }
}
