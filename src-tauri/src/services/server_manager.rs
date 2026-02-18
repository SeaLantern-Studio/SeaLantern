use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::server::*;

const DATA_FILE: &str = "sea_lantern_servers.json";

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
    pub logs: Mutex<HashMap<String, Vec<String>>>,
    pub data_dir: Mutex<String>,
}

impl ServerManager {
    pub fn new() -> Self {
        let data_dir = get_data_dir();
        let servers = load_servers(&data_dir);
        let mut logs_map = HashMap::new();
        for s in &servers {
            logs_map.insert(s.id.clone(), Vec::new());
        }
        ServerManager {
            servers: Mutex::new(servers),
            processes: Mutex::new(HashMap::new()),
            stopping_servers: Mutex::new(HashSet::new()),
            logs: Mutex::new(logs_map),
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

    pub fn request_stop_server(&self, id: &str) -> Result<(), String> {
        if self.is_stopping(id) {
            return Ok(());
        }

        self.mark_stopping(id);
        let sid = id.to_string();
        std::thread::spawn(move || {
            let manager = super::global::server_manager();
            if let Err(err) = manager.stop_server(&sid) {
                manager.append_log(&sid, &format!("[Sea Lantern] 停止失败: {}", err));
                manager.clear_stopping(&sid);
            }
        });

        Ok(())
    }

    fn save(&self) {
        let servers = self.servers.lock().unwrap();
        let data_dir = self.data_dir.lock().unwrap();
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
        let id = uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let jar_path_obj = std::path::Path::new(&req.jar_path);
        let server_dir = jar_path_obj
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        let server = ServerInstance {
            id: id.clone(),
            name: req.name,
            core_type: req.core_type,
            core_version: String::new(),
            mc_version: req.mc_version,
            path: server_dir,
            jar_path: req.jar_path,
            startup_mode: normalize_startup_mode(&req.startup_mode).to_string(),
            java_path: req.java_path,
            max_memory: req.max_memory,
            min_memory: req.min_memory,
            jvm_args: Vec::new(),
            port: req.port,
            created_at: now,
            last_started_at: None,
            commands: Vec::new(),
        };
        self.servers.lock().unwrap().push(server.clone());
        self.logs.lock().unwrap().insert(id, Vec::new());
        self.save();
        Ok(server)
    }

    pub fn import_server(&self, req: ImportServerRequest) -> Result<ServerInstance, String> {
        let startup_mode = normalize_startup_mode(&req.startup_mode).to_string();
        let source_startup_file = std::path::Path::new(&req.jar_path);
        if !source_startup_file.exists() {
            return Err(format!("启动文件不存在: {}", req.jar_path));
        }

        // 在软件目录下创建服务器文件夹
        let id = uuid::Uuid::new_v4().to_string();
        let data_dir = self.data_dir.lock().unwrap().clone();
        let servers_dir = std::path::Path::new(&data_dir).join("servers");
        let server_dir = servers_dir.join(&id);

        // 创建服务器目录
        std::fs::create_dir_all(&server_dir).map_err(|e| format!("无法创建服务器目录: {}", e))?;

        let startup_filename = source_startup_file
            .file_name()
            .ok_or_else(|| "无法获取启动文件名".to_string())?;

        let dest_startup = if startup_mode == "jar" {
            let dest_jar = server_dir.join(startup_filename);
            std::fs::copy(source_startup_file, &dest_jar)
                .map_err(|e| format!("复制JAR文件失败: {}", e))?;
            println!("已复制JAR文件: {} -> {}", req.jar_path, dest_jar.display());
            dest_jar
        } else {
            let source_server_dir = source_startup_file
                .parent()
                .ok_or_else(|| "无法获取启动文件所在目录".to_string())?;

            println!(
                "脚本模式导入：复制服务端目录 {} -> {}",
                source_server_dir.display(),
                server_dir.display()
            );
            copy_dir_recursive(source_server_dir, &server_dir)
                .map_err(|e| format!("复制服务端目录失败: {}", e))?;

            let copied_startup = server_dir.join(startup_filename);
            if !copied_startup.exists() {
                return Err(format!("复制后的启动文件不存在: {}", copied_startup.display()));
            }
            copied_startup
        };

        let server_properties_path = server_dir.join("server.properties");
        if !server_properties_path.exists() {
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

        // 创建服务器实例
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let server = ServerInstance {
            id: id.clone(),
            name: req.name,
            core_type: "unknown".into(),
            core_version: String::new(),
            mc_version: "unknown".into(),
            path: server_dir.to_string_lossy().to_string(),
            jar_path: dest_startup.to_string_lossy().to_string(),
            startup_mode,
            java_path: req.java_path,
            max_memory: req.max_memory,
            min_memory: req.min_memory,
            jvm_args: Vec::new(),
            port: req.port,
            created_at: now,
            last_started_at: None,
            commands: Vec::new(),
        };

        self.servers.lock().unwrap().push(server.clone());
        self.logs.lock().unwrap().insert(id, Vec::new());
        self.save();
        Ok(server)
    }

    pub fn import_modpack(&self, req: ImportModpackRequest) -> Result<ServerInstance, String> {
        let source_path = std::path::Path::new(&req.modpack_path);
        if !source_path.exists() {
            return Err(format!("整合包文件夹不存在: {}", req.modpack_path));
        }
        if !source_path.is_dir() {
            return Err("所选路径不是文件夹".to_string());
        }

        // 查找服务端JAR文件
        let source_jar = find_server_jar(source_path)?;
        println!("找到服务端JAR文件: {}", source_jar);

        // 在软件目录下创建服务器文件夹
        let id = uuid::Uuid::new_v4().to_string();
        let data_dir = self.data_dir.lock().unwrap().clone();
        let servers_dir = std::path::Path::new(&data_dir).join("servers");
        let server_dir = servers_dir.join(&id);

        // 创建服务器目录
        std::fs::create_dir_all(&server_dir).map_err(|e| format!("无法创建服务器目录: {}", e))?;

        // 复制整合包文件夹的所有内容到服务器目录
        println!("正在复制整合包文件: {} -> {}", source_path.display(), server_dir.display());
        copy_dir_recursive(source_path, &server_dir)
            .map_err(|e| format!("复制整合包文件失败: {}", e))?;

        // 获取复制后的JAR文件路径
        let jar_filename = std::path::Path::new(&source_jar)
            .file_name()
            .ok_or_else(|| "无法获取JAR文件名".to_string())?;
        let dest_jar = server_dir.join(jar_filename);

        if !dest_jar.exists() {
            return Err(format!("复制后的JAR文件不存在: {}", dest_jar.display()));
        }

        // 创建服务器实例
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let server = ServerInstance {
            id: id.clone(),
            name: req.name,
            core_type: "modpack".into(),
            core_version: String::new(),
            mc_version: "unknown".into(),
            path: server_dir.to_string_lossy().to_string(),
            jar_path: dest_jar.to_string_lossy().to_string(),
            startup_mode: "jar".to_string(),
            java_path: req.java_path,
            max_memory: req.max_memory,
            min_memory: req.min_memory,
            jvm_args: Vec::new(),
            port: req.port,
            created_at: now,
            last_started_at: None,
            commands: Vec::new(),
        };

        println!(
            "创建服务器实例: id={}, path={}, jar_path={}",
            server.id, server.path, server.jar_path
        );

        self.servers.lock().unwrap().push(server.clone());
        self.logs.lock().unwrap().insert(id, Vec::new());
        self.save();
        Ok(server)
    }

    pub fn start_server(&self, id: &str) -> Result<(), String> {
        let server = {
            let servers = self.servers.lock().unwrap();
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

        // Check if already running
        {
            let mut procs = self.processes.lock().unwrap();
            if let Some(child) = procs.get_mut(id) {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        procs.remove(id);
                    } // Dead process, clean up
                    Ok(None) => return Err("服务器已在运行中".to_string()),
                    Err(_) => {
                        procs.remove(id);
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
        match self.load_preload_script(id, &server) {
            Ok(msg) => {
                self.append_log(id, &format!("[preload] 预加载脚本执行完毕：{}", msg));
                println!("预处理脚本执行完毕：{}", msg);
            }
            Err(e) => {
                // 记录错误到日志，继续启动服务器
                self.append_log(id, &format!("[preload] 预处理脚本执行失败: {}", e));
                println!("预处理脚本执行失败: {}，但将继续启动服务器", e);
            }
        }

        let startup_mode = normalize_startup_mode(&server.startup_mode);
        let startup_path_obj = std::path::Path::new(&server.jar_path);
        let managed_console_encoding =
            resolve_managed_console_encoding(startup_mode, startup_path_obj);

        if startup_mode != "jar" {
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

        let mut cmd = match startup_mode {
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
                        java_home_dir_str.replace('"', "\"\""),
                        java_bin_dir_str.replace('"', "\"\""),
                        startup_filename.replace('"', "\"\"")
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

        cmd.current_dir(&server.path);

        // 使用文件重定向，避免piped导致的Java代理加载问题
        let log_file = std::path::Path::new(&server.path).join("latest.log");

        // 清空旧日志文件，避免读取历史日志
        let stdout_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_file)
            .map_err(|e| format!("无法创建日志文件: {}", e))?;

        let stderr_file = stdout_file
            .try_clone()
            .map_err(|e| format!("无法克隆文件句柄: {}", e))?;

        cmd.stdout(Stdio::from(stdout_file));
        cmd.stderr(Stdio::from(stderr_file));
        cmd.stdin(Stdio::piped());

        // 隐藏控制台窗口
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let child = cmd.spawn().map_err(|e| format!("启动失败: {}", e))?;
        println!("Java进程已启动，PID: {:?}", child.id());

        self.processes.lock().unwrap().insert(id.to_string(), child);

        {
            let mut servers = self.servers.lock().unwrap();
            if let Some(s) = servers.iter_mut().find(|s| s.id == id) {
                s.last_started_at = Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                );
            }
        }
        self.save();
        self.append_log(id, "[Sea Lantern] 服务器启动中...");

        // 启动日志读取线程
        let logs_ref = &self.logs as *const Mutex<HashMap<String, Vec<String>>>;
        let procs_ref = &self.processes as *const Mutex<HashMap<String, Child>>;
        let max_lines = settings.max_log_lines as usize;
        let lid = id.to_string();
        let ptr = logs_ref as usize;
        let p_ptr = procs_ref as usize;
        let ml = max_lines;
        let log_path = log_file.clone();

        std::thread::spawn(move || {
            use std::io::Seek;
            let mut pos = 0u64;
            let mut last_size = 0u64;

            loop {
                // Check if process still exists in manager
                let should_exit = unsafe {
                    let m = &*(p_ptr as *const Mutex<HashMap<String, Child>>);
                    if let Ok(procs) = m.lock() {
                        !procs.contains_key(&lid)
                    } else {
                        false
                    }
                };
                if should_exit {
                    break;
                }

                std::thread::sleep(std::time::Duration::from_millis(500));

                if let Ok(mut file) = std::fs::File::open(&log_path) {
                    if let Ok(metadata) = file.metadata() {
                        let len = metadata.len();

                        if len > last_size {
                            if file.seek(std::io::SeekFrom::Start(pos)).is_ok() {
                                let m = unsafe {
                                    &*(ptr as *const Mutex<HashMap<String, Vec<String>>>)
                                };
                                let mut buffer = Vec::new();

                                if file.read_to_end(&mut buffer).is_ok() {
                                    let content = decode_console_bytes(&buffer);
                                    for line in content.lines() {
                                        if !line.trim().is_empty() {
                                            if let Ok(mut l) = m.lock() {
                                                if let Some(v) = l.get_mut(&lid) {
                                                    v.push(line.to_string());
                                                    if v.len() > ml {
                                                        let d = v.len() - ml;
                                                        v.drain(0..d);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    pos = len;
                                }
                            }
                            last_size = len;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn stop_server(&self, id: &str) -> Result<(), String> {
        // Check if actually running first
        let is_running = {
            let mut procs = self.processes.lock().unwrap();
            if let Some(child) = procs.get_mut(id) {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        procs.remove(id);
                        false
                    }
                    Ok(None) => true,
                    Err(_) => {
                        procs.remove(id);
                        false
                    }
                }
            } else {
                false
            }
        };

        if !is_running {
            self.clear_stopping(id);
            self.append_log(id, "[Sea Lantern] 服务器未运行");
            return Ok(());
        }

        // Send stop command
        self.append_log(id, "[Sea Lantern] 正在发送停止命令...");
        let _ = self.send_command(id, "stop");

        // Wait for graceful shutdown (up to 10 seconds)
        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let mut procs = self.processes.lock().unwrap();
            if let Some(child) = procs.get_mut(id) {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        procs.remove(id);
                        self.clear_stopping(id);
                        self.append_log(id, "[Sea Lantern] 服务器已正常停止");
                        return Ok(());
                    }
                    Ok(None) => {} // Still running
                    Err(_) => {
                        procs.remove(id);
                        self.clear_stopping(id);
                        return Ok(());
                    }
                }
            } else {
                self.clear_stopping(id);
                self.append_log(id, "[Sea Lantern] 服务器已停止");
                return Ok(());
            }
        }

        // Force kill after timeout
        let mut procs = self.processes.lock().unwrap();
        if let Some(mut child) = procs.remove(id) {
            let _ = child.kill();
            let _ = child.wait();
            self.append_log(id, "[Sea Lantern] 服务器超时，已强制终止");
        }
        self.clear_stopping(id);
        Ok(())
    }

    //预处理脚本函数实现
    pub fn load_preload_script(&self, id: &str, server: &ServerInstance) -> Result<String, String> {
        // Windows实现
        #[cfg(target_os = "windows")]
        {
            let preload_script = std::path::Path::new(&server.path).join("preload.bat");
            if preload_script.exists() {
                println!("发现预加载脚本: {:?}", preload_script);
                self.append_log(id, "[preload] 开始执行预加载脚本...");

                // Windows 下执行 bat 文件，使用 CREATE_NO_WINDOW 避免弹窗
                let mut cmd = Command::new("cmd");
                cmd.args(["/c", preload_script.to_str().unwrap_or("preload.bat")])
                    .current_dir(&server.path);

                // 隐藏命令行窗口
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                cmd.creation_flags(CREATE_NO_WINDOW);

                match cmd.output() {
                    Ok(result) => {
                        if result.status.success() {
                            if !result.stdout.is_empty() {
                                let output = String::from_utf8_lossy(&result.stdout);
                                for line in output.lines() {
                                    self.append_log(id, &format!("[preload] {}", line));
                                }
                            }
                            Ok("执行成功".to_string())
                        } else {
                            let error = String::from_utf8_lossy(&result.stderr);
                            Err(format!("preload.bat 执行失败: {}", error))
                        }
                    }
                    Err(e) => Err(e.to_string()),
                }
            } else {
                Ok("未找到脚本".to_string())
            }
        }

        // 非 Windows 实现
        #[cfg(not(target_os = "windows"))]
        {
            let preload_script = std::path::Path::new(&server.path).join("preload.sh");
            if preload_script.exists() {
                println!("发现预加载脚本: {:?}", preload_script);
                self.append_log(id, "[preload] 开始执行预加载脚本...");

                // 非 Windows 系统下，作为 shell 脚本执行
                match Command::new("sh")
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
                                    self.append_log(id, &format!("[preload] {}", line));
                                }
                            }
                            Ok("执行成功".to_string())
                        } else {
                            let error = String::from_utf8_lossy(&result.stderr);
                            Err(format!("preload.sh 执行失败: {}", error))
                        }
                    }
                    Err(e) => Err(e.to_string()),
                }
            } else {
                Ok("未找到脚本".to_string())
            }
        }
    }

    pub fn send_command(&self, id: &str, command: &str) -> Result<(), String> {
        let mut procs = self.processes.lock().unwrap();
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
        self.servers.lock().unwrap().clone()
    }

    pub fn get_server_status(&self, id: &str) -> ServerStatusInfo {
        let mut procs = self.processes.lock().unwrap();
        let is_running = if let Some(child) = procs.get_mut(id) {
            match child.try_wait() {
                Ok(Some(_)) => {
                    procs.remove(id);
                    false
                }
                Ok(None) => true,
                Err(_) => {
                    procs.remove(id);
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
        // Only stop if actually running
        {
            let procs = self.processes.lock().unwrap();
            if procs.contains_key(id) {
                drop(procs);
                let _ = self.stop_server(id);
            }
        }
        self.servers.lock().unwrap().retain(|s| s.id != id);
        self.logs.lock().unwrap().remove(id);
        self.save();
        Ok(())
    }

    pub fn get_logs(&self, id: &str, since: usize) -> Vec<String> {
        let logs = self.logs.lock().unwrap();
        if let Some(v) = logs.get(id) {
            if since < v.len() {
                v[since..].to_vec()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    fn append_log(&self, id: &str, msg: &str) {
        if let Ok(mut logs) = self.logs.lock() {
            if let Some(v) = logs.get_mut(id) {
                v.push(msg.to_string());
            }
        }
    }

    pub fn stop_all_servers(&self) {
        let ids: Vec<String> = self.processes.lock().unwrap().keys().cloned().collect();
        for id in ids {
            let _ = self.stop_server(&id);
        }
    }

    pub fn force_stop_all_servers(&self) {
        let ids: Vec<String> = self.processes.lock().unwrap().keys().cloned().collect();
        let mut procs = self.processes.lock().unwrap();
        for id in ids {
            if let Some(mut child) = procs.remove(&id) {
                let _ = child.kill();
                let _ = child.wait();
                self.append_log(&id, "[Sea Lantern] 已强制终止服务器进程");
            }
        }
    }

    pub fn add_server_command(
        &self,
        server_id: &str,
        name: &str,
        command: &str,
    ) -> Result<(), String> {
        let mut servers = self.servers.lock().unwrap();
        if let Some(server) = servers.iter_mut().find(|s| s.id == server_id) {
            let command_id = uuid::Uuid::new_v4().to_string();
            server.commands.push(ServerCommand {
                id: command_id,
                name: name.to_string(),
                command: command.to_string(),
            });
            drop(servers);
            self.save();
            Ok(())
        } else {
            Err("未找到服务器".to_string())
        }
    }

    pub fn update_server_command(
        &self,
        server_id: &str,
        command_id: &str,
        name: &str,
        command: &str,
    ) -> Result<(), String> {
        let mut servers = self.servers.lock().unwrap();
        if let Some(server) = servers.iter_mut().find(|s| s.id == server_id) {
            if let Some(cmd) = server.commands.iter_mut().find(|c| c.id == command_id) {
                cmd.name = name.to_string();
                cmd.command = command.to_string();
                drop(servers);
                self.save();
                Ok(())
            } else {
                Err("未找到自定义指令".to_string())
            }
        } else {
            Err("未找到服务器".to_string())
        }
    }

    pub fn delete_server_command(&self, server_id: &str, command_id: &str) -> Result<(), String> {
        let mut servers = self.servers.lock().unwrap();
        if let Some(server) = servers.iter_mut().find(|s| s.id == server_id) {
            let initial_len = server.commands.len();
            server.commands.retain(|c| c.id != command_id);
            if initial_len != server.commands.len() {
                drop(servers);
                self.save();
                Ok(())
            } else {
                Err("未找到自定义指令".to_string())
            }
        } else {
            Err("未找到服务器".to_string())
        }
    }

    pub fn update_server_name(&self, id: &str, name: &str) -> Result<(), String> {
        let mut servers = self.servers.lock().unwrap();
        if let Some(server) = servers.iter_mut().find(|s| s.id == id) {
            server.name = name.to_string();
            drop(servers);
            self.save();
            Ok(())
        } else {
            Err("未找到服务器".to_string())
        }
    }
}

fn get_data_dir() -> String {
    // 使用软件根目录（可执行文件所在目录）
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let data_dir = exe_dir.to_path_buf();
            println!("数据目录: {}", data_dir.display());
            return data_dir.to_string_lossy().to_string();
        }
    }

    // 如果获取失败，使用当前工作目录
    println!("警告: 无法获取可执行文件目录，使用当前目录");
    ".".to_string()
}

fn normalize_startup_mode(mode: &str) -> &str {
    match mode.to_ascii_lowercase().as_str() {
        "bat" => "bat",
        "sh" => "sh",
        _ => "jar",
    }
}

fn resolve_managed_console_encoding(
    startup_mode: &str,
    startup_path: &std::path::Path,
) -> ManagedConsoleEncoding {
    #[cfg(target_os = "windows")]
    {
        if startup_mode == "bat" {
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

fn find_server_jar(modpack_path: &std::path::Path) -> Result<String, String> {
    // 常见的服务端JAR文件名模式
    let patterns = vec![
        "server.jar",
        "forge.jar",
        "fabric-server.jar",
        "minecraft_server.jar",
        "paper.jar",
        "spigot.jar",
        "purpur.jar",
    ];

    // 首先尝试精确匹配
    for pattern in &patterns {
        let jar_path = modpack_path.join(pattern);
        if jar_path.exists() {
            return Ok(jar_path.to_string_lossy().to_string());
        }
    }

    // 如果没有精确匹配，查找所有.jar文件
    let entries = std::fs::read_dir(modpack_path).map_err(|e| format!("无法读取文件夹: {}", e))?;

    let mut jar_files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "jar" {
                    jar_files.push(path);
                }
            }
        }
    }

    if jar_files.is_empty() {
        return Err("整合包文件夹中未找到JAR文件".to_string());
    }

    // 如果只有一个JAR文件，直接使用
    if jar_files.len() == 1 {
        return Ok(jar_files[0].to_string_lossy().to_string());
    }

    // 如果有多个JAR文件，优先选择包含服务端关键词的文件
    for jar in &jar_files {
        if let Some(name) = jar.file_name() {
            let name_lower = name.to_string_lossy().to_lowercase();
            if name_lower.contains("server")
                || name_lower.contains("forge")
                || name_lower.contains("fabric")
                || name_lower.contains("mohist")
                || name_lower.contains("paper")
                || name_lower.contains("spigot")
                || name_lower.contains("purpur")
                || name_lower.contains("bukkit")
                || name_lower.contains("catserver")
                || name_lower.contains("arclight")
            {
                return Ok(jar.to_string_lossy().to_string());
            }
        }
    }

    // 如果都不匹配，返回第一个JAR文件
    Ok(jar_files[0].to_string_lossy().to_string())
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
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
