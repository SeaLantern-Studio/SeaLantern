use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::server::*;
use crate::services::server_log_pipeline;
use serde::{Deserialize, Serialize};

const DATA_FILE: &str = "sea_lantern_servers.json";
const RUN_PATH_MAP_FILE: &str = "sea_lantern_run_path_map.json";

/// 验证服务器名称，防止路径遍历攻击
/// 返回清理后的名称或错误信息
fn validate_server_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("服务器名称不能为空".to_string());
    }
    if trimmed.len() > 64 {
        return Err("服务器名称不能超过64个字符".to_string());
    }
    // 禁止的字符：路径分隔符和Windows保留字符
    let forbidden_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];
    for c in forbidden_chars {
        if trimmed.contains(c) {
            return Err(format!("服务器名称包含非法字符: '{}'", c));
        }
    }
    // 禁止以点开头（防止隐藏文件）或以空格/点结尾
    if trimmed.starts_with('.') || trimmed.ends_with('.') || trimmed.ends_with(' ') {
        return Err("服务器名称不能以点开头或结尾，也不能以空格结尾".to_string());
    }
    // 禁止Windows保留名称
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let upper = trimmed.to_uppercase();
    for r in reserved {
        if upper == r || upper.starts_with(&format!("{}.", r)) {
            return Err(format!("服务器名称不能使用系统保留名称: {}", r));
        }
    }
    Ok(trimmed.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunPathServerMapping {
    run_path: String,
    server_id: String,
    server_name: String,
    startup_mode: String,
    startup_file_path: Option<String>,
    custom_command: Option<String>,
    source_modpack_path: String,
    updated_at: u64,
}

#[derive(Clone, Copy, Debug)]
enum ManagedConsoleEncoding {
    Utf8,
    #[cfg(target_os = "windows")]
    Gbk,
}

impl ManagedConsoleEncoding {
    fn java_name(self) -> &'static str {
        match self {
            ManagedConsoleEncoding::Utf8 => "UTF-8",
            #[cfg(target_os = "windows")]
            ManagedConsoleEncoding::Gbk => "GBK",
        }
    }

    #[cfg(target_os = "windows")]
    fn cmd_code_page(self) -> &'static str {
        match self {
            ManagedConsoleEncoding::Utf8 => "65001",
            ManagedConsoleEncoding::Gbk => "936",
        }
    }
}

pub struct ServerManager {
    pub servers: Mutex<Vec<ServerInstance>>,
    pub processes: Mutex<HashMap<String, Child>>,
    pub stopping_servers: Mutex<HashSet<String>>,
    pub starting_servers: Mutex<HashSet<String>>,
    pub data_dir: Mutex<String>,
}

impl ServerManager {
    pub fn new() -> Self {
        let data_dir = get_data_dir();
        let servers = load_servers(&data_dir);
        ServerManager {
            servers: Mutex::new(servers),
            processes: Mutex::new(HashMap::new()),
            stopping_servers: Mutex::new(HashSet::new()),
            starting_servers: Mutex::new(HashSet::new()),
            data_dir: Mutex::new(data_dir),
        }
    }

    fn is_stopping(&self, id: &str) -> bool {
        self.stopping_servers
            .lock()
            .map(|stopping| stopping.contains(id))
            .unwrap_or(false)
    }

    fn mark_stopping(&self, id: &str) {
        if let Ok(mut stopping) = self.stopping_servers.lock() {
            stopping.insert(id.to_string());
        }
    }

    fn clear_stopping(&self, id: &str) {
        if let Ok(mut stopping) = self.stopping_servers.lock() {
            stopping.remove(id);
        }
    }

    fn is_starting(&self, id: &str) -> bool {
        self.starting_servers
            .lock()
            .map(|s| s.contains(id))
            .unwrap_or(false)
    }

    fn mark_starting(&self, id: &str) {
        if let Ok(mut s) = self.starting_servers.lock() {
            s.insert(id.to_string());
        }
    }

    pub fn clear_starting(&self, id: &str) {
        if let Ok(mut s) = self.starting_servers.lock() {
            s.remove(id);
        }
    }

    pub fn request_stop_server(&self, id: &str) -> Result<(), String> {
        if self.is_stopping(id) {
            return Ok(());
        }

        self.mark_stopping(id);
        let sid = id.to_string();
        std::thread::spawn(move || {
            let manager = super::global::server_manager();
            if let Err(err) = manager.stop_server(&sid) {
                let _ = server_log_pipeline::append_sealantern_log(
                    &sid,
                    &format!("[Sea Lantern] 停止失败: {}", err),
                );
                manager.clear_stopping(&sid);
            }
        });

        Ok(())
    }

    fn save(&self) {
        let servers = self.servers.lock().expect("servers lock poisoned");
        let data_dir = self.data_dir.lock().expect("data_dir lock poisoned");
        save_servers(&data_dir, &servers);
    }

    fn get_app_settings(&self) -> crate::models::settings::AppSettings {
        super::global::settings_manager().get()
    }

    fn build_managed_jvm_args(
        &self,
        server: &ServerInstance,
        settings: &crate::models::settings::AppSettings,
        console_encoding: ManagedConsoleEncoding,
    ) -> Vec<String> {
        let java_encoding = console_encoding.java_name();
        let mut args = vec![
            format!("-Xmx{}M", server.max_memory),
            format!("-Xms{}M", server.min_memory),
            format!("-Dfile.encoding={}", java_encoding),
            format!("-Dsun.stdout.encoding={}", java_encoding),
            format!("-Dsun.stderr.encoding={}", java_encoding),
        ];

        let jvm = settings.default_jvm_args.trim();
        if !jvm.is_empty() {
            args.extend(jvm.split_whitespace().map(|arg| arg.to_string()));
        }

        args.extend(server.jvm_args.iter().cloned());
        args
    }

