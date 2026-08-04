use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::core_parsing::{extract_minecraft_version, CoreKind};

/// 根据启动文件扩展名选择的脚本格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupScriptKind {
    Batch,
    Shell,
    PowerShell,
}

impl StartupScriptKind {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|extension| extension.to_str())? {
            extension
                if extension.eq_ignore_ascii_case("bat")
                    || extension.eq_ignore_ascii_case("cmd") =>
            {
                Some(Self::Batch)
            }
            extension if extension.eq_ignore_ascii_case("sh") => Some(Self::Shell),
            extension if extension.eq_ignore_ascii_case("ps1") => Some(Self::PowerShell),
            _ => None,
        }
    }
}

/// 在启动脚本中发现的一个 Java 进程调用。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaLaunch {
    pub java_command: String,
    pub jvm_arguments: Vec<String>,
    pub jar_path: Option<PathBuf>,
    pub main_class: Option<String>,
    pub argument_files: Vec<PathBuf>,
    pub application_arguments: Vec<String>,
}

/// 解析的启动脚本元数据，不执行 shell 内容或展开变量。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupScriptInfo {
    pub kind: StartupScriptKind,
    pub launches: Vec<JavaLaunch>,
    pub inferred_core: CoreKind,
    pub minecraft_version: Option<String>,
}

/// 从文件解析启动脚本，不执行它。
pub fn parse_startup_script_file(path: &Path) -> Result<StartupScriptInfo, StartupParseError> {
    let kind = StartupScriptKind::from_path(path)
        .ok_or_else(|| StartupParseError::UnsupportedScript { path: path.to_path_buf() })?;
    let content = fs::read_to_string(path)
        .map_err(|source| StartupParseError::Read { path: path.to_path_buf(), source })?;
    Ok(parse_startup_script_content(kind, &content))
}

/// 解析启动脚本内容，不执行命令或展开变量。
pub fn parse_startup_script_content(kind: StartupScriptKind, content: &str) -> StartupScriptInfo {
    let launches = logical_script_lines(kind, content)
        .iter()
        .flat_map(|line| split_command_segments(kind, line))
        .filter_map(|segment| parse_java_launch(&segment))
        .collect::<Vec<_>>();
    let inferred_core = infer_core_kind(&launches);
    let minecraft_version = infer_minecraft_version(&launches);

    StartupScriptInfo {
        kind,
        launches,
        inferred_core,
        minecraft_version,
    }
}

