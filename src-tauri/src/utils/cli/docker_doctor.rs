use std::path::Path;
use std::process::Command;

use super::cli_env::{
    configured_docker_rcon_host_override, configured_servers_container_root,
    configured_servers_host_root, configured_web_bind_host, effective_cli_web_bind_host,
    is_container_like_environment, is_headless_http_environment,
};
use super::server_shared::{trace_docker_action, trace_docker_error};
use crate::utils::docker_cli::{
    docker_executable_path, ensure_docker_command_success,
    inspect_docker_image_reference_with_soft_failures, DockerImageAvailability,
    DockerImageInspectOutcome,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DoctorStatus {
    Pass,
    Warn,
    Fail,
}

impl DoctorStatus {
    fn as_label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DoctorCheck {
    name: &'static str,
    status: DoctorStatus,
    detail: String,
    hint: Option<String>,
}

#[derive(Debug, Clone)]
struct DockerDoctorContext {
    is_container_like: bool,
    is_headless_http: bool,
    effective_web_bind: String,
    configured_web_bind: Option<String>,
    docker_rcon_host_override: Option<String>,
    servers_container_root: Option<String>,
    servers_host_root: Option<String>,
    docker_executable: Option<String>,
    docker_info: Option<Result<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DockerDoctorReport {
    checks: Vec<DoctorCheck>,
    effective_rcon_host: String,
}

impl DockerDoctorReport {
    fn has_failures(&self) -> bool {
        self.checks
            .iter()
            .any(|check| check.status == DoctorStatus::Fail)
    }

    fn counts(&self) -> (usize, usize, usize) {
        self.checks
            .iter()
            .fold((0, 0, 0), |acc, check| match check.status {
                DoctorStatus::Pass => (acc.0 + 1, acc.1, acc.2),
                DoctorStatus::Warn => (acc.0, acc.1 + 1, acc.2),
                DoctorStatus::Fail => (acc.0, acc.1, acc.2 + 1),
            })
    }
}

impl DockerDoctorContext {
    fn has_path_mapping(&self) -> bool {
        self.servers_host_root
            .as_deref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
            && self
                .servers_container_root
                .as_deref()
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
    }
}

pub(super) fn handle_docker_command(args: &[String]) -> i32 {
    trace_docker_action("invoke", &format!("args={}", args.join(" | ")));
    match args.first().map(String::as_str) {
        None | Some("help") | Some("--help") | Some("-h") => {
            print_docker_help();
            0
        }
        Some("doctor") => run_docker_doctor(),
        Some("image-check") => run_docker_image_check(&args[1..]),
        Some("pull") => run_docker_pull(&args[1..]),
        Some(other) => {
            trace_docker_error(
                "unknown_subcommand",
                &format!("subcommand={}", other),
                "unsupported docker subcommand",
            );
            eprintln!("未知 docker 子命令: {}", other);
            print_docker_help();
            2
        }
    }
}

pub(super) fn print_docker_help() {
    println!("用法: sealantern docker <子命令>");
    println!("  doctor    检查 Docker CLI、守护进程、RCON 宿主地址和容器路径映射");
    println!("  image-check <image[:tag]>    检查目标镜像是否本地已缓存或可从远端解析");
    println!("  pull <image[:tag]>    主动拉取目标镜像，便于后续 server create/start 重试");
}

fn run_docker_image_check(args: &[String]) -> i32 {
    let Some(image_ref) = args
        .first()
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
    else {
        eprintln!("用法: sealantern docker image-check <image[:tag]>");
        return 2;
    };

    trace_docker_action("image_check", &format!("image_ref={}", image_ref));

    match inspect_docker_image_reference_with_soft_failures(image_ref) {
        Ok(DockerImageInspectOutcome::Available(availability)) => {
            println!("Docker 镜像检查通过");
            println!("  image_ref: {}", image_ref);
            println!(
                "  availability: {}",
                match availability {
                    DockerImageAvailability::LocalCached => "local_cached",
                    DockerImageAvailability::RemoteResolvable => "remote_resolvable",
                }
            );
            println!(
                "  detail: {}",
                match availability {
                    DockerImageAvailability::LocalCached => {
                        "当前宿主机已缓存该镜像，可直接创建容器"
                    }
                    DockerImageAvailability::RemoteResolvable => {
                        "本地未缓存，但远端仓库可解析该镜像引用"
                    }
                }
            );
            trace_docker_action(
                "image_check_completed",
                &format!("image_ref={} result=ok", image_ref),
            );
            0
        }
        Ok(DockerImageInspectOutcome::SoftFailure { failure_kind, message }) => {
            println!("Docker 镜像检查未确认可用性");
            println!("  image_ref: {}", image_ref);
            println!("  availability: unknown_soft_failure");
            println!("  failure_kind: {:?}", failure_kind);
            println!("  detail: {}", message);
            println!(
                "  hint: 如你仅做离线预配置，可继续 create-only / compose；真正 start 仍需要本地已有镜像或网络恢复"
            );
            println!("  hint: 如需主动预热镜像，可执行 sealantern docker pull {}", image_ref);
            trace_docker_action(
                "image_check_soft_failure",
                &format!("image_ref={} failure_kind={:?}", image_ref, failure_kind),
            );
            1
        }
        Err(err) => {
            eprintln!("Docker 镜像检查失败: {}", err);
            trace_docker_error("image_check_failed", &format!("image_ref={}", image_ref), &err);
            2
        }
    }
}

fn run_docker_pull(args: &[String]) -> i32 {
    let Some(image_ref) = args
        .first()
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
    else {
        eprintln!("用法: sealantern docker pull <image[:tag]>");
        return 2;
    };

    trace_docker_action("pull_start", &format!("image_ref={}", image_ref));

    let docker_path = match docker_executable_path() {
        Ok(path) => path,
        Err(err) => {
            trace_docker_error("pull_prepare_failed", &format!("image_ref={}", image_ref), &err);
            eprintln!("Docker 镜像拉取失败: {}", err);
            return 2;
        }
    };

    let output = match Command::new(&docker_path)
        .args(["pull", image_ref])
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            let message = format!("执行 docker pull 失败: {}", err);
            trace_docker_error("pull_spawn_failed", &format!("image_ref={}", image_ref), &message);
            eprintln!("Docker 镜像拉取失败: {}", message);
            return 2;
        }
    };

    match ensure_docker_command_success(output, "docker pull", Some(image_ref), None) {
        Ok(()) => {
            println!("Docker 镜像拉取完成");
            println!("  image_ref: {}", image_ref);
            println!("  next: sealantern server start <server-id>");
            trace_docker_action("pull_completed", &format!("image_ref={} result=ok", image_ref));
            0
        }
        Err(err) => {
            trace_docker_error("pull_failed", &format!("image_ref={}", image_ref), &err);
            eprintln!("Docker 镜像拉取失败: {}", err);
            2
        }
    }
}

fn run_docker_doctor() -> i32 {
    trace_docker_action("doctor_start", "");
    let context = collect_docker_doctor_context();
    let report = build_docker_doctor_report(&context);
    print_docker_doctor_report(&report, &context);

    if report.has_failures() {
        trace_docker_error("doctor_completed", "", "report_has_failures");
        2
    } else {
        trace_docker_action("doctor_completed", "result=ok");
        0
    }
}

fn collect_docker_doctor_context() -> DockerDoctorContext {
    let docker_executable = resolve_docker_executable_path();
    let docker_info = docker_executable.as_deref().map(run_docker_info_check);

    DockerDoctorContext {
        is_container_like: is_container_like_environment(),
        is_headless_http: is_headless_http_environment(),
        effective_web_bind: effective_cli_web_bind_host(),
        configured_web_bind: configured_web_bind_host(),
        docker_rcon_host_override: configured_docker_rcon_host_override(),
        servers_container_root: configured_servers_container_root()
            .map(|path| path.to_string_lossy().to_string()),
        servers_host_root: configured_servers_host_root()
            .map(|path| path.to_string_lossy().to_string()),
        docker_executable,
        docker_info,
    }
}

fn resolve_docker_executable_path() -> Option<String> {
    docker_executable_path().ok()
}

fn run_docker_info_check(docker_executable: &str) -> Result<String, String> {
    let output = Command::new(docker_executable)
        .args(["info", "--format", "{{.ServerVersion}}"])
        .output()
        .map_err(|err| format!("执行 docker info 失败: {}", err))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            Ok("docker info 返回成功".to_string())
        } else {
            Ok(format!("Docker ServerVersion={}", stdout))
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        Err(if detail.is_empty() {
            "docker info 返回失败，未给出更多输出".to_string()
        } else {
            format!("docker info 返回失败: {}", detail)
        })
    }
}