    fn write_user_jvm_args(
        &self,
        server: &ServerInstance,
        settings: &crate::models::settings::AppSettings,
        console_encoding: ManagedConsoleEncoding,
    ) -> Result<(), String> {
        let args = self.build_managed_jvm_args(server, settings, console_encoding);
        let user_jvm_args_path = std::path::Path::new(&server.path).join("user_jvm_args.txt");
        let content = if args.is_empty() {
            String::new()
        } else {
            format!("{}\n", args.join("\n"))
        };

        std::fs::write(&user_jvm_args_path, content)
            .map_err(|e| format!("写入 user_jvm_args.txt 失败: {}", e))
    }

    pub fn create_server(&self, req: CreateServerRequest) -> Result<ServerInstance, String> {
        let server_name = validate_server_name(&req.name)?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs();
        let jar_path_obj = std::path::Path::new(&req.jar_path);
        let server_dir = jar_path_obj
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        let server = ServerInstance {
            id: id.clone(),
            name: server_name,
            core_type: req.core_type,
            core_version: String::new(),
            mc_version: req.mc_version,
            path: server_dir,
            jar_path: req.jar_path,
            startup_mode: normalize_startup_mode(&req.startup_mode).to_string(),
            custom_command: req.custom_command,
            java_path: req.java_path,
            max_memory: req.max_memory,
            min_memory: req.min_memory,
            jvm_args: Vec::new(),
            port: req.port,
            created_at: now,
            last_started_at: None,
        };
        self.servers
            .lock()
            .expect("servers lock poisoned")
            .push(server.clone());
        self.save();
        Ok(server)
    }

