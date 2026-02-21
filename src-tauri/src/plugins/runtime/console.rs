use super::PluginRuntime;
use mlua::Table;

impl PluginRuntime {
    pub(super) fn setup_console_namespace(&self, sl: &Table) -> Result<(), String> {
        use crate::plugins::api::emit_permission_log;
        use crate::services::global::server_manager;

        let console_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create console table: {}", e))?;

        let plugin_id = self.plugin_id.clone();

        const ALLOWED_COMMANDS: &[&str] = &[
            "tell",
            "msg",
            "w",
            "say",
            "teammsg",
            "me",
            "give",
            "clear",
            "xp",
            "experience",
            "kick",
            "ban",
            "pardon",
            "banlist",
            "whitelist",
            "op",
            "deop",
            "effect",
            "enchant",
            "time",
            "weather",
            "gamerule",
            "difficulty",
            "gamemode",
            "spawnpoint",
            "tp",
            "teleport",
            "spreadplayers",
            "particle",
            "playsound",
            "title",
        ];

        const BLOCKED_COMMANDS: &[&str] = &[
            "stop",
            "reload",
            "restart",
            "plugins",
            "plugin",
            "version",
            "debug",
            "save-all",
            "save-off",
            "save-on",
            "list",
            "help",
            "seed",
            "timings",
            "perworldinventory",
            "pwi",
        ];

        fn is_command_allowed(command: &str) -> Result<(), String> {
            let cmd_lower = command.to_lowercase();
            let cmd_first = cmd_lower.split_whitespace().next().unwrap_or("");

            for blocked in BLOCKED_COMMANDS {
                if cmd_first == *blocked {
                    return Err(format!("命令 '{}' 被禁止执行", command));
                }
            }

            for allowed in ALLOWED_COMMANDS {
                if cmd_first == *allowed {
                    return Ok(());
                }
            }

            Err(format!(
                "命令 '{}' 不在允许列表中。允许的命令: {}",
                command,
                ALLOWED_COMMANDS.join(", ")
            ))
        }

        let pid = plugin_id.clone();
        let send_fn = self
            .lua
            .create_function(move |_, (server_id, command): (String, String)| {
                let servers = server_manager().get_server_list();
                if !servers.iter().any(|s| s.id == server_id) {
                    return Err(mlua::Error::runtime(format!("服务器不存在: {}", server_id)));
                }

                let status = server_manager().get_server_status(&server_id);
                if status.status != crate::models::server::ServerStatus::Running {
                    return Err(mlua::Error::runtime("服务器未运行"));
                }

                if let Err(e) = is_command_allowed(&command) {
                    return Err(mlua::Error::runtime(e));
                }

                let _ = emit_permission_log(
                    &pid,
                    "command",
                    "sl.console.send",
                    &format!("[{}] {}", server_id, command),
                );

                server_manager()
                    .send_command(&server_id, &command)
                    .map_err(|e| mlua::Error::runtime(format!("发送命令失败: {}", e)))?;

                Ok(true)
            })
            .map_err(|e| format!("Failed to create console.send: {}", e))?;
        console_table
            .set("send", send_fn)
            .map_err(|e| format!("Failed to set console.send: {}", e))?;

        let get_logs_fn = self
            .lua
            .create_function(move |lua, (server_id, count): (String, Option<usize>)| {
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
                for (i, log) in logs.iter().enumerate() {
                    let entry = lua.create_table()?;
                    entry.set("content", log.clone())?;
                    result.set(i + 1, entry)?;
                }

                Ok(result)
            })
            .map_err(|e| format!("Failed to create console.get_logs: {}", e))?;
        console_table
            .set("get_logs", get_logs_fn)
            .map_err(|e| format!("Failed to set console.get_logs: {}", e))?;

        let get_status_fn = self
            .lua
            .create_function(move |_, server_id: String| {
                let servers = server_manager().get_server_list();
                if !servers.iter().any(|s| s.id == server_id) {
                    return Err(mlua::Error::runtime(format!("服务器不存在: {}", server_id)));
                }

                let status = server_manager().get_server_status(&server_id);
                let status_str = match status.status {
                    crate::models::server::ServerStatus::Running => "running",
                    crate::models::server::ServerStatus::Stopped => "stopped",
                    crate::models::server::ServerStatus::Starting => "starting",
                    crate::models::server::ServerStatus::Stopping => "stopping",
                    crate::models::server::ServerStatus::Error => "error",
                };

                Ok(status_str.to_string())
            })
            .map_err(|e| format!("Failed to create console.get_status: {}", e))?;
        console_table
            .set("get_status", get_status_fn)
            .map_err(|e| format!("Failed to set console.get_status: {}", e))?;

        sl.set("console", console_table)
            .map_err(|e| format!("Failed to set sl.console: {}", e))?;

        Ok(())
    }
}
