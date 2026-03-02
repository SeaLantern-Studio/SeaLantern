use super::PluginRuntime;
use mlua::Table;
use std::collections::{HashMap, HashSet};

pub fn i18n_arg(key: &str, value: &str) -> HashMap<String, String> {
    HashMap::from([(key.to_string(), value.to_string())])
}

pub fn i18n_args(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

impl PluginRuntime {
    pub(super) fn setup_console_namespace(&self, sl: &Table) -> Result<(), String> {
        use crate::plugins::api::emit_permission_log;
        use crate::services::global::{i18n_service, server_manager, settings_manager};

        let console_table = self.lua.create_table().map_err(|e| {
            i18n_service()
                .t_with_options("console.create_table_failed", &i18n_arg("0", &e.to_string()))
        })?;

        let plugin_id = self.plugin_id.clone();

        // 日志相关
        const DEFAULT_LOG_COUNT: usize = 100;
        const MAX_LOG_COUNT: usize = 1000;

        fn validate_server_id(server_id: &str) -> Result<(), String> {
            let servers = server_manager().get_server_list();
            if !servers.iter().any(|s| s.id == server_id) {
                return Err(i18n_service()
                    .t_with_options("console.server_not_found", &i18n_arg("0", server_id)));
            }
            Ok(())
        }

        fn with_valid_server<T, F>(server_id: &str, f: F) -> Result<T, mlua::Error>
        where
            F: FnOnce() -> Result<T, mlua::Error>,
        {
            validate_server_id(server_id).map_err(mlua::Error::runtime)?;
            f()
        }

        fn sanitize_command(command: &str) -> Result<String, String> {
            const FORBIDDEN_CHARS: &[char] = &[
                '|', ';', '\n', '\r', '&', '$', '`', '<', '>', '\t', '\\', '(', ')', '[', ']', '{',
                '}',
            ];

            if command.contains(FORBIDDEN_CHARS) {
                return Err(i18n_service().t("console.command_has_forbidden_chars"));
            }

            let trimmed = command.trim();
            if trimmed.is_empty() {
                return Err(i18n_service().t("console.empty_command"));
            }

            Ok(trimmed.to_string())
        }

        fn is_command_allowed(command: &str) -> Result<String, String> {
            let sanitized = sanitize_command(command)?;
            let cmd_lower = sanitized.to_lowercase();
            let cmd_first = cmd_lower.split_whitespace().next().unwrap_or("");

            let settings = settings_manager().get();
            let allowed: HashSet<&str> = settings
                .plugin_allowed_commands
                .iter()
                .map(|s| s.as_str())
                .collect();
            let blocked: HashSet<&str> = settings
                .plugin_blocked_commands
                .iter()
                .map(|s| s.as_str())
                .collect();

            if blocked.contains(cmd_first) {
                return Err(i18n_service()
                    .t_with_options("console.command_forbidden", &i18n_arg("0", command)));
            }

            if allowed.contains(cmd_first) {
                return Ok(sanitized);
            }

            Err(i18n_service().t_with_options(
                "console.command_not_allowed",
                &i18n_args(&[
                    ("0", command),
                    ("1", &allowed.iter().copied().collect::<Vec<_>>().join(", ")),
                ]),
            ))
        }

        let pid = plugin_id.clone();
        let send_fn = self
            .lua
            .create_function(move |_, (server_id, command): (String, String)| {
                with_valid_server(&server_id, || {
                    let status = server_manager().get_server_status(&server_id);
                    if status.status != crate::models::server::ServerStatus::Running {
                        return Err(mlua::Error::runtime(
                            i18n_service().t("console.server_not_running"),
                        ));
                    }

                    let sanitized_cmd =
                        is_command_allowed(&command).map_err(mlua::Error::runtime)?;

                    match server_manager().send_command(&server_id, &sanitized_cmd) {
                        Ok(_) => {
                            let _ = emit_permission_log(
                                &pid,
                                "command",
                                "sl.console.send",
                                &format!("[{}] {}", server_id, sanitized_cmd),
                            );
                            Ok(true)
                        }
                        Err(e) => Err(mlua::Error::runtime(i18n_service().t_with_options(
                            "console.send_command_failed",
                            &i18n_arg("0", &e.to_string()),
                        ))),
                    }
                })
            })
            .map_err(|e| {
                i18n_service()
                    .t_with_options("console.create_send_failed", &i18n_arg("0", &e.to_string()))
            })?;
        console_table.set("send", send_fn).map_err(|e| {
            i18n_service().t_with_options("console.set_send_failed", &i18n_arg("0", &e.to_string()))
        })?;

        let get_logs_fn = self
            .lua
            .create_function(move |lua, (server_id, count): (String, Option<usize>)| {
                with_valid_server(&server_id, || {
                    let count = count.unwrap_or(DEFAULT_LOG_COUNT).min(MAX_LOG_COUNT);

                    let logs =
                        crate::services::server_log_pipeline::get_logs(&server_id, 0, Some(count));

                    let result = lua.create_table()?;
                    for (i, log) in logs.iter().enumerate() {
                        let entry = lua.create_table()?;
                        entry.set("content", log.clone())?;
                        result.set(i + 1, entry)?;
                    }

                    Ok(result)
                })
            })
            .map_err(|e| {
                i18n_service().t_with_options(
                    "console.create_get_logs_failed",
                    &i18n_arg("0", &e.to_string()),
                )
            })?;
        console_table.set("get_logs", get_logs_fn).map_err(|e| {
            i18n_service()
                .t_with_options("console.set_get_logs_failed", &i18n_arg("0", &e.to_string()))
        })?;

        let get_status_fn = self
            .lua
            .create_function(move |_, server_id: String| {
                with_valid_server(&server_id, || {
                    let status = server_manager().get_server_status(&server_id);
                    Ok(status.status.as_str().to_string())
                })
            })
            .map_err(|e| {
                i18n_service().t_with_options(
                    "console.create_get_status_failed",
                    &i18n_arg("0", &e.to_string()),
                )
            })?;
        console_table
            .set("get_status", get_status_fn)
            .map_err(|e| {
                i18n_service()
                    .t_with_options("console.set_get_status_failed", &i18n_arg("0", &e.to_string()))
            })?;

        sl.set("console", console_table).map_err(|e| {
            i18n_service()
                .t_with_options("console.set_console_failed", &i18n_arg("0", &e.to_string()))
        })?;

        Ok(())
    }
}