    pub fn import_server(&self, req: ImportServerRequest) -> Result<ServerInstance, String> {
        let server_name = validate_server_name(&req.name)?;
        let startup_mode = normalize_startup_mode(&req.startup_mode).to_string();
        let source_startup_file = std::path::Path::new(&req.jar_path);
        if !source_startup_file.exists() {
            return Err(format!("启动文件不存在: {}", req.jar_path));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let data_dir = self
            .data_dir
            .lock()
            .expect("data_dir lock poisoned")
            .clone();
        let servers_dir = std::path::Path::new(&data_dir).join("servers");
        let server_dir = servers_dir.join(&id);

        // 创建服务器目录
        std::fs::create_dir_all(&server_dir).map_err(|e| format!("无法创建服务器目录: {}", e))?;

        let startup_filename = source_startup_file
            .file_name()
            .ok_or_else(|| "无法获取启动文件名".to_string())?;

        // 获取启动文件所在目录，复制整个目录内容到 UUID 文件夹
        let source_server_dir = source_startup_file
            .parent()
            .ok_or_else(|| "无法获取启动文件所在目录".to_string())?;

        println!(
            "导入服务器：复制源目录 {} -> {}",
            source_server_dir.display(),
            server_dir.display()
        );
        copy_dir_recursive(source_server_dir, &server_dir)
            .map_err(|e| format!("复制服务端目录失败: {}", e))?;

        let dest_startup = server_dir.join(startup_filename);
        if !dest_startup.exists() {
            return Err(format!("复制后的启动文件不存在: {}", dest_startup.display()));
        }

        let server_properties_path = server_dir.join("server.properties");
        let mut port = req.port;

        // 如果 server.properties 已存在，读取其中的端口信息
        if server_properties_path.exists() {
            if let Ok(props) = crate::services::config_parser::read_properties(
                server_properties_path.to_str().unwrap_or_default(),
            ) {
                if let Some(port_str) = props.get("server-port") {
                    if let Ok(parsed_port) = port_str.parse::<u16>() {
                        port = parsed_port;
                        println!("从 server.properties 读取端口: {}", port);
                    }
                }
            }
        } else {
            // 如果不存在，创建默认的 server.properties
            let server_properties_content = format!(
                "# Minecraft server properties\n\
                 # Generated by SeaLantern\n\
                 server-port={}\n\
                 online-mode={}\n",
                req.port, req.online_mode
            );
            std::fs::write(&server_properties_path, server_properties_content)
                .map_err(|e| format!("创建 server.properties 失败: {}", e))?;
            println!("已创建 server.properties: {}", server_properties_path.display());
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs();

        // 检测核心类型
        let core_type = super::server_installer::detect_core_type(&dest_startup.to_string_lossy());
        println!("检测到核心类型: {}", core_type);

        let server = ServerInstance {
            id: id.clone(),
            name: server_name,
            core_type,
            core_version: String::new(),
            mc_version: "unknown".into(),
            path: server_dir.to_string_lossy().to_string(),
            jar_path: dest_startup.to_string_lossy().to_string(),
            startup_mode,
            custom_command: req.custom_command,
            java_path: req.java_path,
            max_memory: req.max_memory,
            min_memory: req.min_memory,
            jvm_args: Vec::new(),
            port,
            created_at: now,
            last_started_at: None,
        };

        self.servers
            .lock()
            .expect("servers lock poisoned")
            .push(server.clone());
        self.save();
        Ok(server)
    }

    pub fn import_modpack(&self, req: ImportModpackRequest) -> Result<ServerInstance, String> {
        let source_path = Path::new(&req.modpack_path);
        if !source_path.exists() {
            return Err(format!("整合包路径不存在: {}", req.modpack_path));
        }

        let id = uuid::Uuid::new_v4().to_string();

        let base_path = req.run_path.trim().to_string();
        if base_path.is_empty() {
            return Err("运行目录不能为空，请选择开服路径".to_string());
        }

        // 使用 UUID 前30位作为文件夹名称，确保唯一性
        let server_name = validate_server_name(&req.name)?;
        let folder_name = uuid::Uuid::new_v4().to_string().replace("-", "")[..30].to_string();
        let run_dir = PathBuf::from(&base_path).join(&folder_name);

        // 检查目标目录是否已存在
        if run_dir.exists() {
            return Err(format!(
                "目录已存在：{}，请更换启动项或选择其他路径",
                run_dir.to_string_lossy()
            ));
        }

        // 判断文件类型并处理
        let source_file_name = source_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("server.jar");
        let source_extension = source_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .unwrap_or_default();

        if source_path.is_file() {
            std::fs::create_dir_all(&run_dir).map_err(|e| format!("无法创建运行目录: {}", e))?;

            // jar 文件直接复制到目标目录
            if source_extension == "jar" {
                let target_jar = run_dir.join(source_file_name);
                std::fs::copy(source_path, &target_jar)
                    .map_err(|e| format!("复制 JAR 文件失败: {}", e))?;
            } else {
                // 其他压缩包解压
                super::server_installer::extract_modpack_archive(source_path, &run_dir)?;
            }
        } else if source_path.is_dir() {
            if !paths_equal(source_path, &run_dir) {
                if path_is_child_of(&run_dir, source_path) {
                    return Err("运行目录不能位于整合包源目录内部，请选择其他目录".to_string());
                }
                std::fs::create_dir_all(&run_dir)
                    .map_err(|e| format!("无法创建运行目录: {}", e))?;
                copy_dir_recursive(source_path, &run_dir)
                    .map_err(|e| format!("复制整合包文件失败: {}", e))?;
            }
        } else {
            return Err("无效的整合包路径".to_string());
        }

        let startup_mode = normalize_startup_mode(&req.startup_mode).to_string();
        let custom_command = req
            .custom_command
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let selected_core_type = req
            .core_type
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let selected_mc_version = req
            .mc_version
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let startup_file_path = if startup_mode == "custom" {
            None
        } else {
            let raw_path = req
                .startup_file_path
                .as_ref()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "未提供启动文件路径".to_string())?;
            Some(resolve_startup_file_path(source_path, &run_dir, raw_path)?)
        };

        let startup_path = startup_file_path.clone().unwrap_or_default();
        if startup_mode != "custom" && !Path::new(&startup_path).exists() {
            return Err(format!("启动文件不存在: {}", startup_path));
        }

        if startup_mode == "custom" && custom_command.is_none() {
            return Err("自定义启动命令不能为空".to_string());
        }

        let data_dir = self
            .data_dir
            .lock()
            .expect("data_dir lock poisoned")
            .clone();

        upsert_run_path_mapping(
            &data_dir,
            RunPathServerMapping {
                run_path: run_dir.to_string_lossy().to_string(),
                server_id: id.clone(),
                server_name: server_name.clone(),
                startup_mode: startup_mode.clone(),
                startup_file_path: startup_file_path.clone(),
                custom_command: custom_command.clone(),
                source_modpack_path: req.modpack_path.clone(),
                updated_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("time went backwards")
                    .as_secs(),
            },
        )?;

        let mut port = req.port;
        let server_properties_path = run_dir.join("server.properties");
        if server_properties_path.exists() {
            if let Ok(props) = crate::services::config_parser::read_properties(
                server_properties_path.to_str().unwrap_or_default(),
            ) {
                if let Some(port_str) = props.get("server-port") {
                    if let Ok(parsed_port) = port_str.parse::<u16>() {
                        port = parsed_port;
                    }
                }
            }
            let mut updates = HashMap::new();
            updates.insert("online-mode".to_string(), req.online_mode.to_string());
            crate::services::config_parser::write_properties(
                server_properties_path.to_str().unwrap_or_default(),
                &updates,
            )
            .map_err(|e| format!("更新 server.properties 失败: {}", e))?;
        } else {
            let content = format!(
                "# Minecraft server properties\n# Generated by SeaLantern\nserver-port={}\nonline-mode={}\n",
                req.port, req.online_mode
            );
            std::fs::write(&server_properties_path, content)
                .map_err(|e| format!("创建 server.properties 失败: {}", e))?;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs();
        let detected_core_type = if startup_mode == "custom" {
            "modpack".to_string()
        } else {
            super::server_installer::detect_core_type(&startup_path)
        };
        let core_type = selected_core_type.unwrap_or(detected_core_type);
        let mc_version = selected_mc_version.unwrap_or_else(|| "unknown".to_string());

        let server = ServerInstance {
            id: id.clone(),
            name: server_name,
            core_type,
            core_version: String::new(),
            mc_version,
            path: run_dir.to_string_lossy().to_string(),
            jar_path: startup_path,
            startup_mode,
            custom_command,
            java_path: req.java_path,
            max_memory: req.max_memory,
            min_memory: req.min_memory,
            jvm_args: Vec::new(),
            port,
            created_at: now,
            last_started_at: None,
        };

        println!(
            "创建服务器实例: id={}, path={}, startup_path={}",
            server.id, server.path, server.jar_path
        );

        self.servers
            .lock()
            .expect("servers lock poisoned")
            .push(server.clone());
        self.save();
        Ok(server)
    }

    pub fn add_existing_server(
        &self,
        req: AddExistingServerRequest,
    ) -> Result<ServerInstance, String> {
        let server_name = validate_server_name(&req.name)?;
        let server_path = std::path::Path::new(&req.server_path);

        // 验证路径存在且是目录
        if !server_path.exists() {
            return Err(format!("服务器目录不存在: {}", req.server_path));
        }
        if !server_path.is_dir() {
            return Err("所选路径不是文件夹".to_string());
        }

        // 检查目录权限
        let test_file = server_path.join(".sl_permission_test");
        if std::fs::write(&test_file, "").is_err() {
            return Err("无法写入服务器目录，请检查权限".to_string());
        }
        let _ = std::fs::remove_file(&test_file);

        let requested_mode = normalize_startup_mode(&req.startup_mode).to_string();
        let (jar_path, startup_mode, custom_command) = if requested_mode == "custom" {
            let command = req
                .custom_command
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "自定义启动命令不能为空".to_string())?;
            (String::new(), requested_mode, Some(command))
        } else if let Some(ref exec_path) = req.executable_path {
            let path = std::path::Path::new(exec_path);
            if !path.exists() {
                return Err(format!("选择的可执行文件不存在: {}", exec_path));
            }
            (exec_path.clone(), detect_startup_mode_from_path(path), None)
        } else {
            let (path, mode) = find_server_executable(server_path)?;
            (path, mode, None)
        };

        // 尝试从server.properties读取端口
        let mut port = req.port;
        let server_properties_path = server_path.join("server.properties");
        if server_properties_path.exists() {
            if let Ok(props) = crate::services::config_parser::read_properties(
                server_properties_path.to_str().unwrap_or_default(),
            ) {
                if let Some(port_str) = props.get("server-port") {
                    if let Ok(parsed_port) = port_str.parse::<u16>() {
                        port = parsed_port;
                        println!("从 server.properties 读取端口: {}", port);
                    }
                }
            }
        }

        // 检测服务端类型
        let core_type = if startup_mode == "custom" {
            "Unknown".to_string()
        } else {
            super::server_installer::detect_core_type(&jar_path)
        };
        println!("检测到核心类型: {}", core_type);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs();

        let id = uuid::Uuid::new_v4().to_string();

        let server = ServerInstance {
            id: id.clone(),
            name: server_name,
            core_type,
            core_version: String::new(),
            mc_version: "unknown".into(),
            path: req.server_path,
            jar_path,
            startup_mode,
            custom_command,
            java_path: req.java_path,
            max_memory: req.max_memory,
            min_memory: req.min_memory,
            jvm_args: Vec::new(),
            port,
            created_at: now,
            last_started_at: None,
        };

        self.servers
            .lock()
            .expect("servers lock poisoned")
            .push(server.clone());
        self.save();
        Ok(server)
    }

    pub fn start_server(&self, id: &str) -> Result<(), String> {
        let server = {
            let servers = self.servers.lock().expect("servers lock poisoned");
            servers
                .iter()
                .find(|s| s.id == id)
                .ok_or_else(|| "未找到服务器".to_string())?
                .clone()
        };

        println!(
            "准备启动服务器: id={}, name={}, startup_mode={}, startup_path={}, java_path={}",
            server.id, server.name, server.startup_mode, server.jar_path, server.java_path
        );

        {
            let mut procs = self.processes.lock().expect("processes lock poisoned");
            if let Some(child) = procs.get_mut(id) {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        procs.remove(id);
                        server_log_pipeline::shutdown_writer(id);
                    } // Dead process, clean up
                    Ok(None) => return Err("服务器已在运行中".to_string()),
                    Err(_) => {
                        procs.remove(id);
                        server_log_pipeline::shutdown_writer(id);
                    }
                }
            }
        }

