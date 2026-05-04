//! 服务器管理总入口
//!
//! 这里负责把建服、启停、路径更新和进程状态整理到同一个管理器里

mod common;
mod fs;
mod process;
mod provisioning;
mod runtime_control;
mod runtime_start;

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::process::Child;
use std::sync::Mutex;

use crate::models::server::*;
use serde::{Deserialize, Serialize};

use super::installer;
use super::log_pipeline as server_log_pipeline;
use common::{get_data_dir, normalize_startup_mode, validate_server_name, ManagedConsoleEncoding};
use fs::{load_servers, remove_run_path_mapping, save_servers, update_run_path_mapping};

/// 强停前返回给前端的确认信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceStopPreparation {
    /// 本次强停确认口令
    pub token: String,
    /// 口令失效时间戳
    pub expires_at: u64,
}

/// 启动回退信息
#[derive(Debug, Clone)]
pub struct StartFallbackInfo {
    /// 原始启动方式
    pub from_mode: String,
    /// 回退后的启动方式
    pub to_mode: String,
    /// 触发回退的原因
    pub reason: String,
}

/// 一次启动请求的结果
#[derive(Debug, Clone)]
pub struct StartServerReport {
    /// 服务器 ID
    pub server_id: String,
    /// 服务器名称
    pub server_name: String,
    /// 启动回退信息
    pub fallback: Option<StartFallbackInfo>,
}

/// 服务器运行和配置管理器
pub struct ServerManager {
    pub servers: Mutex<Vec<ServerInstance>>,
    pub processes: Mutex<HashMap<String, Child>>,
    pub stopping_servers: Mutex<HashSet<String>>,
    pub starting_servers: Mutex<HashSet<String>>,
    pub pending_force_stop_tokens: Mutex<HashMap<String, (String, u64)>>,
    pub data_dir: Mutex<String>,
}

impl ServerManager {
    /// 创建服务器管理器
    ///
    /// 启动时会一并读入已经保存的服务器列表
    pub fn new() -> Self {
        let data_dir = get_data_dir();
        let servers = load_servers(&data_dir);
        ServerManager {
            servers: Mutex::new(servers),
            processes: Mutex::new(HashMap::new()),
            stopping_servers: Mutex::new(HashSet::new()),
            starting_servers: Mutex::new(HashSet::new()),
            pending_force_stop_tokens: Mutex::new(HashMap::new()),
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

    /// 清理启动中标记
    ///
    /// 这个方法会在启动流程结束后调用
    pub fn clear_starting(&self, id: &str) {
        if let Ok(mut s) = self.starting_servers.lock() {
            s.remove(id);
        }
    }

    /// 请求优雅停服
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    pub fn request_stop_server(&self, id: &str) -> Result<(), String> {
        runtime_control::request_stop_server(self, id)
    }

    /// 生成强停确认口令
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    pub fn prepare_force_stop_server(&self, id: &str) -> Result<ForceStopPreparation, String> {
        runtime_control::prepare_force_stop_server(self, id)
    }

    /// 使用确认口令执行强停
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    /// - `confirmation_token`: 强停确认口令
    pub fn force_stop_server(&self, id: &str, confirmation_token: &str) -> Result<(), String> {
        runtime_control::force_stop_server(self, id, confirmation_token)
    }

    fn save(&self) -> Result<(), String> {
        let servers = self.lock_servers()?;
        let data_dir = self.data_dir_value()?;
        save_servers(&data_dir, &servers);
        Ok(())
    }

    fn get_app_settings(&self) -> crate::models::settings::AppSettings {
        crate::services::global::settings_manager().get()
    }

    fn lock_servers(&self) -> Result<std::sync::MutexGuard<'_, Vec<ServerInstance>>, String> {
        self.servers
            .lock()
            .map_err(|_| "servers lock poisoned".to_string())
    }

    fn lock_processes(&self) -> Result<std::sync::MutexGuard<'_, HashMap<String, Child>>, String> {
        self.processes
            .lock()
            .map_err(|_| "processes lock poisoned".to_string())
    }

