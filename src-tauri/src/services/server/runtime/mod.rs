mod control;
pub mod docker_itzg;
pub mod local;
pub mod local_helper;

mod types;

use once_cell::sync::Lazy;

use crate::models::server::{ServerInstance, ServerRuntimeConfig};

pub use types::{
    RuntimeForceStopPreparation, RuntimeProcessHandle, RuntimeStartRequest, RuntimeStartResult,
    RuntimeStatusSnapshot, ServerRuntime,
};

static LOCAL_RUNTIME: local::LocalServerRuntime = local::LocalServerRuntime;
static DOCKER_ITZG_RUNTIME: Lazy<docker_itzg::DockerItzgRuntime> =
    Lazy::new(docker_itzg::DockerItzgRuntime::new);

pub fn resolve_runtime(server: &ServerInstance) -> Result<&'static dyn ServerRuntime, String> {
    match (server.runtime_kind.as_str(), &server.runtime) {
        ("local", ServerRuntimeConfig::Local(_)) => Ok(&LOCAL_RUNTIME),
        ("docker_itzg", ServerRuntimeConfig::DockerItzg(_)) => Ok(&*DOCKER_ITZG_RUNTIME),
        (kind, ServerRuntimeConfig::Local(_)) => Err(format!(
            "服务器运行时声明与配置不一致: runtime_kind={}, runtime.kind=local",
            kind
        )),
        (kind, ServerRuntimeConfig::DockerItzg(_)) => Err(format!(
            "服务器运行时声明与配置不一致: runtime_kind={}, runtime.kind=docker_itzg",
            kind
        )),
    }
}