        let settings = self.get_app_settings();
        if settings.auto_accept_eula {
            let eula = std::path::Path::new(&server.path).join("eula.txt");
            let _ = std::fs::write(&eula, "# Auto-accepted by Sea Lantern\neula=true\n");
        }

        //预处理脚本
        #[cfg(target_os = "windows")]
        {
            let preload_script = std::path::Path::new(&server.path).join("preload.bat");
            if preload_script.exists() {
                println!("发现预加载脚本: {:?}", preload_script);
                let _ = server_log_pipeline::append_sealantern_log(
                    id,
                    "[preload] 开始执行预加载脚本...",
                );

                let mut cmd = std::process::Command::new("cmd");
                cmd.args(["/c", preload_script.to_str().unwrap_or("preload.bat")])
                    .current_dir(&server.path);

                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                cmd.creation_flags(CREATE_NO_WINDOW);

                match cmd.output() {
                    Ok(result) => {
                        if result.status.success() {
                            println!("preload.bat 执行成功");
                            if !result.stdout.is_empty() {
                                let output = String::from_utf8_lossy(&result.stdout);
                                for line in output.lines() {
                                    let _ = server_log_pipeline::append_sealantern_log(
                                        id,
                                        &format!("[preload] {}", line),
                                    );
                                }
                            }
                            let _ = server_log_pipeline::append_sealantern_log(
                                id,
                                "[preload] 预加载脚本执行成功",
                            );
                        } else {
                            let error = String::from_utf8_lossy(&result.stderr);
                            println!("preload.bat 执行失败: {}", error);
                            let _ = server_log_pipeline::append_sealantern_log(
                                id,
                                &format!("[preload] 执行失败: {}", error),
                            );
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("执行 preload.bat 失败: {}", e);
                        println!("{}", error_msg);
                        let _ = server_log_pipeline::append_sealantern_log(
                            id,
                            &format!("[preload] {}", error_msg),
                        );
                    }
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let preload_script = std::path::Path::new(&server.path).join("preload.sh");
            if preload_script.exists() {
                println!("发现预加载脚本: {:?}", preload_script);
                let _ = server_log_pipeline::append_sealantern_log(
                    id,
                    "[preload] 开始执行预加载脚本...",
                );

                match std::process::Command::new("sh")
                    .arg(&preload_script)
                    .current_dir(&server.path)
                    .output()
                {
                    Ok(result) => {
                        if result.status.success() {
                            println!("preload.sh 执行成功");
                            if !result.stdout.is_empty() {
                                let output = String::from_utf8_lossy(&result.stdout);
                                for line in output.lines() {
                                    let _ = server_log_pipeline::append_sealantern_log(
                                        id,
                                        &format!("[preload] {}", line),
                                    );
                                }
                            }
                            let _ = server_log_pipeline::append_sealantern_log(
                                id,
                                "[preload] 预加载脚本执行成功",
                            );
                        } else {
                            let error = String::from_utf8_lossy(&result.stderr);
                            println!("preload.sh 执行失败: {}", error);
                            let _ = server_log_pipeline::append_sealantern_log(
                                id,
                                &format!("[preload] 执行失败: {}", error),
                            );
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("执行 preload.sh 失败: {}", e);
                        println!("{}", error_msg);
                        let _ = server_log_pipeline::append_sealantern_log(
                            id,
                            &format!("[preload] {}", error_msg),
                        );
                    }
                }
            }
        }

        let startup_mode = normalize_startup_mode(&server.startup_mode);
        let startup_path_obj = std::path::Path::new(&server.jar_path);
        let managed_console_encoding = if startup_mode == "custom" {
            ManagedConsoleEncoding::Utf8
        } else {
            resolve_managed_console_encoding(startup_mode, startup_path_obj)
        };

        if startup_mode == "bat" || startup_mode == "sh" || startup_mode == "ps1" {
            if let Some(major_version) = detect_java_major_version(&server.java_path) {
                if major_version < 9 {
                    return Err(format!(
                        "当前 Java 版本 {} 不支持 @user_jvm_args.txt 参数文件语法，请改用 Java 9+（NeoForge 建议 Java 21）",
                        major_version
                    ));
                }
            }
        }

        let java_path_obj = std::path::Path::new(&server.java_path);
        let java_bin_dir = java_path_obj
            .parent()
            .ok_or_else(|| format!("Java 路径无效，缺少 bin 目录: {}", server.java_path))?;
        let java_home_dir = java_bin_dir.parent().unwrap_or(java_bin_dir);
        let java_bin_dir_str = java_bin_dir.to_string_lossy().to_string();
        let java_home_dir_str = java_home_dir.to_string_lossy().to_string();
        let startup_filename = startup_path_obj
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| server.jar_path.clone());

        let starter_installer_url = if startup_mode == "starter" {
            let detected_core_type = super::server_installer::detect_core_type(&server.jar_path);
            let core_key =
                super::server_installer::CoreType::normalize_to_api_core_key(&server.core_type)
                    .or_else(|| {
                        super::server_installer::CoreType::normalize_to_api_core_key(
                            &detected_core_type,
                        )
                    })
                    .ok_or_else(|| {
                        format!(
                            "无法识别 Starter 核心类型：{}",
                            if server.core_type.trim().is_empty() {
                                detected_core_type
                            } else {
                                server.core_type.clone()
                            }
                        )
                    })?;

            let mc_version = server.mc_version.trim();
            if mc_version.is_empty() || mc_version.eq_ignore_ascii_case("unknown") {
                return Err("Starter 启动需要 MC 版本，请在步骤三中选择后再创建服务器".to_string());
            }

            let (installer_url, installer_sha256) =
                super::starter_installer_links::fetch_starter_installer_url(&core_key, mc_version)?;
            if let Some(sha256) = installer_sha256 {
                let _ = server_log_pipeline::append_sealantern_log(
                    id,
                    &format!(
                        "[Sea Lantern] Starter 安装器: core={}, version={}, sha256={}",
                        core_key, mc_version, sha256
                    ),
                );
            }
            Some(installer_url)
        } else {
            None
        };

        let mut cmd = match startup_mode {
            "custom" => {
                let custom_command = server
                    .custom_command
                    .as_ref()
                    .map(|value| value.trim())
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| "自定义启动命令为空".to_string())?;

                #[cfg(target_os = "windows")]
                {
                    let mut custom_cmd = Command::new("cmd");
                    custom_cmd.arg("/d");
                    custom_cmd.arg("/c");
                    custom_cmd.arg(custom_command);
                    custom_cmd.env("JAVA_HOME", &java_home_dir_str);
                    let existing_path = std::env::var("PATH").unwrap_or_default();
                    let path_value = if existing_path.is_empty() {
                        java_bin_dir_str.clone()
                    } else {
                        format!("{};{}", java_bin_dir_str, existing_path)
                    };
                    custom_cmd.env("PATH", path_value);
                    custom_cmd
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let mut custom_cmd = Command::new("sh");
                    custom_cmd.arg("-c");
                    custom_cmd.arg(custom_command);
                    custom_cmd.env("JAVA_HOME", &java_home_dir_str);
                    let existing_path = std::env::var("PATH").unwrap_or_default();
                    let path_value = if existing_path.is_empty() {
                        java_bin_dir_str.clone()
                    } else {
                        format!("{}:{}", java_bin_dir_str, existing_path)
                    };
                    custom_cmd.env("PATH", path_value);
                    custom_cmd
                }
            }
            "bat" => {
                self.write_user_jvm_args(&server, &settings, managed_console_encoding)?;

                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;

                    let mut bat_cmd = Command::new("cmd");
                    let code_page = managed_console_encoding.cmd_code_page();
                    let launch_command = format!(
                        "chcp {}>nul & set \"JAVA_HOME={}\" & set \"PATH={};%PATH%\" & call \"{}\" nogui",
                        code_page,
                        escape_cmd_arg(&java_home_dir_str),
                        escape_cmd_arg(&java_bin_dir_str),
                        escape_cmd_arg(&startup_filename)
                    );
                    bat_cmd.arg("/d");
                    bat_cmd.arg("/c");
                    bat_cmd.raw_arg(&launch_command);
                    bat_cmd
                }
                #[cfg(not(target_os = "windows"))]
                {
                    return Err("BAT 启动方式仅支持 Windows".to_string());
                }
            }
            "sh" => {
                self.write_user_jvm_args(&server, &settings, managed_console_encoding)?;
                let mut sh_cmd = Command::new("sh");
                sh_cmd.arg(&startup_filename);
                sh_cmd.arg("nogui");
                sh_cmd.env("JAVA_HOME", &java_home_dir_str);
                let existing_path = std::env::var("PATH").unwrap_or_default();
                let path_value = if existing_path.is_empty() {
                    java_bin_dir_str.clone()
                } else {
                    format!("{}:{}", java_bin_dir_str, existing_path)
                };
                sh_cmd.env("PATH", path_value);
                sh_cmd
            }
            "ps1" => {
                self.write_user_jvm_args(&server, &settings, managed_console_encoding)?;
                #[cfg(target_os = "windows")]
                {
                    let mut ps_cmd = Command::new("powershell");
                    ps_cmd.arg("-NoProfile");
                    ps_cmd.arg("-NonInteractive");
                    ps_cmd.arg("-ExecutionPolicy");
                    ps_cmd.arg("Bypass");
                    ps_cmd.arg("-File");
                    ps_cmd.arg(&startup_filename);
                    ps_cmd.arg("nogui");
                    ps_cmd.env("JAVA_HOME", &java_home_dir_str);
                    let existing_path = std::env::var("PATH").unwrap_or_default();
                    let path_value = if existing_path.is_empty() {
                        java_bin_dir_str.clone()
                    } else {
                        format!("{};{}", java_bin_dir_str, existing_path)
                    };
                    ps_cmd.env("PATH", path_value);
                    ps_cmd
                }
                #[cfg(not(target_os = "windows"))]
                {
                    return Err("PS1 启动方式仅支持 Windows".to_string());
                }
            }
            "starter" => {
                let installer_url = starter_installer_url
                    .clone()
                    .ok_or_else(|| "Starter 安装器下载链接为空".to_string())?;
                let mut starter_cmd = Command::new(&server.java_path);
                for arg in self.build_managed_jvm_args(&server, &settings, managed_console_encoding)
                {
                    starter_cmd.arg(arg);
                }
                starter_cmd.arg("-jar");
                starter_cmd.arg(&startup_filename);
                starter_cmd.arg("nogui");
                starter_cmd.arg("--installer");
                starter_cmd.arg(installer_url);
                starter_cmd
            }
            "jar" => {
                let mut jar_cmd = Command::new(&server.java_path);
                for arg in self.build_managed_jvm_args(&server, &settings, managed_console_encoding)
                {
                    jar_cmd.arg(arg);
                }
                jar_cmd.arg("-jar");
                jar_cmd.arg(&startup_filename);
                jar_cmd.arg("nogui");
                jar_cmd
            }
            _ => {
                let mut jar_cmd = Command::new(&server.java_path);
                for arg in self.build_managed_jvm_args(&server, &settings, managed_console_encoding)
                {
                    jar_cmd.arg(arg);
                }
                jar_cmd.arg("-jar");
                jar_cmd.arg(&startup_filename);
                jar_cmd.arg("nogui");
                jar_cmd
            }
        };

        let command_for_log = format_command_for_log(&cmd);
        let _ = server_log_pipeline::append_sealantern_log(
            id,
            &format!("[Sea Lantern] 启动命令: {}", command_for_log),
        );

        cmd.current_dir(&server.path);

        server_log_pipeline::init_db(Path::new(&server.path))?;

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::piped());