fn build_docker_doctor_report(context: &DockerDoctorContext) -> DockerDoctorReport {
    let effective_rcon_host = context
        .docker_rcon_host_override
        .clone()
        .unwrap_or_else(|| {
            if context.is_container_like {
                "host.docker.internal".to_string()
            } else {
                "127.0.0.1".to_string()
            }
        });

    let runtime_detail = if context.is_container_like {
        format!(
            "当前环境按容器/Headless 逻辑处理，effective_web_bind={}",
            context.effective_web_bind
        )
    } else {
        format!(
            "当前环境按桌面/宿主机逻辑处理，effective_web_bind={}",
            context.effective_web_bind
        )
    };

    let docker_cli_check = match &context.docker_executable {
        Some(path) => DoctorCheck {
            name: "docker_cli",
            status: DoctorStatus::Pass,
            detail: format!("已找到 Docker CLI: {}", path),
            hint: None,
        },
        None => DoctorCheck {
            name: "docker_cli",
            status: DoctorStatus::Fail,
            detail: "PATH 中未找到 docker / docker.exe".to_string(),
            hint: Some(
                "请先安装 Docker Desktop 或 Docker Engine，并确认 docker 命令可直接执行"
                    .to_string(),
            ),
        },
    };

    let docker_daemon_check = match &context.docker_info {
        Some(Ok(detail)) => DoctorCheck {
            name: "docker_daemon",
            status: DoctorStatus::Pass,
            detail: detail.clone(),
            hint: None,
        },
        Some(Err(detail)) => DoctorCheck {
            name: "docker_daemon",
            status: DoctorStatus::Fail,
            detail: detail.clone(),
            hint: Some("请确认 Docker 守护进程已经启动，且当前用户具备访问权限".to_string()),
        },
        None => DoctorCheck {
            name: "docker_daemon",
            status: DoctorStatus::Fail,
            detail: "未执行 docker info，因为 Docker CLI 尚不可用".to_string(),
            hint: Some("先修复 docker_cli 检查项，再重新运行 doctor".to_string()),
        },
    };

    let mapping_check = if !context.is_container_like {
        DoctorCheck {
            name: "path_mapping",
            status: DoctorStatus::Pass,
            detail: "当前不是容器化路径场景，Docker 数据目录映射不是必选项".to_string(),
            hint: None,
        }
    } else if context.has_path_mapping() {
        DoctorCheck {
            name: "path_mapping",
            status: DoctorStatus::Pass,
            detail: format!(
                "已配置容器路径映射: container_root={} -> host_root={}",
                context
                    .servers_container_root
                    .clone()
                    .unwrap_or_else(|| "<missing>".to_string()),
                context
                    .servers_host_root
                    .clone()
                    .unwrap_or_else(|| "<missing>".to_string())
            ),
            hint: None,
        }
    } else {
        DoctorCheck {
            name: "path_mapping",
            status: DoctorStatus::Fail,
            detail: "当前环境会按容器/Headless 逻辑创建 Docker 服务器，但未配置 SEALANTERN_SERVERS_CONTAINER_ROOT / SEALANTERN_SERVERS_HOST_ROOT".to_string(),
            hint: Some("如果要让 Sea Lantern 在容器里创建 itzg 容器，请显式设置这两个变量，或在 server 命令里直接传 --data-dir".to_string()),
        }
    };

    let web_bind_check = if context.is_container_like && context.effective_web_bind == "127.0.0.1" {
        DoctorCheck {
            name: "web_bind",
            status: DoctorStatus::Warn,
            detail:
                "当前容器化/Headless 环境下仍绑定到 127.0.0.1，容器外通常无法访问 CLI Web 控制台"
                    .to_string(),
            hint: Some(
                "当前默认只监听 loopback；如需容器外访问，请显式设置 SEALANTERN_HTTP_BIND=0.0.0.0:3000，或指定一个实际需要监听的地址"
                    .to_string(),
            ),
        }
    } else {
        DoctorCheck {
            name: "web_bind",
            status: DoctorStatus::Pass,
            detail: match &context.configured_web_bind {
                Some(bind) => format!("已显式配置 CLI Web 绑定地址: {}", bind),
                None => format!(
                    "CLI Web 将使用默认绑定地址: {}（仅本地回环可访问，除非显式改绑）",
                    context.effective_web_bind
                ),
            },
            hint: None,
        }
    };

    let rcon_check = if context.is_container_like && context.docker_rcon_host_override.is_none() {
        DoctorCheck {
            name: "rcon_host",
            status: DoctorStatus::Warn,
            detail: format!(
                "当前将使用默认 Docker RCON 宿主地址: {}",
                effective_rcon_host
            ),
            hint: Some("在部分 Linux / 自定义网络环境里，host.docker.internal 可能不可用；必要时请显式设置 SEALANTERN_DOCKER_RCON_HOST".to_string()),
        }
    } else {
        DoctorCheck {
            name: "rcon_host",
            status: DoctorStatus::Pass,
            detail: format!("Docker RCON 宿主地址将使用: {}", effective_rcon_host),
            hint: None,
        }
    };

    let container_root_check = match &context.servers_container_root {
        Some(container_root) if !Path::new(container_root).exists() => DoctorCheck {
            name: "container_root_visibility",
            status: DoctorStatus::Warn,
            detail: format!(
                "已配置 SEALANTERN_SERVERS_CONTAINER_ROOT={}, 但当前进程内看不到这个目录",
                container_root
            ),
            hint: Some(
                "如果 Sea Lantern 自己运行在容器里，请确认该目录已挂载进当前容器".to_string(),
            ),
        },
        Some(container_root) => DoctorCheck {
            name: "container_root_visibility",
            status: DoctorStatus::Pass,
            detail: format!(
                "当前进程可以访问 SEALANTERN_SERVERS_CONTAINER_ROOT={}",
                container_root
            ),
            hint: None,
        },
        None => DoctorCheck {
            name: "container_root_visibility",
            status: if context.is_container_like {
                DoctorStatus::Warn
            } else {
                DoctorStatus::Pass
            },
            detail: if context.is_container_like {
                "当前未配置 SEALANTERN_SERVERS_CONTAINER_ROOT，因此不会验证容器内 servers 根目录可见性".to_string()
            } else {
                "当前不是容器化路径场景，无需检查容器内 servers 根目录".to_string()
            },
            hint: None,
        },
    };

    DockerDoctorReport {
        checks: vec![
            DoctorCheck {
                name: "runtime_mode",
                status: DoctorStatus::Pass,
                detail: runtime_detail,
                hint: None,
            },
            docker_cli_check,
            docker_daemon_check,
            mapping_check,
            web_bind_check,
            rcon_check,
            container_root_check,
        ],
        effective_rcon_host,
    }
}

