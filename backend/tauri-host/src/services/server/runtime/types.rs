use crate::models::server::{ServerInstance, ServerStatus};
use crate::services::server::manager::ServerManager;
use std::process::Child;

pub struct RuntimeStartRequest<'a> {
    pub server_id: &'a str,
    pub server: &'a ServerInstance,
}

pub struct RuntimeStartResult {
    pub process_handle: Option<RuntimeProcessHandle>,
    pub fallback: Option<crate::services::server::manager::StartFallbackInfo>,
}

pub enum RuntimeProcessHandle {
    LocalChild(Child),
}

pub struct RuntimeStatusSnapshot {
    pub status: ServerStatus,
    pub pid: Option<u32>,
    pub detail_message: Option<String>,
    pub error_message: Option<String>,
}

pub struct RuntimeForceStopPreparation {
    pub supported: bool,
}

pub trait ServerRuntime: Send + Sync {
    fn start(&self, request: RuntimeStartRequest<'_>) -> Result<RuntimeStartResult, String>;

    fn start_with_manager(
        &self,
        _manager: &ServerManager,
        request: RuntimeStartRequest<'_>,
    ) -> Result<RuntimeStartResult, String> {
        self.start(request)
    }

    fn send_command(&self, server: &ServerInstance, command: &str) -> Result<(), String>;

    fn send_command_with_manager(
        &self,
        _manager: &ServerManager,
        server: &ServerInstance,
        command: &str,
    ) -> Result<(), String> {
        self.send_command(server, command)
    }

    fn request_stop(&self, server: &ServerInstance) -> Result<(), String>;

    fn request_stop_with_manager(
        &self,
        _manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<(), String> {
        self.request_stop(server)
    }

    fn stop(&self, server: &ServerInstance) -> Result<(), String>;

    fn stop_with_manager(
        &self,
        _manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<(), String> {
        self.stop(server)
    }

    fn prepare_force_stop(
        &self,
        server: &ServerInstance,
    ) -> Result<RuntimeForceStopPreparation, String>;

    fn prepare_force_stop_with_manager(
        &self,
        _manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<RuntimeForceStopPreparation, String> {
        self.prepare_force_stop(server)
    }

    fn force_stop(&self, server: &ServerInstance) -> Result<(), String>;

    fn force_stop_with_manager(
        &self,
        _manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<(), String> {
        self.force_stop(server)
    }

    fn status(&self, server: &ServerInstance) -> Result<RuntimeStatusSnapshot, String>;

    fn status_with_manager(
        &self,
        _manager: &ServerManager,
        server: &ServerInstance,
    ) -> Result<RuntimeStatusSnapshot, String> {
        self.status(server)
    }
}