        // 隐藏控制台窗口
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = cmd.spawn().map_err(|e| format!("启动失败: {}", e))?;
        println!("Java进程已启动，PID: {:?}", child.id());

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        self.processes
            .lock()
            .expect("processes lock poisoned")
            .insert(id.to_string(), child);
        self.mark_starting(id);

        {
            let mut servers = self.servers.lock().expect("servers lock poisoned");
            if let Some(s) = servers.iter_mut().find(|s| s.id == id) {
                s.last_started_at = Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("time went backwards")
                        .as_secs(),
                );
            }
        }
        self.save();
        let _ = server_log_pipeline::append_sealantern_log(id, "[Sea Lantern] 服务器启动中...");

        if let Some(stdout) = stdout {
            server_log_pipeline::spawn_server_output_reader(id.to_string(), stdout);
        }
        if let Some(stderr) = stderr {
            server_log_pipeline::spawn_server_output_reader(id.to_string(), stderr);
        }

        Ok(())
    }

    pub fn stop_server(&self, id: &str) -> Result<(), String> {
        // 日志 Writer 生命周期说明：
        // 1) 停服流程中“最后一条 Sea Lantern 提示日志”要先入队，随后再 shutdown_writer。
        //    这样可以保证提示日志也被刷盘，不会因为先关 Writer 而丢失。
        // 2) shutdown_writer 会触发 writer 线程 flush+join，确保 SQLite 句柄被释放。
        //    这对 Windows 很关键，可避免删除目录或外部工具读取 DB 时遇到句柄占用。
        // 3) 所有 return 分支都要覆盖 shutdown_writer，避免异常路径漏清理。
        // Check if actually running first
        let is_running = {
            let mut procs = self.processes.lock().expect("processes lock poisoned");
            if let Some(child) = procs.get_mut(id) {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        procs.remove(id);
                        server_log_pipeline::shutdown_writer(id);
                        false
                    }
                    Ok(None) => true,
                    Err(_) => {
                        procs.remove(id);
                        server_log_pipeline::shutdown_writer(id);
                        false
                    }
                }
            } else {
                false
            }
        };

        if !is_running {
            self.clear_stopping(id);
            let _ = server_log_pipeline::append_sealantern_log(id, "[Sea Lantern] 服务器未运行");
            server_log_pipeline::shutdown_writer(id);
            return Ok(());
        }

        let _ = server_log_pipeline::append_sealantern_log(id, "[Sea Lantern] 正在发送停止命令...");
        let _ = self.send_command(id, "stop");

        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let mut procs = self.processes.lock().expect("processes lock poisoned");
            if let Some(child) = procs.get_mut(id) {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        procs.remove(id);
                        self.clear_stopping(id);
                        let _ = server_log_pipeline::append_sealantern_log(
                            id,
                            "[Sea Lantern] 服务器已正常停止",
                        );
                        server_log_pipeline::shutdown_writer(id);
                        return Ok(());
                    }
                    Ok(None) => {} // Still running
                    Err(_) => {
                        procs.remove(id);
                        self.clear_stopping(id);
                        server_log_pipeline::shutdown_writer(id);
                        return Ok(());
                    }
                }
            } else {
                self.clear_stopping(id);
                let _ =
                    server_log_pipeline::append_sealantern_log(id, "[Sea Lantern] 服务器已停止");
                server_log_pipeline::shutdown_writer(id);
                return Ok(());
            }
        }

        let mut procs = self.processes.lock().expect("processes lock poisoned");
        if let Some(mut child) = procs.remove(id) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = server_log_pipeline::append_sealantern_log(
                id,
                "[Sea Lantern] 服务器超时，已强制终止",
            );
        }
        server_log_pipeline::shutdown_writer(id);
        self.clear_stopping(id);
        Ok(())
    }

    pub fn send_command(&self, id: &str, command: &str) -> Result<(), String> {
        let mut procs = self.processes.lock().expect("processes lock poisoned");
        let child = procs
            .get_mut(id)
            .ok_or_else(|| "服务器未运行".to_string())?;
        if let Some(ref mut stdin) = child.stdin {
            writeln!(stdin, "{}", command).map_err(|e| format!("发送失败: {}", e))?;
            stdin.flush().map_err(|e| format!("发送失败: {}", e))?;
        }
        Ok(())
    }

    pub fn get_server_list(&self) -> Vec<ServerInstance> {
        self.servers.lock().expect("servers lock poisoned").clone()
    }

    pub fn get_server_status(&self, id: &str) -> ServerStatusInfo {
        let mut procs = self.processes.lock().expect("processes lock poisoned");
        let is_running = if let Some(child) = procs.get_mut(id) {
            match child.try_wait() {
                Ok(Some(_)) => {
                    procs.remove(id);
                    server_log_pipeline::shutdown_writer(id);
                    self.clear_starting(id);
                    false
                }
                Ok(None) => true,
                Err(_) => {
                    procs.remove(id);
                    server_log_pipeline::shutdown_writer(id);
                    self.clear_starting(id);
                    false
                }
            }
        } else {
            false
        };
        ServerStatusInfo {
            id: id.to_string(),
            status: if self.is_stopping(id) {
                ServerStatus::Stopping
            } else if is_running && self.is_starting(id) {
                ServerStatus::Starting
            } else if is_running {
                ServerStatus::Running
            } else {
                ServerStatus::Stopped
            },
            pid: None,
            uptime: None,
        }
    }

    pub fn delete_server(&self, id: &str) -> Result<(), String> {
        {
            let procs = self.processes.lock().expect("processes lock poisoned");
            if procs.contains_key(id) {
                drop(procs);
                let _ = self.stop_server(id);
            }
        }

        server_log_pipeline::shutdown_writer(id);

        let server_path = {
            let servers = self.servers.lock().expect("servers lock poisoned");
            servers.iter().find(|s| s.id == id).map(|s| s.path.clone())
        };
        if let Some(path) = server_path {
            let dir = std::path::Path::new(&path);
            if dir.exists() {
                std::fs::remove_dir_all(dir).map_err(|e| format!("删除服务器目录失败: {}", e))?;
            }
        }

        self.servers
            .lock()
            .expect("servers lock poisoned")
            .retain(|s| s.id != id);
        let data_dir = self
            .data_dir
            .lock()
            .expect("data_dir lock poisoned")
            .clone();
        remove_run_path_mapping(&data_dir, id);
        self.save();
        Ok(())
    }

    pub fn get_running_server_ids(&self) -> Vec<String> {
        let procs = self.processes.lock().expect("processes lock poisoned");
        procs.keys().cloned().collect()
    }

    pub fn update_server_name(&self, id: &str, new_name: &str) -> Result<(), String> {
        let validated_name = validate_server_name(new_name)?;
        let mut servers = self.servers.lock().expect("servers lock poisoned");
        if let Some(server) = servers.iter_mut().find(|s| s.id == id) {
            server.name = validated_name;
            drop(servers);
            self.save();
            Ok(())
        } else {
            Err("未找到服务器".to_string())
        }
    }

    pub fn stop_all_servers(&self) {
        let ids: Vec<String> = self
            .processes
            .lock()
            .expect("processes lock poisoned")
            .keys()
            .cloned()
            .collect();
        for id in ids {
            let _ = self.stop_server(&id);
        }
    }
}