fn print_docker_doctor_report(report: &DockerDoctorReport, context: &DockerDoctorContext) {
    println!("Docker 运行环境自检");
    println!(
        "  runtime_mode: {}",
        if context.is_container_like {
            "container_like"
        } else {
            "desktop"
        }
    );
    println!(
        "  http_mode: {}",
        if context.is_headless_http {
            "headless_http"
        } else {
            "desktop_web"
        }
    );
    println!("  effective_web_bind: {}", context.effective_web_bind);
    println!("  effective_rcon_host: {}", report.effective_rcon_host);
    println!();

    for check in &report.checks {
        println!("[{}] {}", check.status.as_label(), check.name);
        println!("  {}", check.detail);
        if let Some(hint) = &check.hint {
            println!("  hint: {}", hint);
        }
    }

    let (pass_count, warn_count, fail_count) = report.counts();
    println!();
    println!("summary: pass={}, warn={}, fail={}", pass_count, warn_count, fail_count);
    if report.has_failures() {
        println!("doctor 结果: 存在阻断项，当前 Docker 运行链路尚未就绪");
    } else if warn_count > 0 {
        println!("doctor 结果: 基本可用，但仍建议处理 WARN 项以减少部署差异");
    } else {
        println!("doctor 结果: 当前 Docker 运行链路检查通过");
    }
}

