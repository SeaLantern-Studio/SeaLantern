#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum WebMode {
    #[default]
    Disabled,
    Enabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum CliMode {
    #[default]
    Disabled,
    Enabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct CliServerCommand {
    pub positional_name: Option<String>,
    pub name: Option<String>,
    pub folder: Option<String>,
    pub runtime: Option<String>,
    pub mc_version: Option<String>,
    pub core_type: Option<String>,
    pub port: Option<Option<u16>>,
    pub min_memory_mb: Option<u32>,
    pub max_memory_mb: Option<u32>,
    pub web: WebMode,
    pub web_port: Option<Option<u16>>,
    pub cli: CliMode,
    pub detach: bool,
    pub create_only: bool,
    pub jar_path: Option<String>,
    pub java_path: Option<String>,
    pub java_path_prevalidated: bool,
    pub runtime_prevalidated: bool,
    pub java_from_env_only: bool,
    pub startup_mode: Option<String>,
    pub image: Option<String>,
    pub image_tag: Option<String>,
    pub data_dir: Option<String>,
    pub container_name: Option<String>,
    pub docker_backend: Option<String>,
    pub command_mode: Option<String>,
    pub docker_env: Vec<(String, String)>,
    pub docker_mounts: Vec<String>,
    pub docker_publishes: Vec<String>,
    pub server_tag: Option<String>,
    pub aliases: Vec<String>,
    pub entry: Option<String>,
}

pub(super) fn parse_server_command(args: &[String]) -> Result<CliServerCommand, String> {
    if args.is_empty() {
        return Err("缺少服务器名称或 --name".to_string());
    }
    if matches!(args[0].as_str(), "help" | "--help" | "-h") {
        return Err("__PRINT_HELP__".to_string());
    }

    let mut command = CliServerCommand::default();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];

        if !arg.starts_with('-') {
            if command.positional_name.is_none() {
                command.positional_name = Some(arg.clone());
                index += 1;
                continue;
            }
            return Err(format!("无法识别的多余位置参数: {}", arg));
        }

        if let Some(value) = arg.strip_prefix("-p:") {
            command.port = Some(Some(parse_port(value, "-p")?));
            index += 1;
            continue;
        }
        if arg == "-p" {
            if let Some(next) = args.get(index + 1).filter(|value| !value.starts_with('-')) {
                command.port = Some(Some(parse_port(next, "-p")?));
                index += 2;
            } else {
                command.port = Some(None);
                index += 1;
            }
            continue;
        }
        if let Some(value) = arg.strip_prefix("-p=") {
            command.port = Some(Some(parse_port(value, "-p")?));
            index += 1;
            continue;
        }
        if arg == "--port" {
            if let Some(next) = args.get(index + 1).filter(|value| !value.starts_with('-')) {
                command.port = Some(Some(parse_port(next, "--port")?));
                index += 2;
            } else {
                command.port = Some(None);
                index += 1;
            }
            continue;
        }
        if arg == "--min" {
            let value = args
                .get(index + 1)
                .ok_or_else(|| "参数缺少值: --min".to_string())?;
            command.min_memory_mb = Some(parse_memory_mb(value)?);
            index += 2;
            continue;
        }
        if arg == "--max" {
            let value = args
                .get(index + 1)
                .ok_or_else(|| "参数缺少值: --max".to_string())?;
            command.max_memory_mb = Some(parse_memory_mb(value)?);
            index += 2;
            continue;
        }
        if arg == "--web" {
            command.web = WebMode::Enabled;
            if let Some(next) = args.get(index + 1).filter(|value| !value.starts_with('-')) {
                command.web_port = Some(Some(parse_port(next, "--web")?));
                index += 2;
            } else {
                command.web_port = Some(None);
                index += 1;
            }
            continue;
        }
        if let Some(value) = arg.strip_prefix("--web=") {
            command.web = WebMode::Enabled;
            command.web_port = Some(Some(parse_port(value, "--web")?));
            index += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--port=") {
            command.port = Some(Some(parse_port(value, "--port")?));
            index += 1;
            continue;
        }

        if let Some(value) = arg.strip_prefix("-min:") {
            command.min_memory_mb = Some(parse_memory_mb(value)?);
            index += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("-max:") {
            command.max_memory_mb = Some(parse_memory_mb(value)?);
            index += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("-web:") {
            command.web = WebMode::Enabled;
            command.web_port = Some(Some(parse_port(value, "-web")?));
            index += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--web:") {
            command.web = WebMode::Enabled;
            command.web_port = Some(Some(parse_port(value, "--web")?));
            index += 1;
            continue;
        }

        match arg.as_str() {
            "--cli" => {
                command.cli = CliMode::Enabled;
                index += 1;
                continue;
            }
            "--detach" => {
                command.detach = true;
                index += 1;
                continue;
            }
            "--create-only" | "--no-start" => {
                command.create_only = true;
                index += 1;
                continue;
            }
            "--J" => {
                command.java_from_env_only = true;
                index += 1;
                continue;
            }
            _ => {}
        }

        let (flag, value) = split_flag_value(args, &mut index)?;
        match flag.as_str() {
            "--name" | "--n" => command.name = Some(value),
            "--folder" | "--f" | "--fd" => command.folder = Some(value),
            "--runtime" | "--r" => command.runtime = Some(value),
            "--mc" | "--mc-version" => command.mc_version = Some(value),
            "--core" => command.core_type = Some(value),
            "--min" => command.min_memory_mb = Some(parse_memory_mb(&value)?),
            "--max" => command.max_memory_mb = Some(parse_memory_mb(&value)?),
            "--jar" => command.jar_path = Some(value),
            "--java" | "--j" => command.java_path = Some(value),
            "--startup" => command.startup_mode = Some(value),
            "--image" => command.image = Some(value),
            "--tag" | "--t" => command.server_tag = Some(value),
            "--image-tag" => command.image_tag = Some(value),
            "--data-dir" => command.data_dir = Some(value),
            "--container-name" => command.container_name = Some(value),
            "--docker-backend" => command.docker_backend = Some(value),
            "--command-mode" => command.command_mode = Some(value),
            "--env" => {
                let (key, env_value) = parse_env_assignment(&value)?;
                command.docker_env.push((key, env_value));
            }
            "--mount" => command.docker_mounts.push(value),
            "--publish" => command.docker_publishes.push(value),
            "--alias" => command.aliases.push(value),
            "--entry" => command.entry = Some(value),
            other => return Err(format!("未知参数: {}", other)),
        }
    }

    Ok(command)
}

fn split_flag_value(args: &[String], index: &mut usize) -> Result<(String, String), String> {
    let arg = &args[*index];
    if let Some((flag, value)) = arg.split_once('=') {
        *index += 1;
        return Ok((flag.to_string(), value.to_string()));
    }

    let flag = arg.clone();
    *index += 1;
    let value = args
        .get(*index)
        .ok_or_else(|| format!("参数缺少值: {}", flag))?
        .clone();
    *index += 1;
    Ok((flag, value))
}

fn parse_port(value: &str, flag: &str) -> Result<u16, String> {
    value
        .parse::<u16>()
        .map_err(|_| format!("{} 需要有效端口号: {}", flag, value))
}

fn parse_memory_mb(value: &str) -> Result<u32, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("内存值不能为空".to_string());
    }

    let upper = trimmed.to_ascii_uppercase();
    if let Some(number) = upper.strip_suffix('G') {
        let base = number
            .trim()
            .parse::<u32>()
            .map_err(|_| format!("无效的内存值: {}", value))?;
        return Ok(base.saturating_mul(1024));
    }
    if let Some(number) = upper.strip_suffix('M') {
        return number
            .trim()
            .parse::<u32>()
            .map_err(|_| format!("无效的内存值: {}", value));
    }

    upper
        .parse::<u32>()
        .map_err(|_| format!("无效的内存值: {}", value))
}