#[cfg(target_os = "windows")]
fn escape_cmd_arg(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '^' => out.push_str("^^"),
            '&' => out.push_str("^&"),
            '|' => out.push_str("^|"),
            '<' => out.push_str("^<"),
            '>' => out.push_str("^>"),
            '(' => out.push_str("^("),
            ')' => out.push_str("^)"),
            '%' => out.push_str("%%"),
            '"' => out.push_str("\"\""),
            other => out.push(other),
        }
    }
    out
}

fn get_data_dir() -> String {
    // 使用统一的应用数据目录，确保 MSI 安装时数据存储在 %AppData%
    crate::utils::path::get_or_create_app_data_dir()
}

fn normalize_startup_mode(mode: &str) -> &str {
    match mode.to_ascii_lowercase().as_str() {
        "starter" => "starter",
        "bat" => "bat",
        "sh" => "sh",
        "ps1" => "ps1",
        "custom" => "custom",
        _ => "jar",
    }
}

fn detect_startup_mode_from_path(path: &Path) -> String {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "bat" => "bat".to_string(),
        "sh" => "sh".to_string(),
        "ps1" => "ps1".to_string(),
        _ => "jar".to_string(),
    }
}

fn find_server_executable(server_path: &Path) -> Result<(String, String), String> {
    let preferred_scripts = [
        "start.bat",
        "run.bat",
        "launch.bat",
        "start.sh",
        "run.sh",
        "launch.sh",
        "start.ps1",
        "run.ps1",
        "launch.ps1",
    ];

    for script in preferred_scripts {
        let script_path = server_path.join(script);
        if script_path.exists() {
            let mode = detect_startup_mode_from_path(&script_path);
            return Ok((script_path.to_string_lossy().to_string(), mode));
        }
    }

    if let Ok(jar_path) = super::server_installer::find_server_jar(server_path) {
        return Ok((jar_path, "jar".to_string()));
    }

    let entries =
        std::fs::read_dir(server_path).map_err(|e| format!("无法读取服务器目录: {}", e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .unwrap_or_default();
        if extension != "jar" && extension != "bat" && extension != "sh" && extension != "ps1" {
            continue;
        }

        let mode = detect_startup_mode_from_path(&path);
        return Ok((path.to_string_lossy().to_string(), mode));
    }

    Err("未找到可用的启动文件（.jar/.bat/.sh/.ps1）".to_string())
}

fn resolve_managed_console_encoding(
    startup_mode: &str,
    startup_path: &std::path::Path,
) -> ManagedConsoleEncoding {
    #[cfg(target_os = "windows")]
    {
        if startup_mode == "bat" || startup_mode == "ps1" {
            return detect_windows_batch_encoding(startup_path);
        }
    }

    let _ = startup_mode;
    let _ = startup_path;
    ManagedConsoleEncoding::Utf8
}

#[cfg(target_os = "windows")]
fn detect_windows_batch_encoding(startup_path: &std::path::Path) -> ManagedConsoleEncoding {
    let bytes = match std::fs::read(startup_path) {
        Ok(bytes) => bytes,
        Err(_) => return ManagedConsoleEncoding::Utf8,
    };

    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) || std::str::from_utf8(&bytes).is_ok() {
        ManagedConsoleEncoding::Utf8
    } else {
        ManagedConsoleEncoding::Gbk
    }
}