#[cfg(test)]
mod tests {
    use super::{build_docker_doctor_report, DockerDoctorContext, DoctorStatus};
    use crate::utils::docker_cli::{DockerCommandFailureKind, DockerImageInspectOutcome};

    fn render_docker_help_output() -> String {
        let mut lines = Vec::new();
        lines.push("用法: sealantern docker <子命令>".to_string());
        lines
            .push("  doctor    检查 Docker CLI、守护进程、RCON 宿主地址和容器路径映射".to_string());
        lines.push(
            "  image-check <image[:tag]>    检查目标镜像是否本地已缓存或可从远端解析".to_string(),
        );
        lines.push(
            "  pull <image[:tag]>    主动拉取目标镜像，便于后续 server create/start 重试"
                .to_string(),
        );
        lines.join("\n")
    }

    fn sample_context() -> DockerDoctorContext {
        DockerDoctorContext {
            is_container_like: false,
            is_headless_http: false,
            effective_web_bind: "127.0.0.1".to_string(),
            configured_web_bind: None,
            docker_rcon_host_override: None,
            servers_container_root: None,
            servers_host_root: None,
            docker_executable: Some("C:/Docker/docker.exe".to_string()),
            docker_info: Some(Ok("Docker ServerVersion=27.0.0".to_string())),
        }
    }