fn parse_env_assignment(value: &str) -> Result<(String, String), String> {
    let Some((key, env_value)) = value.split_once('=') else {
        return Err(format!("--env 需要 KEY=VALUE 形式: {}", value));
    };

    let key = key.trim();
    if key.is_empty() {
        return Err(format!("--env 键不能为空: {}", value));
    }

    Ok((key.to_string(), env_value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{parse_memory_mb, parse_server_command, CliMode, WebMode};

    #[test]
    fn parse_memory_mb_supports_g_and_m() {
        assert_eq!(parse_memory_mb("2G").unwrap(), 2048);
        assert_eq!(parse_memory_mb("512M").unwrap(), 512);
        assert_eq!(parse_memory_mb("1024").unwrap(), 1024);
    }

    #[test]
    fn parse_server_command_supports_compact_flags() {
        let args = vec![
            "fabric-1.20.1".to_string(),
            "--mc".to_string(),
            "1.20.1".to_string(),
            "--core".to_string(),
            "fabric".to_string(),
            "-p:25565".to_string(),
            "-min:2G".to_string(),
            "-max:4G".to_string(),
            "-web:8000".to_string(),
            "--jar".to_string(),
            "E:/srv/server.jar".to_string(),
            "--java".to_string(),
            "C:/Java/bin/java.exe".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(parsed.positional_name.as_deref(), Some("fabric-1.20.1"));
        assert_eq!(parsed.port, Some(Some(25565)));
        assert_eq!(parsed.min_memory_mb, Some(2048));
        assert_eq!(parsed.max_memory_mb, Some(4096));
        assert_eq!(parsed.web, WebMode::Enabled);
        assert_eq!(parsed.web_port, Some(Some(8000)));
    }

    #[test]
    fn parse_server_command_supports_name_and_folder_aliases() {
        let args = vec![
            "--n".to_string(),
            "cache server".to_string(),
            "--fd".to_string(),
            "E:/servers/cache".to_string(),
            "--cli".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(parsed.name.as_deref(), Some("cache server"));
        assert_eq!(parsed.folder.as_deref(), Some("E:/servers/cache"));
    }

    #[test]
    fn parse_server_command_supports_equals_style_flags() {
        let args = vec![
            "paper-docker".to_string(),
            "--runtime=docker".to_string(),
            "--mc=1.21.1".to_string(),
            "--core=paper".to_string(),
            "--image=itzg/minecraft-server".to_string(),
            "--image-tag=java21".to_string(),
            "--web=8000".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(parsed.runtime.as_deref(), Some("docker"));
        assert_eq!(parsed.mc_version.as_deref(), Some("1.21.1"));
        assert_eq!(parsed.image.as_deref(), Some("itzg/minecraft-server"));
        assert_eq!(parsed.web_port, Some(Some(8000)));
    }

    #[test]
    fn parse_server_command_supports_web_and_cli_together() {
        let args = vec![
            "fabric-1.20.1".to_string(),
            "--mc".to_string(),
            "1.20.1".to_string(),
            "--core".to_string(),
            "fabric".to_string(),
            "--web".to_string(),
            "--cli".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(parsed.web, WebMode::Enabled);
        assert_eq!(parsed.web_port, Some(None));
        assert_eq!(parsed.cli, CliMode::Enabled);
    }

    #[test]
    fn parse_server_command_supports_detach_mode() {
        let args = vec![
            "paper-docker".to_string(),
            "--runtime".to_string(),
            "docker".to_string(),
            "--mc".to_string(),
            "1.21.1".to_string(),
            "--core".to_string(),
            "paper".to_string(),
            "--detach".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert!(parsed.detach);
        assert_eq!(parsed.cli, CliMode::Disabled);
        assert_eq!(parsed.web, WebMode::Disabled);
    }

    #[test]
    fn parse_server_command_supports_create_only_mode() {
        let args = vec![
            "paper-docker".to_string(),
            "--runtime".to_string(),
            "docker".to_string(),
            "--create-only".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert!(parsed.create_only);
        assert!(!parsed.detach);
    }

    #[test]
    fn parse_server_command_supports_double_dash_web_colon_style() {
        let args = vec![
            "fabric-1.20.1".to_string(),
            "--mc".to_string(),
            "1.20.1".to_string(),
            "--core".to_string(),
            "fabric".to_string(),
            "--web:8000".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(parsed.web, WebMode::Enabled);
        assert_eq!(parsed.web_port, Some(Some(8000)));
    }

    #[test]
    fn parse_server_command_supports_space_style_web_and_port_values() {
        let args = vec![
            "fabric-1.20.1".to_string(),
            "--mc".to_string(),
            "1.20.1".to_string(),
            "--core".to_string(),
            "fabric".to_string(),
            "--web".to_string(),
            "8000".to_string(),
            "-p".to_string(),
            "25570".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(parsed.web, WebMode::Enabled);
        assert_eq!(parsed.web_port, Some(Some(8000)));
        assert_eq!(parsed.port, Some(Some(25570)));
    }

    #[test]
    fn parse_server_command_uses_tag_as_name_fallback_but_not_docker_image_tag() {
        let args = vec![
            "--tag".to_string(),
            "cache-server".to_string(),
            "--runtime".to_string(),
            "docker".to_string(),
            "--mc".to_string(),
            "1.21.1".to_string(),
            "--core".to_string(),
            "paper".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(parsed.server_tag.as_deref(), Some("cache-server"));
        assert!(parsed.image_tag.is_none());
    }

    #[test]
    fn parse_server_command_matches_goal_style_local_command_shape() {
        let args = vec![
            "fabric-1.20.1".to_string(),
            "--mc".to_string(),
            "1.20.1".to_string(),
            "--core".to_string(),
            "fabric".to_string(),
            "--jar".to_string(),
            "E:/srv/server.jar".to_string(),
            "--java".to_string(),
            "C:/Java/bin/java.exe".to_string(),
            "-p:25565".to_string(),
            "-min:2G".to_string(),
            "-max:4G".to_string(),
            "-web:8000".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(parsed.positional_name.as_deref(), Some("fabric-1.20.1"));
        assert_eq!(parsed.mc_version.as_deref(), Some("1.20.1"));
        assert_eq!(parsed.core_type.as_deref(), Some("fabric"));
        assert_eq!(parsed.jar_path.as_deref(), Some("E:/srv/server.jar"));
        assert_eq!(parsed.java_path.as_deref(), Some("C:/Java/bin/java.exe"));
        assert_eq!(parsed.port, Some(Some(25565)));
        assert_eq!(parsed.min_memory_mb, Some(2048));
        assert_eq!(parsed.max_memory_mb, Some(4096));
        assert_eq!(parsed.web, WebMode::Enabled);
        assert_eq!(parsed.web_port, Some(Some(8000)));
        assert_eq!(parsed.cli, CliMode::Disabled);
    }

    #[test]
    fn parse_server_command_supports_env_java_aliases_and_repeated_aliases() {
        let args = vec![
            "cache-server".to_string(),
            "--mc".to_string(),
            "1.20.1".to_string(),
            "--core".to_string(),
            "fabric".to_string(),
            "--java".to_string(),
            "%env:JAVA_HOME%".to_string(),
            "--alias".to_string(),
            "cache_server".to_string(),
            "--alias".to_string(),
            "test_server".to_string(),
            "--cli".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(parsed.java_path.as_deref(), Some("%env:JAVA_HOME%"));
        assert_eq!(parsed.aliases, vec!["cache_server", "test_server"]);
        assert_eq!(parsed.cli, CliMode::Enabled);
    }

    #[test]
    fn parse_server_command_supports_env_only_java_mode_and_custom_entry() {
        let args = vec![
            "custom-local".to_string(),
            "--mc".to_string(),
            "1.20.1".to_string(),
            "--core".to_string(),
            "fabric".to_string(),
            "--J".to_string(),
            "--entry".to_string(),
            "java -Xmx4G -Xms4G -jar server.jar nogui".to_string(),
            "--cli".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert!(parsed.java_from_env_only);
        assert_eq!(parsed.entry.as_deref(), Some("java -Xmx4G -Xms4G -jar server.jar nogui"));
        assert_eq!(parsed.cli, CliMode::Enabled);
    }

    #[test]
    fn parse_server_command_supports_repeated_docker_env_and_mount_flags() {
        let args = vec![
            "paper-docker".to_string(),
            "--runtime".to_string(),
            "docker".to_string(),
            "--env".to_string(),
            "STOP_DURATION=180".to_string(),
            "--env=DISABLE_HEALTHCHECK=true".to_string(),
            "--mount".to_string(),
            "E:/plugins:/data/plugins:ro".to_string(),
            "--publish".to_string(),
            "24454:24454/udp".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(
            parsed.docker_env,
            vec![
                ("STOP_DURATION".to_string(), "180".to_string()),
                ("DISABLE_HEALTHCHECK".to_string(), "true".to_string())
            ]
        );
        assert_eq!(parsed.docker_mounts, vec!["E:/plugins:/data/plugins:ro"]);
        assert_eq!(parsed.docker_publishes, vec!["24454:24454/udp"]);
    }
}
