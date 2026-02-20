use super::helpers::validate_server_path;
use super::PluginRuntime;
use mlua::Table;
use std::fs;
use std::path::PathBuf;

impl PluginRuntime {
    pub(super) fn setup_server_namespace(&self, sl: &Table) -> Result<(), String> {
        use crate::services::global::server_manager;

        let server_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create server table: {}", e))?;

        const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
        let permissions = self.permissions.clone();

        let perms = permissions.clone();
        let list_fn = self
            .lua
            .create_function(move |lua, ()| {
                if !perms.iter().any(|p| p == "server") {
                    return Err(mlua::Error::runtime(
                        "Permission denied: 'server' permission required",
                    ));
                }
                let servers = server_manager().get_server_list();
                let result = lua.create_table()?;
                for (i, server) in servers.iter().enumerate() {
                    let entry = lua.create_table()?;
                    entry.set("id", server.id.clone())?;
                    entry.set("name", server.name.clone())?;
                    entry.set("path", server.path.clone())?;
                    entry.set("version", server.mc_version.clone())?;
                    entry.set("server_type", server.core_type.clone())?;
                    result.set(i + 1, entry)?;
                }
                Ok(result)
            })
            .map_err(|e| format!("Failed to create server.list: {}", e))?;
        server_table
            .set("list", list_fn)
            .map_err(|e| format!("Failed to set server.list: {}", e))?;

        let perms = permissions.clone();
        let get_path_fn =
            self.lua
                .create_function(move |_, server_id: String| {
                    if !perms.iter().any(|p| p == "server") {
                        return Err(mlua::Error::runtime(
                            "Permission denied: 'server' permission required",
                        ));
                    }
                    let servers = server_manager().get_server_list();
                    let server = servers.iter().find(|s| s.id == server_id).ok_or_else(|| {
                        mlua::Error::runtime(format!("服务器不存在: {}", server_id))
                    })?;
                    Ok(server.path.clone())
                })
                .map_err(|e| format!("Failed to create server.get_path: {}", e))?;
        server_table
            .set("get_path", get_path_fn)
            .map_err(|e| format!("Failed to set server.get_path: {}", e))?;

        let perms = permissions.clone();
        let read_file_fn =
            self.lua
                .create_function(move |_, (server_id, relative_path): (String, String)| {
                    if !perms.iter().any(|p| p == "server") {
                        return Err(mlua::Error::runtime(
                            "Permission denied: 'server' permission required",
                        ));
                    }
                    let servers = server_manager().get_server_list();
                    let server = servers.iter().find(|s| s.id == server_id).ok_or_else(|| {
                        mlua::Error::runtime(format!("服务器不存在: {}", server_id))
                    })?;

                    let server_dir = PathBuf::from(&server.path);
                    let full_path = validate_server_path(&server_dir, &relative_path)?;

                    let metadata = fs::metadata(&full_path)
                        .map_err(|e| mlua::Error::runtime(format!("无法获取文件信息: {}", e)))?;
                    if metadata.len() > MAX_FILE_SIZE {
                        return Err(mlua::Error::runtime("文件过大 (最大 10MB)"));
                    }

                    fs::read_to_string(&full_path)
                        .map_err(|e| mlua::Error::runtime(format!("读取文件失败: {}", e)))
                })
                .map_err(|e| format!("Failed to create server.read_file: {}", e))?;
        server_table
            .set("read_file", read_file_fn)
            .map_err(|e| format!("Failed to set server.read_file: {}", e))?;

        let perms = permissions.clone();
        let write_file_fn = self
            .lua
            .create_function(
                move |_, (server_id, relative_path, content): (String, String, String)| {
                    if !perms.iter().any(|p| p == "server") {
                        return Err(mlua::Error::runtime(
                            "Permission denied: 'server' permission required",
                        ));
                    }
                    let servers = server_manager().get_server_list();
                    let server = servers.iter().find(|s| s.id == server_id).ok_or_else(|| {
                        mlua::Error::runtime(format!("服务器不存在: {}", server_id))
                    })?;

                    let server_dir = PathBuf::from(&server.path);
                    let full_path = validate_server_path(&server_dir, &relative_path)?;

                    if let Some(parent) = full_path.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|e| mlua::Error::runtime(format!("创建目录失败: {}", e)))?;
                    }

                    fs::write(&full_path, content)
                        .map_err(|e| mlua::Error::runtime(format!("写入文件失败: {}", e)))?;
                    Ok(true)
                },
            )
            .map_err(|e| format!("Failed to create server.write_file: {}", e))?;
        server_table
            .set("write_file", write_file_fn)
            .map_err(|e| format!("Failed to set server.write_file: {}", e))?;

        let perms = permissions.clone();
        let list_dir_fn =
            self.lua
                .create_function(move |lua, (server_id, relative_path): (String, String)| {
                    if !perms.iter().any(|p| p == "server") {
                        return Err(mlua::Error::runtime(
                            "Permission denied: 'server' permission required",
                        ));
                    }
                    let servers = server_manager().get_server_list();
                    let server = servers.iter().find(|s| s.id == server_id).ok_or_else(|| {
                        mlua::Error::runtime(format!("服务器不存在: {}", server_id))
                    })?;

                    let server_dir = PathBuf::from(&server.path);
                    let full_path = validate_server_path(&server_dir, &relative_path)?;

                    if !full_path.is_dir() {
                        return Err(mlua::Error::runtime("路径不是目录"));
                    }

                    let entries = fs::read_dir(&full_path)
                        .map_err(|e| mlua::Error::runtime(format!("读取目录失败: {}", e)))?;

                    let result = lua.create_table()?;
                    let mut i = 1;
                    for entry in entries {
                        let entry = entry
                            .map_err(|e| mlua::Error::runtime(format!("读取目录项失败: {}", e)))?;
                        let metadata = entry.metadata().map_err(|e| {
                            mlua::Error::runtime(format!("获取文件信息失败: {}", e))
                        })?;

                        let item = lua.create_table()?;
                        item.set("name", entry.file_name().to_string_lossy().to_string())?;
                        item.set("is_dir", metadata.is_dir())?;
                        item.set("size", metadata.len())?;
                        result.set(i, item)?;
                        i += 1;
                    }
                    Ok(result)
                })
                .map_err(|e| format!("Failed to create server.list_dir: {}", e))?;
        server_table
            .set("list_dir", list_dir_fn)
            .map_err(|e| format!("Failed to set server.list_dir: {}", e))?;

        let perms = permissions.clone();
        let exists_fn =
            self.lua
                .create_function(move |_, (server_id, relative_path): (String, String)| {
                    if !perms.iter().any(|p| p == "server") {
                        return Err(mlua::Error::runtime(
                            "Permission denied: 'server' permission required",
                        ));
                    }
                    let servers = server_manager().get_server_list();
                    let server = servers.iter().find(|s| s.id == server_id).ok_or_else(|| {
                        mlua::Error::runtime(format!("服务器不存在: {}", server_id))
                    })?;

                    let server_dir = PathBuf::from(&server.path);
                    let full_path = validate_server_path(&server_dir, &relative_path)?;

                    Ok(full_path.exists())
                })
                .map_err(|e| format!("Failed to create server.exists: {}", e))?;
        server_table
            .set("exists", exists_fn)
            .map_err(|e| format!("Failed to set server.exists: {}", e))?;

        let perms = permissions.clone();
        let logs_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create server.logs table: {}", e))?;

        let get_logs_fn = self
            .lua
            .create_function(move |lua, (server_id, count): (String, Option<usize>)| {
                if !perms.iter().any(|p| p == "server") {
                    return Err(mlua::Error::runtime(
                        "Permission denied: 'server' permission required",
                    ));
                }

                let servers = server_manager().get_server_list();
                if !servers.iter().any(|s| s.id == server_id) {
                    return Err(mlua::Error::runtime(format!("服务器不存在: {}", server_id)));
                }

                let count = count.unwrap_or(100).min(1000);
                let all_logs = server_manager().get_logs(&server_id, 0);

                let start = if all_logs.len() > count {
                    all_logs.len() - count
                } else {
                    0
                };
                let logs = &all_logs[start..];

                let result = lua.create_table()?;
                for (i, line) in logs.iter().enumerate() {
                    result.set(i + 1, line.clone())?;
                }
                Ok(result)
            })
            .map_err(|e| format!("Failed to create server.logs.get: {}", e))?;
        logs_table
            .set("get", get_logs_fn)
            .map_err(|e| format!("Failed to set server.logs.get: {}", e))?;

        let perms = permissions.clone();
        let get_all_logs_fn = self
            .lua
            .create_function(move |lua, count: Option<usize>| {
                if !perms.iter().any(|p| p == "server") {
                    return Err(mlua::Error::runtime(
                        "Permission denied: 'server' permission required",
                    ));
                }

                let count = count.unwrap_or(100).min(1000);

                let running_ids = server_manager().get_running_server_ids();
                let logs_map = server_manager().get_all_logs();

                let result = lua.create_table()?;
                let mut i = 1;
                for (server_id, logs) in logs_map {
                    if !running_ids.contains(&server_id) {
                        continue;
                    }

                    let start = if logs.len() > count {
                        logs.len() - count
                    } else {
                        0
                    };
                    let slice = &logs[start..];

                    let entry = lua.create_table()?;
                    entry.set("server_id", server_id)?;

                    let lines_table = lua.create_table()?;
                    for (j, line) in slice.iter().enumerate() {
                        lines_table.set(j + 1, line.clone())?;
                    }
                    entry.set("logs", lines_table)?;

                    result.set(i, entry)?;
                    i += 1;
                }

                Ok(result)
            })
            .map_err(|e| format!("Failed to create server.logs.getAll: {}", e))?;
        logs_table
            .set("getAll", get_all_logs_fn)
            .map_err(|e| format!("Failed to set server.logs.getAll: {}", e))?;

        server_table
            .set("logs", logs_table)
            .map_err(|e| format!("Failed to set server.logs: {}", e))?;

        sl.set("server", server_table)
            .map_err(|e| format!("Failed to set sl.server: {}", e))?;

        Ok(())
    }
}