fn parse_java_major_version(raw_version: &str) -> Option<u32> {
    let version = raw_version.trim().trim_matches('"');
    let mut parts = version.split('.');
    let first = parts.next()?.parse::<u32>().ok()?;
    if first == 1 {
        parts.next()?.parse::<u32>().ok()
    } else {
        Some(first)
    }
}

fn detect_java_major_version(java_path: &str) -> Option<u32> {
    let output = Command::new(java_path).arg("-version").output().ok()?;
    let text = if output.stderr.is_empty() {
        decode_console_bytes(&output.stdout)
    } else {
        decode_console_bytes(&output.stderr)
    };

    for line in text.lines() {
        if line.contains("version") {
            let mut segments = line.split('"');
            let _ = segments.next();
            if let Some(version_text) = segments.next() {
                return parse_java_major_version(version_text);
            }
        }
    }

    None
}

fn format_command_for_log(command: &Command) -> String {
    let mut parts = Vec::new();

    let env_parts = command
        .get_envs()
        .filter_map(|(key, value)| {
            value.map(|v| {
                format!(
                    "{}={}",
                    key.to_string_lossy(),
                    quote_command_fragment(&v.to_string_lossy())
                )
            })
        })
        .collect::<Vec<String>>();
    if !env_parts.is_empty() {
        parts.push(format!("env {{{}}}", env_parts.join(", ")));
    }

    parts.push(quote_command_fragment(&command.get_program().to_string_lossy()));
    parts.extend(
        command
            .get_args()
            .map(|arg| quote_command_fragment(&arg.to_string_lossy())),
    );

    parts.join(" ")
}