    fn data_dir_value(&self) -> Result<String, String> {
        self.data_dir
            .lock()
            .map(|dir| dir.clone())
            .map_err(|_| "data_dir lock poisoned".to_string())
    }

    /// 尝试从服务器目录下的 SL.json 读取启动配置
    fn read_sl_startup_config(server: &ServerInstance) -> Option<(u32, u32)> {
        let sl_path = std::path::Path::new(&server.path).join("SL.json");
        if !sl_path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&sl_path).ok()?;
        let config: crate::commands::server::config::SLStartupConfig =
            serde_json::from_str(&content).ok()?;
        match (config.max_memory, config.min_memory) {
            (Some(max), Some(min)) => Some((max, min)),
            (Some(max), None) => Some((max, server.min_memory)),
            (None, Some(min)) => Some((server.max_memory, min)),
            (None, None) => None,
        }
    }

    /// 按当前设置拼出托管启动参数
    ///
    /// 优先从 SL.json 读取内存值，没有则回退到 ServerInstance 的默认值
    fn build_managed_jvm_args(
        &self,
        server: &ServerInstance,
        settings: &crate::models::settings::AppSettings,
        console_encoding: ManagedConsoleEncoding,
    ) -> Vec<String> {
        let java_encoding = console_encoding.java_name();
        let (max_mem, min_mem) = Self::read_sl_startup_config(server)
            .unwrap_or((server.max_memory, server.min_memory));
        let mut args = vec![
            format!("-Xmx{}M", max_mem),
            format!("-Xms{}M", min_mem),
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

    /// 写入 `user_jvm_args.txt`
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

    /// 新建一个服务器记录
    ///
    /// # Parameters
    ///
    /// - `req`: 新建服务器请求
    pub fn create_server(&self, req: CreateServerRequest) -> Result<ServerInstance, String> {
        provisioning::create_server(self, req)
    }

    /// 导入一个已有服务端目录副本
    ///
    /// # Parameters
    ///
    /// - `req`: 导入服务器请求
    pub fn import_server(&self, req: ImportServerRequest) -> Result<ServerInstance, String> {
        provisioning::import_server(self, req)
    }

    /// 导入整合包并生成服务器记录
    ///
    /// # Parameters
    ///
    /// - `req`: 整合包导入请求
    pub fn import_modpack(&self, req: ImportModpackRequest) -> Result<ServerInstance, String> {
        provisioning::import_modpack(self, req)
    }

    /// 接入一个现有服务器目录
    ///
    /// # Parameters
    ///
    /// - `req`: 接入已有服务器请求
    pub fn add_existing_server(
        &self,
        req: AddExistingServerRequest,
    ) -> Result<ServerInstance, String> {
        provisioning::add_existing_server(self, req)
    }

    /// 启动服务器
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    pub fn start_server(&self, id: &str) -> Result<StartServerReport, String> {
        runtime_start::start_server(self, id)
    }

    /// 停止服务器
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    pub fn stop_server(&self, id: &str) -> Result<(), String> {
        runtime_control::stop_server(self, id)
    }

    /// 向运行中的服务器发送控制台命令
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    /// - `command`: 要发送的控制台命令
    pub fn send_command(&self, id: &str, command: &str) -> Result<(), String> {
        let mut procs = self.lock_processes()?;
        let child = procs
            .get_mut(id)
            .ok_or_else(|| format!("服务器未运行: {}", id))?;
        if let Some(ref mut stdin) = child.stdin {
            writeln!(stdin, "{}", command).map_err(|e| format!("发送失败（id={}）: {}", id, e))?;
            stdin
                .flush()
                .map_err(|e| format!("发送失败（id={}）: {}", id, e))?;
        }
        Ok(())
    }

    /// 读取全部服务器列表
    pub fn get_server_list(&self) -> Vec<ServerInstance> {
        self.lock_servers()
            .map(|servers| servers.clone())
            .unwrap_or_default()
    }

    /// 查询单个服务器运行状态
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    pub fn get_server_status(&self, id: &str) -> ServerStatusInfo {
        runtime_control::get_server_status(self, id)
    }

    /// 删除服务器记录和对应目录
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    pub fn delete_server(&self, id: &str) -> Result<(), String> {
        {
            let procs = self.lock_processes()?;
            if procs.contains_key(id) {
                drop(procs);
                let _ = self.stop_server(id);
            }
        }

        server_log_pipeline::shutdown_writer(id);

        let server_path = {
            let servers = self.lock_servers()?;
            servers.iter().find(|s| s.id == id).map(|s| s.path.clone())
        };
        if let Some(path) = server_path {
            let dir = std::path::Path::new(&path);
            if dir.exists() {
                std::fs::remove_dir_all(dir).map_err(|e| format!("删除服务器目录失败: {}", e))?;
            }
        }

        self.lock_servers()?.retain(|s| s.id != id);
        let data_dir = self.data_dir_value()?;
        remove_run_path_mapping(&data_dir, id);
        self.save()?;
        Ok(())
    }

    /// 读取当前正在运行的服务器 ID 列表
    pub fn get_running_server_ids(&self) -> Vec<String> {
        self.lock_processes()
            .map(|procs| procs.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// 更新服务器名称
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    /// - `new_name`: 新名称
    pub fn update_server_name(&self, id: &str, new_name: &str) -> Result<(), String> {
        let validated_name = validate_server_name(new_name)?;
        let mut servers = self.lock_servers()?;
        if let Some(server) = servers.iter_mut().find(|s| s.id == id) {
            server.name = validated_name;
            drop(servers);
            self.save()?;
            Ok(())
        } else {
            Err("未找到服务器".to_string())
        }
    }

    /// 更新服务器路径和启动信息
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    /// - `new_path`: 新的服务器目录
    /// - `new_jar_path`: 新的启动文件路径
    /// - `new_startup_mode`: 新的启动方式
    pub fn update_server_path(
        &self,
        id: &str,
        new_path: &str,
        new_jar_path: Option<&str>,
        new_startup_mode: Option<&str>,
    ) -> Result<ServerInstance, String> {
        // 检查服务器是否正在运行
        {
            let procs = self.lock_processes()?;
            if procs.contains_key(id) {
                return Err("服务器正在运行中，请先停止服务器再修改路径".to_string());
            }
        }

        let mut servers = self.lock_servers()?;
        if let Some(server) = servers.iter_mut().find(|s| s.id == id) {
            // 更新路径
            server.path = new_path.to_string();

            // 如果提供了新的 jar_path，则更新
            if let Some(jar_path) = new_jar_path {
                server.jar_path = jar_path.to_string();
            }

            // 如果提供了新的 startup_mode，则更新
            if let Some(startup_mode) = new_startup_mode {
                server.startup_mode = normalize_startup_mode(startup_mode).to_string();
            }

            // 尝试从新的路径检测核心类型
            if server.startup_mode != "custom" {
                let detected_core = installer::detect_core_type(&server.jar_path);
                if !detected_core.is_empty() && detected_core != "Unknown" {
                    server.core_type = detected_core;
                }
            }

            // 尝试从 server.properties 读取端口
            let server_properties_path = std::path::Path::new(new_path).join("server.properties");
            if server_properties_path.exists() {
                if let Ok(props) = crate::services::server::config::read_properties(
                    server_properties_path.to_str().unwrap_or_default(),
                ) {
                    if let Some(port_str) = props.get("server-port") {
                        if let Ok(parsed_port) = port_str.parse::<u16>() {
                            server.port = parsed_port;
                        }
                    }
                }
            }

            let updated_server = server.clone();
            drop(servers);
            self.save()?;

            // 更新运行路径映射
            let data_dir = self.data_dir_value()?;
            update_run_path_mapping(&data_dir, id, new_path);

            Ok(updated_server)
        } else {
            Err("未找到服务器".to_string())
        }
    }

    /// 停止全部正在运行的服务器
    pub fn stop_all_servers(&self) {
        let ids: Vec<String> = self
            .lock_processes()
            .map(|procs| procs.keys().cloned().collect())
            .unwrap_or_default();
        for id in ids {
            let _ = self.stop_server(&id);
        }
    }
}