    #[test]
    fn doctor_report_passes_desktop_mode_without_mapping() {
        std::env::remove_var("SEALANTERN_SERVERS_HOST_ROOT");
        std::env::remove_var("SEALANTERN_SERVERS_CONTAINER_ROOT");

        let report = build_docker_doctor_report(&sample_context());

        assert!(!report.has_failures());
        assert_eq!(report.effective_rcon_host, "127.0.0.1");
        assert!(report
            .checks
            .iter()
            .any(|check| { check.name == "path_mapping" && check.status == DoctorStatus::Pass }));
    }

    #[test]
    fn doctor_report_fails_when_container_like_mapping_is_missing() {
        std::env::remove_var("SEALANTERN_SERVERS_HOST_ROOT");
        std::env::remove_var("SEALANTERN_SERVERS_CONTAINER_ROOT");

        let mut context = sample_context();
        context.is_container_like = true;
        context.is_headless_http = true;
        context.effective_web_bind = "0.0.0.0".to_string();

        let report = build_docker_doctor_report(&context);

        assert!(report.has_failures());
        assert!(report
            .checks
            .iter()
            .any(|check| { check.name == "path_mapping" && check.status == DoctorStatus::Fail }));
    }

    #[test]
    fn doctor_report_warns_when_container_like_rcon_host_is_implicit() {
        std::env::set_var("SEALANTERN_SERVERS_HOST_ROOT", "E:/srv/sealantern/servers");
        std::env::set_var("SEALANTERN_SERVERS_CONTAINER_ROOT", "/app/data/servers");

        let mut context = sample_context();
        context.is_container_like = true;
        context.is_headless_http = true;
        context.effective_web_bind = "0.0.0.0".to_string();
        context.servers_host_root = Some("E:/srv/sealantern/servers".to_string());
        context.servers_container_root = Some("/app/data/servers".to_string());

        let report = build_docker_doctor_report(&context);

        assert_eq!(report.effective_rcon_host, "host.docker.internal");
        assert!(report
            .checks
            .iter()
            .any(|check| { check.name == "rcon_host" && check.status == DoctorStatus::Warn }));

        std::env::remove_var("SEALANTERN_SERVERS_HOST_ROOT");
        std::env::remove_var("SEALANTERN_SERVERS_CONTAINER_ROOT");
    }