fn quote_command_fragment(value: &str) -> String {
    let requires_quotes = value.is_empty()
        || value.chars().any(|ch| ch.is_whitespace())
        || value.contains('"')
        || value.contains('\'')
        || value.contains(';')
        || value.contains('&')
        || value.contains('|');

    if !requires_quotes {
        return value.to_string();
    }

    format!("\"{}\"", value.replace('"', "\\\""))
}

fn decode_console_bytes(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_string();
    }

    #[cfg(target_os = "windows")]
    {
        let (decoded, _, _) = encoding_rs::GBK.decode(bytes);
        decoded.into_owned()
    }
    #[cfg(not(target_os = "windows"))]
    {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

fn normalize_path_for_compare(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string()
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    normalize_path_for_compare(left) == normalize_path_for_compare(right)
}

fn normalize_absolute_path_for_compare(path: &Path) -> Option<String> {
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };

    let mut normalized = PathBuf::new();
    for component in absolute_path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }

    let normalized = normalize_path_for_compare(&normalized);

    #[cfg(target_os = "windows")]
    {
        Some(normalized.to_ascii_lowercase())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Some(normalized)
    }
}

fn path_is_child_of(candidate: &Path, parent: &Path) -> bool {
    let Some(candidate_norm) = normalize_absolute_path_for_compare(candidate) else {
        return false;
    };
    let Some(parent_norm) = normalize_absolute_path_for_compare(parent) else {
        return false;
    };

    candidate_norm.starts_with(&(parent_norm + "/"))
}

fn resolve_startup_file_path(
    source_path: &Path,
    run_dir: &Path,
    startup_file_path: &str,
) -> Result<String, String> {
    let startup_path = PathBuf::from(startup_file_path);
    if startup_path.is_relative() {
        return Ok(run_dir.join(&startup_path).to_string_lossy().to_string());
    }

    // 如果源是 jar 文件，启动项就是 jar 本身，复制后在 run_dir 中
    if source_path.is_file() {
        let source_file_name = source_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("server.jar");
        return Ok(run_dir.join(source_file_name).to_string_lossy().to_string());
    }

    if source_path.is_dir() {
        let source_norm = normalize_path_for_compare(source_path);
        let startup_norm = normalize_path_for_compare(&startup_path);
        if startup_norm.starts_with(&(source_norm.clone() + "/")) {
            if let Ok(relative) = startup_path.strip_prefix(source_path) {
                return Ok(run_dir.join(relative).to_string_lossy().to_string());
            }
        }
    }

    Err(format!("无法安全映射启动文件路径，请重新扫描后重试: {}", startup_file_path))
}

fn load_run_path_mappings(dir: &str) -> Vec<RunPathServerMapping> {
    let path = Path::new(dir).join(RUN_PATH_MAP_FILE);
    if !path.exists() {
        return Vec::new();
    }

    std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str::<Vec<RunPathServerMapping>>(&content).ok())
        .unwrap_or_default()
}

fn save_run_path_mappings(dir: &str, mappings: &[RunPathServerMapping]) -> Result<(), String> {
    let path = Path::new(dir).join(RUN_PATH_MAP_FILE);
    let json = serde_json::to_string_pretty(mappings)
        .map_err(|e| format!("序列化运行路径映射失败: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("写入运行路径映射失败: {}", e))
}

fn upsert_run_path_mapping(dir: &str, mapping: RunPathServerMapping) -> Result<(), String> {
    let mut mappings = load_run_path_mappings(dir);
    mappings
        .retain(|item| item.server_id != mapping.server_id && item.run_path != mapping.run_path);
    mappings.push(mapping);
    save_run_path_mappings(dir, &mappings)
}

fn remove_run_path_mapping(dir: &str, server_id: &str) {
    let mut mappings = load_run_path_mappings(dir);
    let before = mappings.len();
    mappings.retain(|item| item.server_id != server_id);
    if mappings.len() == before {
        return;
    }

    let _ = save_run_path_mappings(dir, &mappings);
}

fn load_servers(dir: &str) -> Vec<ServerInstance> {
    let p = std::path::Path::new(dir).join(DATA_FILE);
    if !p.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(&p)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}
fn save_servers(dir: &str, servers: &[ServerInstance]) {
    let p = std::path::Path::new(dir).join(DATA_FILE);
    if let Ok(j) = serde_json::to_string_pretty(servers) {
        let _ = std::fs::write(&p, j);
    }
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            // 若遍历到当前复制目标目录本身，直接跳过，作为额外兜底保护。
            if paths_equal(&src_path, dst) {
                continue;
            }
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
