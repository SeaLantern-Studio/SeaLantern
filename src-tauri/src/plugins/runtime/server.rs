use super::helpers::validate_server_path;
use super::PluginRuntime;
use crate::services::global::i18n_service;
use mlua::Table;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

impl PluginRuntime {
    pub(super) fn setup_server_namespace(&self, sl: &Table) -> Result<(), String> {
        use crate::services::global::server_manager;

        let server_table = self
            .lua
            .create_table()
            .map_err(|e| format!("{}: {}", i18n_service().t("server.create_table_failed"), e))?;

        const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
        let permissions = self.permissions.clone();

        let perms = permissions.clone();
        let list_fn = self
            .lua
            .create_function(move |lua, ()| {
                if !perms.iter().any(|p| p == "server") {
                    return Err(mlua::Error::runtime(i18n_service().t("server.permission_denied")));
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
            .map_err(|e| format!("{}: {}", i18n_service().t("server.create_list_failed"), e))?;
        server_table
            .set("list", list_fn)
            .map_err(|e| format!("{}: {}", i18n_service().t("server.set_list_failed"), e))?;

        let perms = permissions.clone();
        let get_path_fn = self
            .lua
            .create_function(move |_, server_id: String| {
                if !perms.iter().any(|p| p == "server") {
                    return Err(mlua::Error::runtime(i18n_service().t("server.permission_denied")));
                }
                let servers = server_manager().get_server_list();
                let server = servers.iter().find(|s| s.id == server_id).ok_or_else(|| {
                    mlua::Error::runtime(i18n_service().t_with_options(
                        "server.server_not_found",
                        &crate::plugins::runtime::console::i18n_arg("0", &server_id),
                    ))
                })?;
                Ok(server.path.clone())
            })
            .map_err(|e| format!("{}: {}", i18n_service().t("server.create_get_path_failed"), e))?;
        server_table
            .set("get_path", get_path_fn)
            .map_err(|e| format!("{}: {}", i18n_service().t("server.set_get_path_failed"), e))?;

        let perms = permissions.clone();
        let read_file_fn = self
            .lua
            .create_function(move |_, (server_id, relative_path): (String, String)| {
                if !perms.iter().any(|p| p == "server") {
                    return Err(mlua::Error::runtime(i18n_service().t("server.permission_denied")));
                }
                let servers = server_manager().get_server_list();
                let server = servers.iter().find(|s| s.id == server_id).ok_or_else(|| {
                    mlua::Error::runtime(i18n_service().t_with_options(
                        "server.server_not_found",
                        &crate::plugins::runtime::console::i18n_arg("0", &server_id),
                    ))
                })?;

                let server_dir = PathBuf::from(&server.path);
                let full_path = validate_server_path(&server_dir, &relative_path)?;

                let metadata = fs::metadata(&full_path).map_err(|e| {
                    mlua::Error::runtime(i18n_service().t_with_options(
                        "server.failed_to_get_metadata",
                        &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                    ))
                })?;
                if metadata.len() > MAX_FILE_SIZE {
                    return Err(mlua::Error::runtime(i18n_service().t("server.file_too_large")));
                }

                fs::read_to_string(&full_path).map_err(|e| {
                    mlua::Error::runtime(i18n_service().t_with_options(
                        "server.failed_to_read_file",
                        &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                    ))
                })
            })
            .map_err(|e| {
                format!("{}: {}", i18n_service().t("server.create_read_file_failed"), e)
            })?;
        server_table
            .set("read_file", read_file_fn)
            .map_err(|e| format!("{}: {}", i18n_service().t("server.set_read_file_failed"), e))?;

        let perms = permissions.clone();
        let write_file_fn = self
            .lua
            .create_function(
                move |_, (server_id, relative_path, content): (String, String, String)| {
                    if !perms.iter().any(|p| p == "server") {
                        return Err(mlua::Error::runtime(
                            i18n_service().t("server.permission_denied"),
                        ));
                    }
                    let servers = server_manager().get_server_list();
                    let server = servers.iter().find(|s| s.id == server_id).ok_or_else(|| {
                        mlua::Error::runtime(i18n_service().t_with_options(
                            "server.server_not_found",
                            &crate::plugins::runtime::console::i18n_arg("0", &server_id),
                        ))
                    })?;

                    let server_dir = PathBuf::from(&server.path);
                    let full_path = validate_server_path(&server_dir, &relative_path)?;

                    if let Some(parent) = full_path.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            mlua::Error::runtime(i18n_service().t_with_options(
                                "server.failed_to_create_dir",
                                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                            ))
                        })?;
                    }

                    fs::write(&full_path, content).map_err(|e| {
                        mlua::Error::runtime(i18n_service().t_with_options(
                            "server.failed_to_write_file",
                            &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                        ))
                    })?;
                    Ok(true)
                },
            )
            .map_err(|e| {
                format!("{}: {}", i18n_service().t("server.create_write_file_failed"), e)
            })?;
        server_table
            .set("write_file", write_file_fn)
            .map_err(|e| format!("{}: {}", i18n_service().t("server.set_write_file_failed"), e))?;

        let perms = permissions.clone();
        let list_dir_fn = self
            .lua
            .create_function(move |lua, (server_id, relative_path): (String, String)| {
                if !perms.iter().any(|p| p == "server") {
                    return Err(mlua::Error::runtime(i18n_service().t("server.permission_denied")));
                }
                let servers = server_manager().get_server_list();
                let server = servers.iter().find(|s| s.id == server_id).ok_or_else(|| {
                    mlua::Error::runtime(i18n_service().t_with_options(
                        "server.server_not_found",
                        &crate::plugins::runtime::console::i18n_arg("0", &server_id),
                    ))
                })?;

                let server_dir = PathBuf::from(&server.path);
                let full_path = validate_server_path(&server_dir, &relative_path)?;

                if !full_path.is_dir() {
                    return Err(mlua::Error::runtime(
                        i18n_service().t("server.path_not_directory"),
                    ));
                }

                let entries = fs::read_dir(&full_path).map_err(|e| {
                    mlua::Error::runtime(i18n_service().t_with_options(
                        "server.failed_to_read_dir",
                        &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                    ))
                })?;

                let result = lua.create_table()?;
                let mut i = 1;
                for entry in entries {
                    let entry = entry.map_err(|e| {
                        mlua::Error::runtime(i18n_service().t_with_options(
                            "server.failed_to_read_entry",
                            &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                        ))
                    })?;
                    let metadata = entry.metadata().map_err(|e| {
                        mlua::Error::runtime(i18n_service().t_with_options(
                            "server.failed_to_get_metadata",
                            &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                        ))
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
            .map_err(|e| format!("{}: {}", i18n_service().t("server.create_list_dir_failed"), e))?;
        server_table
            .set("list_dir", list_dir_fn)
            .map_err(|e| format!("{}: {}", i18n_service().t("server.set_list_dir_failed"), e))?;

        let perms = permissions.clone();
        let exists_fn = self
            .lua
            .create_function(move |_, (server_id, relative_path): (String, String)| {
                if !perms.iter().any(|p| p == "server") {
                    return Err(mlua::Error::runtime(i18n_service().t("server.permission_denied")));
                }
                let servers = server_manager().get_server_list();
                let server = servers.iter().find(|s| s.id == server_id).ok_or_else(|| {
                    mlua::Error::runtime(i18n_service().t_with_options(
                        "server.server_not_found",
                        &crate::plugins::runtime::console::i18n_arg("0", &server_id),
                    ))
                })?;

                let server_dir = PathBuf::from(&server.path);
                let full_path = validate_server_path(&server_dir, &relative_path)?;

                Ok(full_path.exists())
            })
            .map_err(|e| format!("{}: {}", i18n_service().t("server.create_exists_failed"), e))?;
        server_table
            .set("exists", exists_fn)
            .map_err(|e| format!("{}: {}", i18n_service().t("server.set_exists_failed"), e))?;

        let perms = permissions.clone();
        let logs_table = self.lua.create_table().map_err(|e| {
            format!("{}: {}", i18n_service().t("server.create_logs_table_failed"), e)
        })?;

        let get_logs_fn = self
            .lua
            .create_function(move |lua, (server_id, count): (String, Option<usize>)| {
                if !perms.iter().any(|p| p == "server") {
                    return Err(mlua::Error::runtime(i18n_service().t("server.permission_denied")));
                }

                let servers = server_manager().get_server_list();
                if !servers.iter().any(|s| s.id == server_id) {
                    return Err(mlua::Error::runtime(i18n_service().t_with_options(
                        "server.server_not_found",
                        &crate::plugins::runtime::console::i18n_arg("0", &server_id),
                    )));
                }

                let count = count.unwrap_or(100).min(1000);
                let all_logs = crate::services::server_log_pipeline::get_logs(&server_id, 0, None);

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
            .map_err(|e| format!("{}: {}", i18n_service().t("server.create_logs_get_failed"), e))?;
        logs_table
            .set("get", get_logs_fn)
            .map_err(|e| format!("{}: {}", i18n_service().t("server.set_logs_get_failed"), e))?;

        let perms = permissions.clone();
        let get_all_logs_fn = self
            .lua
            .create_function(move |lua, count: Option<usize>| {
                if !perms.iter().any(|p| p == "server") {
                    return Err(mlua::Error::runtime(i18n_service().t("server.permission_denied")));
                }

                let count = count.unwrap_or(100).min(1000);

                let running_ids = server_manager().get_running_server_ids();
                let logs_map = crate::services::server_log_pipeline::get_all_logs();

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
            .map_err(|e| {
                format!("{}: {}", i18n_service().t("server.create_logs_getall_failed"), e)
            })?;
        logs_table
            .set("getAll", get_all_logs_fn)
            .map_err(|e| format!("{}: {}", i18n_service().t("server.set_logs_getall_failed"), e))?;

        server_table
            .set("logs", logs_table)
            .map_err(|e| format!("{}: {}", i18n_service().t("server.set_logs_failed"), e))?;

        let perms = permissions.clone();
        let register_log_processor_fn = self
            .lua
            .create_function(move |_lua, callback: mlua::Function| {
                if !perms.iter().any(|p| p == "server") {
                    return Err(mlua::Error::runtime(i18n_service().t("server.permission_denied")));
                }

                let processor = Arc::new(move |server_id: &str, line: &str| {
                    let result = callback.call::<String>((server_id.to_string(), line.to_string()));
                    result.unwrap_or_else(|_| line.to_string())
                });

                crate::plugins::api::register_server_log_processor(processor)
                    .map_err(|e| mlua::Error::runtime(e))?;

                Ok(true)
            })
            .map_err(|e| {
                format!(
                    "{}: {}",
                    i18n_service().t("server.create_register_log_processor_failed"),
                    e
                )
            })?;
        server_table
            .set("register_log_processor", register_log_processor_fn)
            .map_err(|e| {
                format!("{}: {}", i18n_service().t("server.set_register_log_processor_failed"), e)
            })?;

        sl.set("server", server_table)
            .map_err(|e| format!("{}: {}", i18n_service().t("server.set_server_failed"), e))?;

        Ok(())
    }
}