    #[test]
    fn doctor_report_warns_loopback_bind_in_container_like_mode_with_explicit_guidance() {
        let mut context = sample_context();
        context.is_container_like = true;
        context.is_headless_http = true;
        context.effective_web_bind = "127.0.0.1".to_string();

        let report = build_docker_doctor_report(&context);
        let check = report
            .checks
            .iter()
            .find(|check| check.name == "web_bind")
            .expect("web_bind check");

        assert_eq!(check.status, DoctorStatus::Warn);
        assert!(check.detail.contains("127.0.0.1"));
        assert!(check
            .hint
            .as_deref()
            .unwrap_or_default()
            .contains("SEALANTERN_HTTP_BIND=0.0.0.0:3000"));
    }

    #[test]
    fn doctor_report_describes_default_bind_as_loopback_when_not_explicitly_configured() {
        let report = build_docker_doctor_report(&sample_context());
        let check = report
            .checks
            .iter()
            .find(|check| check.name == "web_bind")
            .expect("web_bind check");

        assert_eq!(check.status, DoctorStatus::Pass);
        assert!(check.detail.contains("默认绑定地址: 127.0.0.1"));
        assert!(check.detail.contains("仅本地回环可访问"));
    }

    #[test]
    fn doctor_report_treats_explicit_external_bind_as_configuration_not_default() {
        let mut context = sample_context();
        context.is_container_like = true;
        context.is_headless_http = true;
        context.effective_web_bind = "0.0.0.0".to_string();
        context.configured_web_bind = Some("0.0.0.0:3000".to_string());

        let report = build_docker_doctor_report(&context);
        let check = report
            .checks
            .iter()
            .find(|check| check.name == "web_bind")
            .expect("web_bind check");

        assert_eq!(check.status, DoctorStatus::Pass);
        assert!(check.detail.contains("已显式配置 CLI Web 绑定地址: 0.0.0.0:3000"));
        assert!(!check.detail.contains("默认绑定地址"));
    }

    #[test]
    fn docker_help_mentions_pull_subcommand() {
        let help = render_docker_help_output();

        assert!(help.contains("pull <image[:tag]>"));
    }

    #[test]
    fn docker_image_soft_failure_shape_can_be_reported_without_hard_failure() {
        let outcome = DockerImageInspectOutcome::SoftFailure {
            failure_kind: DockerCommandFailureKind::Timeout,
            message: "docker manifest inspect 超时".to_string(),
        };

        match outcome {
            DockerImageInspectOutcome::SoftFailure { failure_kind, message } => {
                assert_eq!(failure_kind, DockerCommandFailureKind::Timeout);
                assert!(message.contains("超时"));
            }
            DockerImageInspectOutcome::Available(_) => panic!("expected soft failure"),
        }
    }
}
