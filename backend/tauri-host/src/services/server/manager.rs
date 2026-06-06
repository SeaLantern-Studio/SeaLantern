//! 服务器管理总入口
//!
//! 这里负责把建服、启停、路径更新和进程状态整理到同一个管理器里

mod common;
mod cpu_policy;
mod fs;
pub(crate) mod process;
mod provisioning;
mod registry;
mod runtime_control;
mod runtime_start;
pub(crate) mod startup_support;

use std::collections::{HashMap, HashSet};
use std::process::Child;
use std::sync::Mutex;

use crate::models::server::*;
use crate::services::server::runtime;
use crate::utils::logger;
use crate::utils::server_status::status_blocks_start;
use sea_lantern_server_config_core::properties::read_properties;
use sea_lantern_server_installer_core::detect_core_type;
use serde::{Deserialize, Serialize};

use super::log_pipeline as server_log_pipeline;
use super::manager::process::force_kill_process_tree;
use super::runtime::local::LocalServerRuntime;
pub(crate) use crate::services::server::runtime::docker_itzg::DockerLaunchDetail;
use common::{get_data_dir, normalize_startup_mode, validate_server_name};
use fs::{load_servers, remove_run_path_mapping, save_servers, update_run_path_mapping};
pub use runtime_start::LocalLaunchDetail;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateServerRecordEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub runtime_kind: String,
    pub created_at: u64,
    pub last_started_at: Option<u64>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateServerRecordGroup {
    pub canonical_id: String,
    pub canonical_name: String,
    pub reasons: Vec<String>,
    pub entries: Vec<DuplicateServerRecordEntry>,
    pub removable_ids: Vec<String>,
    pub blocked_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRegistryDedupeReport {
    pub total_servers: usize,
    pub duplicate_groups: Vec<DuplicateServerRecordGroup>,
    pub removed_ids: Vec<String>,
}

fn log_manager_result<T>(
    action: &str,
    detail: &str,
    result: Result<T, String>,
) -> Result<T, String> {
    match result {
        Ok(value) => Ok(value),
        Err(err) => {
            logger::log_user_action_error("server.manager", action, detail, &err);
            Err(err)
        }
    }
}

pub(crate) fn build_local_launch_detail_for_server(
    server: &ServerInstance,
) -> Result<LocalLaunchDetail, String> {
    let settings = crate::services::global::settings_manager().get();
    runtime_start::get_local_launch_detail(server, &settings)
}

pub(crate) fn build_docker_launch_detail_for_server(
    server: &ServerInstance,
) -> Result<DockerLaunchDetail, String> {
    let settings = crate::services::global::settings_manager().get();
    let runtime = server
        .docker_itzg_runtime()
        .ok_or_else(|| format!("当前服务器运行时暂未实现: {}", server.runtime_kind))?;
    crate::services::server::runtime::docker_itzg::build_docker_launch_detail(
        server, runtime, &settings,
    )
}

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
    /// 本次是否只是识别到现有运行态而跳过了重复启动
    pub skipped_existing_state: bool,
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
    pub(crate) fn ensure_local_runtime<'a>(
        &self,
        server: &'a ServerInstance,
    ) -> Result<&'a LocalRuntimeConfig, String> {
        LocalServerRuntime::ensure_config(server)
    }

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

    pub(crate) fn is_stopping(&self, id: &str) -> bool {
        self.stopping_servers
            .lock()
            .map(|stopping| stopping.contains(id))
            .unwrap_or(false)
    }

    pub(crate) fn mark_stopping(&self, id: &str) {
        if let Ok(mut stopping) = self.stopping_servers.lock() {
            stopping.insert(id.to_string());
        }
    }

    pub(crate) fn clear_stopping(&self, id: &str) {
        if let Ok(mut stopping) = self.stopping_servers.lock() {
            stopping.remove(id);
        }
    }

    pub(crate) fn is_starting(&self, id: &str) -> bool {
        self.starting_servers
            .lock()
            .map(|s| s.contains(id))
            .unwrap_or(false)
    }

    pub(crate) fn mark_starting(&self, id: &str) {
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
        let detail = format!("server_id={}", id);
        logger::log_user_action("server.manager", "request_stop", &detail);
        log_manager_result("request_stop", &detail, runtime_control::request_stop_server(self, id))
    }

    /// 生成强停确认口令
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    pub fn prepare_force_stop_server(&self, id: &str) -> Result<ForceStopPreparation, String> {
        let detail = format!("server_id={}", id);
        logger::log_user_action("server.manager", "prepare_force_stop", &detail);
        log_manager_result(
            "prepare_force_stop",
            &detail,
            runtime_control::prepare_force_stop_server(self, id),
        )
    }

    /// 使用确认口令执行强停
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    /// - `confirmation_token`: 强停确认口令
    pub fn force_stop_server(&self, id: &str, confirmation_token: &str) -> Result<(), String> {
        let detail = format!("server_id={} token_present={}", id, !confirmation_token.is_empty());
        logger::log_user_action("server.manager", "force_stop", &detail);
        log_manager_result(
            "force_stop",
            &detail,
            runtime_control::force_stop_server(self, id, confirmation_token),
        )
    }

    pub(crate) fn save(&self) -> Result<(), String> {
        let servers = self.lock_servers()?;
        let data_dir = self.data_dir_value()?;
        save_servers(&data_dir, &servers);
        Ok(())
    }

    pub(crate) fn get_app_settings(&self) -> crate::models::settings::AppSettings {
        crate::services::global::settings_manager().get()
    }

    pub(crate) fn lock_servers(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, Vec<ServerInstance>>, String> {
        self.servers
            .lock()
            .map_err(|_| "servers lock poisoned".to_string())
    }

    pub(crate) fn lock_processes(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HashMap<String, Child>>, String> {
        self.processes
            .lock()
            .map_err(|_| "processes lock poisoned".to_string())
    }

    pub(crate) fn find_server_clone(&self, id: &str) -> Result<ServerInstance, String> {
        self.find_server_clone_optional(id)?
            .ok_or_else(|| format!("未找到服务器: {}", id))
    }

    pub(crate) fn find_server_clone_optional(
        &self,
        id: &str,
    ) -> Result<Option<ServerInstance>, String> {
        let servers = self.lock_servers()?;
        Ok(servers.iter().find(|s| s.id == id).cloned())
    }

    pub(crate) fn force_kill_local_process(&self, child: &mut Child) -> Result<(), String> {
        force_kill_process_tree(child)
    }

    fn data_dir_value(&self) -> Result<String, String> {
        self.data_dir
            .lock()
            .map(|dir| dir.clone())
            .map_err(|_| "data_dir lock poisoned".to_string())
    }

    /// 新建一个服务器记录
    ///
    /// # Parameters
    ///
    /// - `req`: 新建服务器请求
    pub fn create_server(&self, req: CreateServerRequest) -> Result<ServerInstance, String> {
        let detail = format!(
            "name={} core={} mc={} port={} aliases={}",
            req.name,
            req.core_type,
            req.mc_version,
            req.port,
            req.aliases.join(",")
        );
        logger::log_user_action("server.manager", "create_local", &detail);
        log_manager_result("create_local", &detail, provisioning::create_server(self, req))
    }

    pub fn create_docker_itzg_server(
        &self,
        req: CreateDockerItzgServerRequest,
    ) -> Result<ServerInstance, String> {
        let detail = format!(
            "name={} core={} mc={} port={} image={}:{} container={} aliases={}",
            req.name,
            req.core_type,
            req.mc_version,
            req.port,
            req.runtime.image,
            req.runtime.image_tag,
            req.runtime.container_name,
            req.aliases.join(",")
        );
        logger::log_user_action("server.manager", "create_docker_itzg", &detail);
        log_manager_result(
            "create_docker_itzg",
            &detail,
            provisioning::create_docker_itzg_server(self, req),
        )
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
        let detail = format!(
            "name={} path={} port={} startup_mode={} aliases={}",
            req.name,
            req.server_path,
            req.port,
            req.startup_mode,
            req.aliases.join(",")
        );
        logger::log_user_action("server.manager", "attach_existing", &detail);
        log_manager_result("attach_existing", &detail, provisioning::add_existing_server(self, req))
    }

    /// 启动服务器
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    pub fn start_server(&self, id: &str) -> Result<StartServerReport, String> {
        let detail = format!("server_id={}", id);
        logger::log_user_action("server.manager", "start", &detail);
        let result = (|| {
            let server = self.find_server_clone(id)?;

            let status = self.get_server_status(id);
            if status_blocks_start(&status) {
                logger::log_user_action(
                    "server.manager",
                    "start_skip_existing_state",
                    &format!(
                        "server_id={} status={} detail={} error={}",
                        id,
                        status.status.as_str(),
                        status.detail_message.as_deref().unwrap_or(""),
                        status.error_message.as_deref().unwrap_or("")
                    ),
                );
                return Ok(StartServerReport {
                    server_id: server.id.clone(),
                    server_name: server.name.clone(),
                    skipped_existing_state: true,
                    fallback: None,
                });
            }

            let resolved_runtime = runtime::resolve_runtime(&server)?;
            let start_result = resolved_runtime.start_with_manager(
                self,
                runtime::RuntimeStartRequest { server_id: id, server: &server },
            )?;

            if let Some(process_handle) = start_result.process_handle {
                if let Some(mut child) = process_handle.into_local_child() {
                    println!("Java进程已启动，PID: {:?}", child.id());

                    let stdout = child.stdout.take();
                    let stderr = child.stderr.take();

                    self.lock_processes()?.insert(id.to_string(), child);
                    self.mark_starting(id);

                    if let Some(stdout) = stdout {
                        server_log_pipeline::spawn_server_output_reader(id.to_string(), stdout);
                    }
                    if let Some(stderr) = stderr {
                        server_log_pipeline::spawn_server_output_reader(id.to_string(), stderr);
                    }
                }
            }

            {
                let mut servers = self.lock_servers()?;
                if let Some(s) = servers.iter_mut().find(|s| s.id == id) {
                    s.last_started_at = Some(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map_err(|e| format!("获取当前时间失败: {}", e))?
                            .as_secs(),
                    );
                }
            }
            self.save()?;
            let _ = server_log_pipeline::append_sealantern_log(id, "[Sea Lantern] 服务器启动中...");

            Ok(StartServerReport {
                server_id: server.id.clone(),
                server_name: server.name.clone(),
                skipped_existing_state: false,
                fallback: start_result.fallback,
            })
        })();

        log_manager_result("start", &detail, result)
    }

    pub(crate) fn start_local_runtime(
        &self,
        request: runtime::RuntimeStartRequest<'_>,
    ) -> Result<runtime::RuntimeStartResult, String> {
        runtime_start::start_local_runtime(self, request)
    }

    /// 停止服务器
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    pub fn stop_server(&self, id: &str) -> Result<(), String> {
        let detail = format!("server_id={}", id);
        logger::log_user_action("server.manager", "stop", &detail);
        log_manager_result("stop", &detail, runtime_control::stop_server(self, id))
    }

    /// 向运行中的服务器发送控制台命令
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    /// - `command`: 要发送的控制台命令
    pub fn send_command(&self, id: &str, command: &str) -> Result<(), String> {
        let detail = format!("server_id={} command={}", id, command);
        logger::log_user_action("server.manager", "send_command", &detail);
        let result = (|| {
            let server = self.find_server_clone(id)?;
            let resolved_runtime = runtime::resolve_runtime(&server)?;
            resolved_runtime.send_command_with_manager(self, &server, command)
        })();

        log_manager_result("send_command", &detail, result)
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

    pub fn get_local_launch_detail(&self, id: &str) -> Result<LocalLaunchDetail, String> {
        let server = self.find_server_clone(id)?;
        build_local_launch_detail_for_server(&server)
    }

    pub fn get_docker_launch_detail(&self, id: &str) -> Result<DockerLaunchDetail, String> {
        let server = self.find_server_clone(id)?;
        build_docker_launch_detail_for_server(&server)
    }

    pub fn audit_duplicate_server_records(&self) -> Result<ServerRegistryDedupeReport, String> {
        logger::log_user_action("server.manager", "audit_duplicate_records", "");
        log_manager_result(
            "audit_duplicate_records",
            "",
            registry::audit_duplicate_server_records(self),
        )
    }

    pub fn dedupe_duplicate_server_records(&self) -> Result<ServerRegistryDedupeReport, String> {
        logger::log_user_action("server.manager", "dedupe_duplicate_records", "");
        log_manager_result(
            "dedupe_duplicate_records",
            "",
            registry::dedupe_duplicate_server_records(self),
        )
    }

    /// 删除服务器记录和对应目录
    ///
    /// # Parameters
    ///
    /// - `id`: 服务器 ID
    pub fn delete_server(&self, id: &str) -> Result<(), String> {
        let detail = format!("server_id={}", id);
        logger::log_user_action("server.manager", "delete", &detail);
        let result = (|| {
            let status = self.get_server_status(id).status;
            if !matches!(status, ServerStatus::Stopped | ServerStatus::Error) {
                let _ = self.stop_server(id);
            }

            server_log_pipeline::shutdown_writer(id);

            let server_path = self
                .find_server_clone_optional(id)?
                .map(|server| server.path);
            if let Some(path) = server_path {
                let dir = std::path::Path::new(&path);
                if dir.exists() {
                    std::fs::remove_dir_all(dir)
                        .map_err(|e| format!("删除服务器目录失败: {}", e))?;
                }
            }

            self.lock_servers()?.retain(|s| s.id != id);
            let data_dir = self.data_dir_value()?;
            remove_run_path_mapping(&data_dir, id);
            self.save()?;
            Ok(())
        })();

        log_manager_result("delete", &detail, result)
    }

    /// 读取当前正在运行的服务器 ID 列表
    pub fn get_running_server_ids(&self) -> Vec<String> {
        self.get_server_list()
            .into_iter()
            .filter_map(|server| {
                let status = self.get_server_status(&server.id).status;
                if matches!(
                    status,
                    ServerStatus::Running | ServerStatus::Starting | ServerStatus::Stopping
                ) {
                    Some(server.id)
                } else {
                    None
                }
            })
            .collect()
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

    pub fn update_server_java_path(
        &self,
        id: &str,
        new_java_path: &str,
    ) -> Result<ServerInstance, String> {
        let validated_java = crate::services::java_detector::validate_java(new_java_path)
            .map_err(|e| format!("Java 路径校验失败: {}", e))?;

        let mut servers = self.lock_servers()?;
        if let Some(server) = servers.iter_mut().find(|s| s.id == id) {
            let runtime = server
                .local_runtime_mut()
                .ok_or_else(|| "当前运行时不支持修改 Java 路径".to_string())?;
            runtime.java_path = validated_java.path;

            let updated_server = server.clone();
            drop(servers);
            self.save()?;
            Ok(updated_server)
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
                let runtime = server
                    .local_runtime_mut()
                    .ok_or_else(|| "当前运行时不支持修改启动文件路径".to_string())?;
                runtime.jar_path = jar_path.to_string();
            }

            // 如果提供了新的 startup_mode，则更新
            if let Some(startup_mode) = new_startup_mode {
                let runtime = server
                    .local_runtime_mut()
                    .ok_or_else(|| "当前运行时不支持修改启动方式".to_string())?;
                runtime.startup_mode = normalize_startup_mode(startup_mode).to_string();
            }

            // 尝试从新的路径检测核心类型
            if server.startup_mode_str() != "custom" {
                let jar_path = server.jar_path().unwrap_or_default();
                let detected_core = detect_core_type(jar_path);
                if !detected_core.is_empty() && detected_core != "Unknown" {
                    server.core_type = detected_core;
                }
            }

            // 尝试从 server.properties 读取端口
            let server_properties_path = std::path::Path::new(new_path).join("server.properties");
            if server_properties_path.exists() {
                if let Ok(props) = read_properties(server_properties_path.to_str().unwrap_or_default()) {
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
        let ids = self.get_running_server_ids();
        for id in ids {
            let _ = self.stop_server(&id);
        }
    }
}