fn logical_script_lines(kind: StartupScriptKind, content: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for line in content.lines() {
        let line = line.trim_end();
        if let Some(line) = remove_continuation_marker(kind, line) {
            current.push_str(line);
            if !current.is_empty() && !current.chars().last().is_some_and(char::is_whitespace) {
                current.push(' ');
            }
            continue;
        }

        current.push_str(line);
        if !current.trim().is_empty() {
            lines.push(std::mem::take(&mut current));
        } else {
            current.clear();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn remove_continuation_marker(kind: StartupScriptKind, line: &str) -> Option<&str> {
    let marker = match kind {
        StartupScriptKind::Batch => '^',
        StartupScriptKind::Shell => '\\',
        StartupScriptKind::PowerShell => '`',
    };
    let marker_count = line
        .chars()
        .rev()
        .take_while(|character| *character == marker)
        .count();

    if marker_count % 2 == 1 {
        Some(&line[..line.len() - marker.len_utf8()])
    } else {
        None
    }
}

fn split_command_segments(kind: StartupScriptKind, line: &str) -> Vec<String> {
    let line = line.trim_start_matches('@').trim();
    if is_comment_or_control_line(kind, line) {
        return Vec::new();
    }

    let separators = match kind {
        StartupScriptKind::Batch => "&",
        StartupScriptKind::Shell | StartupScriptKind::PowerShell => ";",
    };
    split_outside_quotes(line, separators)
}

fn is_comment_or_control_line(kind: StartupScriptKind, line: &str) -> bool {
    let lowercase = line.to_ascii_lowercase();
    if line.is_empty() || lowercase == "pause" || lowercase.starts_with("echo ") {
        return true;
    }

    match kind {
        StartupScriptKind::Batch => lowercase.starts_with("rem ") || line.starts_with("::"),
        StartupScriptKind::Shell | StartupScriptKind::PowerShell => line.starts_with('#'),
    }
}

fn split_outside_quotes(line: &str, separators: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut segment = String::new();
    let mut quote = None;
    let mut characters = line.chars().peekable();

    while let Some(character) = characters.next() {
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            segment.push(character);
            continue;
        }
        if quote.is_none() && separators.contains(character) {
            if character == '&' && characters.peek() == Some(&'&') {
                let _ = characters.next();
            }
            if !segment.trim().is_empty() {
                segments.push(segment.trim().to_string());
            }
            segment.clear();
            continue;
        }
        segment.push(character);
    }

    if !segment.trim().is_empty() {
        segments.push(segment.trim().to_string());
    }
    segments
}

fn parse_java_launch(segment: &str) -> Option<JavaLaunch> {
    let tokens = tokenize(segment);
    let java_index = tokens.iter().position(|token| is_java_command(token))?;
    let java_command = tokens[java_index].clone();
    let mut launch = JavaLaunch {
        java_command,
        jvm_arguments: Vec::new(),
        jar_path: None,
        main_class: None,
        argument_files: Vec::new(),
        application_arguments: Vec::new(),
    };
    let mut index = java_index + 1;
    let mut parse_application_arguments = false;

    while index < tokens.len() {
        let token = &tokens[index];
        if parse_application_arguments {
            launch.application_arguments.push(token.clone());
            index += 1;
            continue;
        }
        if token == "-jar" {
            if let Some(jar_path) = tokens.get(index + 1) {
                launch.jar_path = Some(PathBuf::from(jar_path));
                index += 2;
                parse_application_arguments = true;
                continue;
            }
            index += 1;
            continue;
        }
        if let Some(argument_file) = token.strip_prefix('@') {
            if !argument_file.is_empty() {
                launch.argument_files.push(PathBuf::from(argument_file));
            }
            index += 1;
            continue;
        }
        if token == "-cp" || token == "-classpath" {
            launch.jvm_arguments.push(token.clone());
            if let Some(class_path) = tokens.get(index + 1) {
                launch.jvm_arguments.push(class_path.clone());
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }
        if token.starts_with('-') {
            launch.jvm_arguments.push(token.clone());
            index += 1;
            continue;
        }
        launch.main_class = Some(token.clone());
        parse_application_arguments = true;
        index += 1;
    }

    Some(launch)
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quote = None;

    for character in input.chars() {
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            } else {
                token.push(character);
            }
            continue;
        }
        if quote.is_none() && character.is_whitespace() {
            if !token.is_empty() {
                tokens.push(std::mem::take(&mut token));
            }
            continue;
        }
        token.push(character);
    }
    if !token.is_empty() {
        tokens.push(token);
    }
    tokens
}

fn is_java_command(token: &str) -> bool {
    let executable = Path::new(token)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(token)
        .to_ascii_lowercase();
    matches!(executable.as_str(), "java" | "java.exe" | "javaw" | "javaw.exe")
}

fn infer_core_kind(launches: &[JavaLaunch]) -> CoreKind {
    launches
        .iter()
        .flat_map(|launch| launch.jar_path.iter().chain(launch.argument_files.iter()))
        .map(|path| CoreKind::from_filename(&path.to_string_lossy()))
        .find(|kind| *kind != CoreKind::Unknown)
        .unwrap_or(CoreKind::Unknown)
}

fn infer_minecraft_version(launches: &[JavaLaunch]) -> Option<String> {
    launches.iter().find_map(|launch| {
        launch
            .jar_path
            .iter()
            .chain(launch.argument_files.iter())
            .find_map(|path| extract_minecraft_version(&path.to_string_lossy()))
    })
}

/// 描述加载启动脚本进行静态解析时的错误。
#[derive(Debug)]
pub enum StartupParseError {
    UnsupportedScript { path: PathBuf },
    Read { path: PathBuf, source: io::Error },
}

impl fmt::Display for StartupParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedScript { path } => {
                write!(formatter, "unsupported startup script extension for {}", path.display())
            }
            Self::Read { path, source } => {
                write!(formatter, "could not read startup script {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for StartupParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnsupportedScript { .. } => None,
            Self::Read { source, .. } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{logical_script_lines, parse_startup_script_content, CoreKind, StartupScriptKind};

    #[test]
    fn logical_lines_preserve_existing_continuation_whitespace() {
        let lines =
            logical_script_lines(StartupScriptKind::Batch, "java -Xmx2G ^\n-jar paper-1.21.1.jar");

        assert_eq!(lines, vec!["java -Xmx2G -jar paper-1.21.1.jar"]);
    }

    #[test]
    fn logical_lines_skip_blank_lines() {
        let lines =
            logical_script_lines(StartupScriptKind::Batch, "\n  \njava -jar paper-1.21.1.jar\n\t");

        assert_eq!(lines, vec!["java -jar paper-1.21.1.jar"]);
    }

    #[test]
    fn parses_a_direct_jar_launch_with_velocity_jvm_flags() {
        let content = "@echo off\n\njava -Xms4096M -Xmx4096M -XX:+AlwaysPreTouch -XX:+ParallelRefProcEnabled -XX:+UnlockExperimentalVMOptions -XX:+UseG1GC -XX:G1HeapRegionSize=4M -XX:MaxInlineLevel=15 -jar server.jar\n\npause";

        let parsed = parse_startup_script_content(StartupScriptKind::Batch, content);

        assert_eq!(parsed.launches.len(), 1);
        let launch = &parsed.launches[0];
        assert_eq!(launch.java_command, "java");
        assert_eq!(launch.jar_path, Some(PathBuf::from("server.jar")));
        assert_eq!(launch.jvm_arguments.len(), 8);
        assert!(launch.jvm_arguments.contains(&"-XX:+UseG1GC".to_string()));
    }

    #[test]
    fn parses_modern_forge_argument_file_launches() {
        let content = "@echo off\njava @user_jvm_args.txt @libraries/net/minecraftforge/forge/1.20.1-47.2.0/win_args.txt %*\npause";

        let parsed = parse_startup_script_content(StartupScriptKind::Batch, content);

        assert_eq!(parsed.launches.len(), 1);
        assert_eq!(parsed.inferred_core, CoreKind::Forge);
        assert_eq!(parsed.minecraft_version.as_deref(), Some("1.20.1"));
        assert_eq!(
            parsed.launches[0].argument_files,
            vec![
                PathBuf::from("user_jvm_args.txt"),
                PathBuf::from("libraries/net/minecraftforge/forge/1.20.1-47.2.0/win_args.txt"),
            ]
        );
    }

    #[test]
    fn parses_a_multiline_batch_forge_launch() {
        let content = "java -Xmx4G ^\n  @user_jvm_args.txt ^\n  @libraries/net/minecraftforge/forge/1.20.1-47.2.0/win_args.txt %*";

        let parsed = parse_startup_script_content(StartupScriptKind::Batch, content);

        assert_eq!(parsed.launches.len(), 1);
        assert_eq!(parsed.inferred_core, CoreKind::Forge);
        assert_eq!(parsed.launches[0].jvm_arguments, vec!["-Xmx4G"]);
        assert_eq!(parsed.launches[0].argument_files.len(), 2);
    }

    #[test]
    fn parses_neoforge_argument_file_launches_before_forge() {
        let content = "java @libraries/net/neoforged/neoforge/1.21.1-21.1.96/win_args.txt";

        let parsed = parse_startup_script_content(StartupScriptKind::Batch, content);

        assert_eq!(parsed.inferred_core, CoreKind::NeoForge);
        assert_eq!(parsed.minecraft_version.as_deref(), Some("1.21.1"));
    }

    #[test]
    fn shell_parser_keeps_quoted_jar_paths_together() {
        let content = "java -Xmx2G -jar \"server files/paper-1.21.1.jar\" nogui";

        let parsed = parse_startup_script_content(StartupScriptKind::Shell, content);

        assert_eq!(
            parsed.launches[0].jar_path,
            Some(PathBuf::from("server files/paper-1.21.1.jar"))
        );
        assert_eq!(parsed.inferred_core, CoreKind::Paper);
    }

    #[test]
    fn parses_shell_and_powershell_line_continuations() {
        let shell = parse_startup_script_content(
            StartupScriptKind::Shell,
            concat!("java -Xmx2G ", "\\", "\n", "             -jar paper-1.21.1.jar nogui"),
        );
        let powershell = parse_startup_script_content(
            StartupScriptKind::PowerShell,
            "java -Xmx2G `\n             -jar paper-1.21.1.jar nogui",
        );

        for parsed in [shell, powershell] {
            assert_eq!(parsed.launches.len(), 1);
            assert_eq!(parsed.launches[0].jvm_arguments, vec!["-Xmx2G"]);
            assert_eq!(parsed.launches[0].jar_path, Some(PathBuf::from("paper-1.21.1.jar")));
        }
    }
}
