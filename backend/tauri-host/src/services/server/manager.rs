//! 服务器管理总入口
//!
//! 这里负责把建服、启停、路径更新和进程状态整理到同一个管理器里

mod common;
mod cpu_policy;
mod fs;
mod i18n;
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
use sea_lantern_server_local_setup_core::{
    normalize_cli_startup_mode, refresh_local_server_core_type,
};
use serde::{Deserialize, Serialize};
use sl_server_info::log::LogStream;

use super::log_pipeline as server_log_pipeline;
use super::manager::process::force_kill_process_tree;
use super::runtime::local::LocalServerRuntime;
pub(crate) use crate::services::server::runtime::docker_itzg::DockerLaunchDetail;
use common::{get_data_dir_checked, validate_server_name};
use fs::{
    load_servers_for_bootstrap, remove_run_path_mapping, save_servers, update_run_path_mapping,
};
use i18n::{manager_t, manager_t1};
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
        Ok(value) => {
            logger::log_info_ctx("server.manager", action, &format!("success {}", detail));
            Ok(value)
        }
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
    let runtime = server.docker_itzg_runtime().ok_or_else(|| {
        manager_t1("server.manager.runtime_not_supported", server.runtime_kind.clone())
    })?;
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

    #[cfg(test)]
    pub fn new() -> Self {
        Self::new_checked().expect("manager should initialize")
    }

    pub fn new_checked() -> Result<Self, String> {
        let data_dir = get_data_dir_checked()?;
        let servers = load_servers_for_bootstrap(&data_dir)?;
        Ok(ServerManager {
            servers: Mutex::new(servers),
            processes: Mutex::new(HashMap::new()),
            stopping_servers: Mutex::new(HashSet::new()),
            starting_servers: Mutex::new(HashSet::new()),
            pending_force_stop_tokens: Mutex::new(HashMap::new()),
            data_dir: Mutex::new(data_dir),
        })
    }

    pub(crate) fn is_stopping_checked(&self, id: &str) -> Result<bool, String> {
        self.stopping_servers
            .lock()
            .map(|stopping| stopping.contains(id))
            .map_err(|_| "stopping_servers lock poisoned".to_string())
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

    pub(crate) fn is_starting_checked(&self, id: &str) -> Result<bool, String> {
        self.starting_servers
            .lock()
            .map(|s| s.contains(id))
            .map_err(|_| "starting_servers lock poisoned".to_string())
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
        save_servers(&data_dir, &servers)
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
            .ok_or_else(|| manager_t1("server.manager.server_not_found", id.to_string()))
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
                        server_log_pipeline::spawn_server_output_reader(
                            id.to_string(),
                            LogStream::Stdout,
                            stdout,
                        );
                    }
                    if let Some(stderr) = stderr {
                        server_log_pipeline::spawn_server_output_reader(
                            id.to_string(),
                            LogStream::Stderr,
                            stderr,
                        );
                    }
                }
            }

            {
                let mut servers = self.lock_servers()?;
                if let Some(s) = servers.iter_mut().find(|s| s.id == id) {
                    s.last_started_at = Some(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map_err(|e| {
                                manager_t1("server.manager.current_time_failed", e.to_string())
                            })?
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
    #[allow(dead_code)]
    pub fn get_server_list(&self) -> Vec<ServerInstance> {
        self.get_server_list_checked().unwrap_or_default()
    }

    pub fn get_server_list_checked(&self) -> Result<Vec<ServerInstance>, String> {
        self.lock_servers().map(|servers| servers.clone())
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
            let server = self.find_server_clone(id)?;
            let status = self.get_server_status(id);
            let has_tracked_process = self.lock_processes()?.contains_key(id);
            if has_tracked_process || status_blocks_start(&status) {
                self.stop_server(id)?;
            }

            server_log_pipeline::shutdown_writer(id);

            let dir = std::path::Path::new(&server.path);
            if dir.exists() {
                std::fs::remove_dir_all(dir).map_err(|e| {
                    manager_t1("server.manager.delete_server_dir_failed", e.to_string())
                })?;
            }

            self.lock_servers()?.retain(|s| s.id != id);
            let data_dir = self.data_dir_value()?;
            remove_run_path_mapping(&data_dir, id)?;
            self.save()?;
            Ok(())
        })();

        log_manager_result("delete", &detail, result)
    }

    /// 读取当前正在运行的服务器 ID 列表
    pub fn get_running_server_ids_checked(&self) -> Result<Vec<String>, String> {
        Ok(self
            .get_server_list_checked()?
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
            .collect())
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
            Err(manager_t("server.manager.server_not_found_short"))
        }
    }

    pub fn update_server_java_path(
        &self,
        id: &str,
        new_java_path: &str,
    ) -> Result<ServerInstance, String> {
        let validated_java = crate::services::java_detector::validate_java(new_java_path)
            .map_err(|e| manager_t1("server.manager.java_path_validate_failed", e.to_string()))?;

        let mut servers = self.lock_servers()?;
        if let Some(server) = servers.iter_mut().find(|s| s.id == id) {
            let runtime = server
                .local_runtime_mut()
                .ok_or_else(|| manager_t("server.manager.update_java_path_unsupported"))?;
            runtime.java_path = validated_java.path;

            let updated_server = server.clone();
            drop(servers);
            self.save()?;
            Ok(updated_server)
        } else {
            Err(manager_t("server.manager.server_not_found_short"))
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
                return Err(manager_t("server.manager.server_running_update_path_blocked"));
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
                    .ok_or_else(|| manager_t("server.manager.update_startup_file_unsupported"))?;
                runtime.jar_path = jar_path.to_string();
            }

            // 如果提供了新的 startup_mode，则更新
            if let Some(startup_mode) = new_startup_mode {
                let runtime = server
                    .local_runtime_mut()
                    .ok_or_else(|| manager_t("server.manager.update_startup_mode_unsupported"))?;
                runtime.startup_mode = normalize_cli_startup_mode(Some(startup_mode))
                    .unwrap_or_else(|_| "jar".to_string());
            }

            // 路径更新后按共享规则刷新核心类型，但 custom/unknown 情况保留现有值
            server.core_type = refresh_local_server_core_type(
                &server.core_type,
                server.startup_mode_str(),
                server.jar_path(),
            );

            // 尝试从 server.properties 读取端口
            let server_properties_path = std::path::Path::new(new_path).join("server.properties");
            if server_properties_path.exists() {
                if let Ok(props) =
                    read_properties(server_properties_path.to_str().unwrap_or_default())
                {
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
            update_run_path_mapping(&data_dir, id, new_path)?;

            Ok(updated_server)
        } else {
            Err(manager_t("server.manager.server_not_found_short"))
        }
    }

    /// 停止全部正在运行的服务器
    #[allow(dead_code)]
    pub fn stop_all_servers(&self) {
        let _ = self.stop_all_servers_checked();
    }

    pub fn stop_all_servers_checked(&self) -> Result<(), String> {
        let ids = self.get_running_server_ids_checked()?;
        for id in ids {
            self.stop_server(&id)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ServerManager;
    use crate::models::server::{
        CpuPolicyConfig, DockerBackendKind, DockerCommandMode, DockerItzgRuntimeConfig,
        JvmPresetConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
    };
    use crate::test_support::{lock_env, EnvGuard};
    use std::collections::BTreeMap;
    use std::process::{Child, Command};
    use std::sync::Arc;

    fn sample_mismatched_runtime_server(path: String) -> ServerInstance {
        ServerInstance {
            id: "delete-running-test".to_string(),
            name: "Delete Running Test".to_string(),
            aliases: Vec::new(),
            core_type: "paper".to_string(),
            core_version: "paper".to_string(),
            mc_version: "1.21.1".to_string(),
            path: path.clone(),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::DockerItzg(DockerItzgRuntimeConfig {
                image: "itzg/minecraft-server".to_string(),
                image_tag: "java21".to_string(),
                container_name: "delete-running-test".to_string(),
                type_value: "PAPER".to_string(),
                version: "1.21.1".to_string(),
                data_dir_mount: path.clone(),
                published_game_port: 25565,
                env: BTreeMap::new(),
                extra_ports: Vec::new(),
                volume_mounts: Vec::new(),
                docker_backend_kind: DockerBackendKind::Cli,
                command_mode: DockerCommandMode::Rcon,
                rcon: None,
                jvm_args: Vec::new(),
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    fn sample_local_server(path: String, jar_path: String, startup_mode: &str) -> ServerInstance {
        ServerInstance {
            id: "local-update-test".to_string(),
            name: "Local Update Test".to_string(),
            aliases: Vec::new(),
            core_type: "paper".to_string(),
            core_version: "paper".to_string(),
            mc_version: "1.21.1".to_string(),
            path,
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path,
                startup_mode: startup_mode.to_string(),
                custom_command: None,
                java_path: "C:/Java/bin/java.exe".to_string(),
                jvm_args: Vec::new(),
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    fn spawn_sleep_process() -> Child {
        #[cfg(windows)]
        {
            Command::new("powershell")
                .args(["-NoProfile", "-Command", "Start-Sleep -Seconds 30"])
                .spawn()
                .expect("sleep process should spawn")
        }

        #[cfg(not(windows))]
        {
            Command::new("sh")
                .args(["-c", "sleep 30"])
                .spawn()
                .expect("sleep process should spawn")
        }
    }

    #[test]
    fn get_running_server_ids_checked_surfaces_server_list_lock_failures() {
        let manager = Arc::new(ServerManager::new_checked().expect("manager should initialize"));
        let cloned = Arc::clone(&manager);
        let poison_thread = std::thread::spawn(move || {
            let _guard = cloned
                .servers
                .lock()
                .expect("servers lock should be acquired");
            panic!("poison server list lock");
        });
        assert!(poison_thread.join().is_err(), "poison thread should panic");

        let error = manager
            .get_running_server_ids_checked()
            .expect_err("lock failure should not be silently treated as no running servers");

        assert_eq!(error, "servers lock poisoned");
    }

    #[test]
    fn is_starting_checked_surfaces_starting_lock_failures() {
        let manager = Arc::new(ServerManager::new_checked().expect("manager should initialize"));
        let cloned = Arc::clone(&manager);
        let poison_thread = std::thread::spawn(move || {
            let _guard = cloned
                .starting_servers
                .lock()
                .expect("starting lock should be acquired");
            panic!("poison starting lock");
        });
        assert!(poison_thread.join().is_err(), "poison thread should panic");

        let error = manager
            .is_starting_checked("alpha")
            .expect_err("lock failure should not be silently treated as not starting");

        assert_eq!(error, "starting_servers lock poisoned");
    }

    #[test]
    fn is_stopping_checked_surfaces_stopping_lock_failures() {
        let manager = Arc::new(ServerManager::new_checked().expect("manager should initialize"));
        let cloned = Arc::clone(&manager);
        let poison_thread = std::thread::spawn(move || {
            let _guard = cloned
                .stopping_servers
                .lock()
                .expect("stopping lock should be acquired");
            panic!("poison stopping lock");
        });
        assert!(poison_thread.join().is_err(), "poison thread should panic");

        let error = manager
            .is_stopping_checked("alpha")
            .expect_err("lock failure should not be silently treated as not stopping");

        assert_eq!(error, "stopping_servers lock poisoned");
    }

    #[test]
    fn delete_server_surfaces_missing_server_instead_of_succeeding() {
        let manager = ServerManager::new_checked().expect("manager should initialize");

        let error = manager
            .delete_server("missing-server")
            .expect_err("missing server delete should not silently succeed");

        assert_eq!(error, "未找到服务器: missing-server");
    }

    #[test]
    fn delete_server_aborts_when_stop_fails_for_running_like_state() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let server_dir = temp_dir.path().join("server");
        std::fs::create_dir_all(&server_dir).expect("server dir should exist");

        let manager = ServerManager::new_checked().expect("manager should initialize");
        manager
            .lock_servers()
            .expect("servers lock should succeed")
            .push(sample_mismatched_runtime_server(server_dir.to_string_lossy().to_string()));
        manager
            .lock_processes()
            .expect("processes lock should succeed")
            .insert("delete-running-test".to_string(), spawn_sleep_process());

        let error = manager
            .delete_server("delete-running-test")
            .expect_err("delete should stop when pre-delete stop fails");

        assert!(error.contains("服务器运行时声明与配置不一致"), "unexpected error: {}", error);
        assert!(server_dir.exists(), "server dir should remain when delete aborts");
        assert!(manager
            .find_server_clone_optional("delete-running-test")
            .expect("server lookup should succeed")
            .is_some());

        let tracked_child = {
            manager
                .lock_processes()
                .expect("processes lock should succeed")
                .remove("delete-running-test")
        };

        if let Some(mut child) = tracked_child {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    #[test]
    fn new_checked_recovers_invalid_server_registry_instead_of_starting_empty() {
        let _env_lock = lock_env();
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let data_file = temp_dir.path().join(crate::utils::constants::DATA_FILE);
        std::fs::write(&data_file, "{").expect("broken server registry should be written");
        let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &temp_dir.path().to_string_lossy());

        let manager = ServerManager::new_checked()
            .expect("invalid server registry should recover during bootstrap");
        let servers = manager
            .lock_servers()
            .expect("servers lock should succeed after bootstrap recovery")
            .clone();

        assert!(servers.is_empty(), "recovered registry should start empty");

        let backup_count = std::fs::read_dir(temp_dir.path())
            .expect("read temp dir")
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .contains(&format!("{}.bak-corrupt-", crate::utils::constants::DATA_FILE))
            })
            .count();

        assert_eq!(backup_count, 1);
        drop(_guard);
        drop(_env_lock);
    }

    #[test]
    fn update_server_path_refreshes_local_core_type_via_shared_resolution() {
        let _env_lock = lock_env();
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &temp_dir.path().to_string_lossy());
        let source_dir = temp_dir.path().join("nukkit-dir");
        std::fs::create_dir_all(&source_dir).expect("server dir should create");
        let jar_path = source_dir.join("nukkit.jar");
        std::fs::write(&jar_path, b"placeholder").expect("jar should write");

        let manager = ServerManager::new();
        manager
            .lock_servers()
            .expect("servers lock should succeed")
            .push(sample_local_server(
                source_dir.to_string_lossy().to_string(),
                jar_path.to_string_lossy().to_string(),
                "jar",
            ));

        let updated = manager
            .update_server_path(
                "local-update-test",
                source_dir.to_string_lossy().as_ref(),
                Some(jar_path.to_string_lossy().as_ref()),
                Some("jar"),
            )
            .expect("update path should succeed");

        assert_eq!(updated.core_type, "nukkit");
    }

    #[test]
    fn update_server_path_preserves_local_core_type_for_custom_mode() {
        let _env_lock = lock_env();
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &temp_dir.path().to_string_lossy());
        let source_dir = temp_dir.path().join("custom-dir");
        std::fs::create_dir_all(&source_dir).expect("server dir should create");
        let script_path = source_dir.join("run.bat");
        std::fs::write(&script_path, b"@echo off\r\n").expect("script should write");

        let manager = ServerManager::new();
        manager
            .lock_servers()
            .expect("servers lock should succeed")
            .push(sample_local_server(
                source_dir.to_string_lossy().to_string(),
                script_path.to_string_lossy().to_string(),
                "custom",
            ));

        let updated = manager
            .update_server_path(
                "local-update-test",
                source_dir.to_string_lossy().as_ref(),
                Some(script_path.to_string_lossy().as_ref()),
                Some("custom"),
            )
            .expect("update path should succeed");

        assert_eq!(updated.core_type, "paper");
    }
}
