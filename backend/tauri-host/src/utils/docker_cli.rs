use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, Instant};

use crate::utils::logger;
use crate::utils::path::find_executable_in_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DockerCommandFailureKind {
    Network,
    ImageNotFound,
    Auth,
    DaemonUnavailable,
    Timeout,
    Other,
}

const DOCKER_IMAGE_INSPECT_TIMEOUT_SECS: u64 = 5;

fn failed_output_with_message(message: String) -> Output {
    Output {
        status: exit_status_from_raw(1),
        stdout: Vec::new(),
        stderr: message.into_bytes(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DockerImageAvailability {
    LocalCached,
    RemoteResolvable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DockerImageInspectOutcome {
    Available(DockerImageAvailability),
    SoftFailure {
        failure_kind: DockerCommandFailureKind,
        message: String,
    },
}

pub(crate) fn docker_executable_path() -> Result<String, String> {
    let executable = if cfg!(windows) {
        "docker.exe"
    } else {
        "docker"
    };
    find_executable_in_path(executable)
        .map(|path| path.to_string_lossy().to_string())
        .ok_or_else(|| "未找到 docker 命令，请先安装并加入 PATH".to_string())
}

pub(crate) fn split_docker_image_reference_tag(image_ref: &str) -> (&str, Option<&str>) {
    let trimmed = image_ref.trim();
    if trimmed.is_empty() || trimmed.contains('@') {
        return (trimmed, None);
    }

    let last_slash = trimmed.rfind('/');
    let last_colon = trimmed.rfind(':');

    match (last_slash, last_colon) {
        (_, None) => (trimmed, None),
        (Some(slash), Some(colon)) if colon > slash => {
            let tag = &trimmed[colon + 1..];
            if tag.trim().is_empty() {
                (trimmed, None)
            } else {
                (&trimmed[..colon], Some(tag))
            }
        }
        (None, Some(colon)) => {
            let tag = &trimmed[colon + 1..];
            if tag.trim().is_empty() {
                (trimmed, None)
            } else {
                (&trimmed[..colon], Some(tag))
            }
        }
        _ => (trimmed, None),
    }
}

pub(crate) fn resolve_docker_image_and_tag(
    image: Option<&str>,
    image_tag: Option<&str>,
    default_image: &str,
    default_tag: &str,
) -> Result<(String, String), String> {
    let raw_image = image
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(default_image);

    if raw_image.contains('@') {
        return Err(
            "当前 CLI 暂不支持使用 Docker digest 作为 --image；请改用仓库名 + --image-tag，或直接传带 tag 的镜像引用"
                .to_string(),
        );
    }

    let (image_name, embedded_tag) = split_docker_image_reference_tag(raw_image);
    let resolved_image = image_name.trim();
    if resolved_image.is_empty() {
        return Err("Docker 镜像名不能为空".to_string());
    }

    let resolved_tag = image_tag
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| embedded_tag.map(str::to_string))
        .unwrap_or_else(|| default_tag.to_string());

    Ok((resolved_image.to_string(), resolved_tag))
}

pub(crate) fn format_docker_image_reference(image: &str, image_tag: &str) -> String {
    format!("{}:{}", image, image_tag)
}

pub(crate) fn inspect_docker_image_reference_with_soft_failures(
    image_ref: &str,
) -> Result<DockerImageInspectOutcome, String> {
    let docker_path = docker_executable_path()?;

    inspect_docker_image_reference_with_runner(
        &docker_path,
        image_ref,
        run_docker_command_with_timeout,
    )
}

fn inspect_docker_image_reference_with_runner<F>(
    docker_path: &str,
    image_ref: &str,
    mut run_command: F,
) -> Result<DockerImageInspectOutcome, String>
where
    F: FnMut(&str, &[&str], Duration, &str) -> Result<Output, String>,
{
    let local_output = match run_command(
        docker_path,
        &["image", "inspect", image_ref],
        Duration::from_secs(DOCKER_IMAGE_INSPECT_TIMEOUT_SECS),
        "docker image inspect",
    ) {
        Ok(output) => output,
        Err(message) => failed_output_with_message(message),
    };
    if local_output.status.success() {
        return Ok(DockerImageInspectOutcome::Available(DockerImageAvailability::LocalCached));
    }

    let manifest_output = match run_command(
        docker_path,
        &["manifest", "inspect", image_ref],
        Duration::from_secs(DOCKER_IMAGE_INSPECT_TIMEOUT_SECS),
        "docker manifest inspect",
    ) {
        Ok(output) => output,
        Err(message) => failed_output_with_message(message),
    };

    interpret_docker_image_inspect_outputs(image_ref, &local_output, &manifest_output)
}

fn interpret_docker_image_inspect_outputs(
    image_ref: &str,
    local_output: &Output,
    manifest_output: &Output,
) -> Result<DockerImageInspectOutcome, String> {
    if local_output.status.success() {
        return Ok(DockerImageInspectOutcome::Available(DockerImageAvailability::LocalCached));
    }

    if manifest_output.status.success() {
        return Ok(DockerImageInspectOutcome::Available(DockerImageAvailability::RemoteResolvable));
    }

    classify_manifest_inspect_outcome(image_ref, manifest_output)
}

fn classify_manifest_inspect_outcome(
    image_ref: &str,
    manifest_output: &Output,
) -> Result<DockerImageInspectOutcome, String> {
    let message = render_docker_command_error(
        "docker manifest inspect",
        manifest_output,
        Some(image_ref),
        None,
    );
    let failure_kind = classify_docker_command_failure(&message);

    match failure_kind {
        DockerCommandFailureKind::ImageNotFound | DockerCommandFailureKind::Auth => Err(message),
        DockerCommandFailureKind::Network
        | DockerCommandFailureKind::Timeout
        | DockerCommandFailureKind::DaemonUnavailable
        | DockerCommandFailureKind::Other => {
            Ok(DockerImageInspectOutcome::SoftFailure { failure_kind, message })
        }
    }
}

fn run_docker_command_with_timeout(
    docker_path: &str,
    args: &[&str],
    timeout: Duration,
    action: &str,
) -> Result<Output, String> {
    let mut child = Command::new(docker_path)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|err| format!("执行 {} 失败: {}", action, err))?;

    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .map_err(|err| format!("等待 {} 输出失败: {}", action, err));
            }
            Ok(None) => {
                if started.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "{} 超时: {}s 内未返回；已按软失败处理，可稍后重试或先执行 sealantern docker pull <image[:tag]>",
                        action,
                        timeout.as_secs()
                    ));
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(err) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("查询 {} 状态失败: {}", action, err));
            }
        }
    }
}

pub(crate) fn ensure_docker_command_success(
    output: Output,
    action: &str,
    image_ref: Option<&str>,
    container_name: Option<&str>,
) -> Result<(), String> {
    if output.status.success() {
        return Ok(());
    }

    Err(render_docker_command_error(action, &output, image_ref, container_name))
}

pub(crate) fn render_docker_command_error(
    action: &str,
    output: &Output,
    image_ref: Option<&str>,
    container_name: Option<&str>,
) -> String {
    let stderr = stderr_text(output);
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let raw = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("退出码: {:?}", output.status.code())
    };

    let failure_kind = classify_docker_command_failure(&raw);
    let image_detail = image_ref
        .map(|value| format!(" image={}", value))
        .unwrap_or_default();
    let container_detail = container_name
        .map(|value| format!(" container={}", value))
        .unwrap_or_default();
    let detail = format!("{}{}", image_detail, container_detail);

    logger::log_trace(&format!(
        "[utils.docker_cli] action=render_command_error action_name={} failure_kind={:?} image_ref={} container={} raw={}",
        action,
        failure_kind,
        image_ref.unwrap_or(""),
        container_name.unwrap_or(""),
        raw.replace('\n', " ")
    ));

    match failure_kind {
        DockerCommandFailureKind::Network => format!(
            "{} 失败:{} 镜像拉取网络不可达或超时。请检查 Docker Hub / 镜像仓库连通性、代理配置，或预先执行 `sealantern docker pull <image[:tag]>`。原始输出: {}",
            action, detail, raw
        ),
        DockerCommandFailureKind::ImageNotFound => format!(
            "{} 失败:{} 镜像或标签不存在。请检查 `--image` / `--image-tag`，例如可优先尝试 `itzg/minecraft-server:latest`。原始输出: {}",
            action, detail, raw
        ),
        DockerCommandFailureKind::Auth => format!(
            "{} 失败:{} 镜像仓库认证或访问被拒绝。请确认已登录对应 registry，且镜像可见。原始输出: {}",
            action, detail, raw
        ),
        DockerCommandFailureKind::DaemonUnavailable => format!(
            "{} 失败:{} Docker daemon 当前不可用。请先运行 `sealantern docker doctor` 或确认 Docker Desktop / Engine 已启动。原始输出: {}",
            action, detail, raw
        ),
        DockerCommandFailureKind::Timeout => format!(
            "{} 失败:{} Docker 镜像探测超时。不会阻断本地记录创建，但建议先执行 `sealantern docker pull <image[:tag]>` 或稍后重试。原始输出: {}",
            action, detail, raw
        ),
        DockerCommandFailureKind::Other => format!("{} 失败:{} {}", action, detail, raw),
    }
}

pub(crate) fn classify_docker_command_failure(message: &str) -> DockerCommandFailureKind {
    let lower = message.to_ascii_lowercase();
    if lower.contains("pull access denied")
        || lower.contains("requested access to the resource is denied")
        || lower.contains("unauthorized")
        || lower.contains("authentication required")
    {
        return DockerCommandFailureKind::Auth;
    }
    if lower.contains("manifest unknown")
        || lower.contains("not found") && lower.contains("manifest")
        || lower.contains("failed to resolve reference") && lower.contains("not found")
    {
        return DockerCommandFailureKind::ImageNotFound;
    }
    if lower.contains("cannot connect to the docker daemon")
        || lower.contains("failed to connect to the docker api")
        || lower.contains("open //./pipe/docker")
        || lower.contains("is the docker daemon running")
    {
        return DockerCommandFailureKind::DaemonUnavailable;
    }
    if lower.contains("connectex")
        || lower.contains("dial tcp")
        || lower.contains("i/o timeout")
        || lower.contains("tls handshake timeout")
        || lower.contains("no https proxy")
        || lower.contains("context deadline exceeded")
    {
        return DockerCommandFailureKind::Network;
    }
    if lower.contains("超时")
        || lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("deadline exceeded")
    {
        return DockerCommandFailureKind::Timeout;
    }

    DockerCommandFailureKind::Other
}

pub(crate) fn docker_error_indicates_missing_container(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("no such container")
        || lower.contains("no such object")
        || lower.contains("container") && lower.contains("not found")
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

#[cfg(test)]
pub(crate) fn interpret_docker_image_inspect_outputs_for_tests(
    image_ref: &str,
    local_output: &Output,
    manifest_output: &Output,
) -> Result<DockerImageInspectOutcome, String> {
    interpret_docker_image_inspect_outputs(image_ref, local_output, manifest_output)
}

#[cfg(test)]
mod tests {
    use super::{
        classify_docker_command_failure, classify_manifest_inspect_outcome,
        docker_error_indicates_missing_container, exit_status_from_raw,
        format_docker_image_reference, inspect_docker_image_reference_with_runner,
        interpret_docker_image_inspect_outputs, render_docker_command_error,
        resolve_docker_image_and_tag, split_docker_image_reference_tag, DockerCommandFailureKind,
        DockerImageAvailability, DockerImageInspectOutcome, DOCKER_IMAGE_INSPECT_TIMEOUT_SECS,
    };
    use std::process::Output;

    fn failed_output(stderr: &str) -> Output {
        Output {
            status: exit_status_from_raw(1),
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    fn successful_output(stdout: &str) -> Output {
        Output {
            status: exit_status_from_raw(0),
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
        }
    }

    fn failed_output_with_stdout(stdout: &str) -> Output {
        Output {
            status: exit_status_from_raw(1),
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
        }
    }

    #[test]
    fn failed_output_with_message_uses_stderr_without_console_spill() {
        let output = super::failed_output_with_message("probe timeout".to_string());
        assert_eq!(String::from_utf8_lossy(&output.stderr), "probe timeout");
    }

    #[test]
    fn classifies_network_failures() {
        let failure = classify_docker_command_failure(
            "failed to resolve reference: dial tcp 1.2.3.4:443: connectex: timeout",
        );

        assert_eq!(failure, DockerCommandFailureKind::Network);
    }

    #[test]
    fn classifies_command_timeout_failures() {
        let failure = classify_docker_command_failure(
            "docker manifest inspect 超时: 5s 内未返回；已按软失败处理",
        );

        assert_eq!(failure, DockerCommandFailureKind::Timeout);
    }

    #[test]
    fn classifies_manifest_unknown_as_image_not_found() {
        let failure = classify_docker_command_failure("manifest unknown: manifest unknown");

        assert_eq!(failure, DockerCommandFailureKind::ImageNotFound);
    }

    #[test]
    fn rendered_error_contains_image_and_container_context() {
        let output = failed_output("unauthorized: authentication required");
        let message = render_docker_command_error(
            "docker pull",
            &output,
            Some("itzg/minecraft-server:latest"),
            Some("sea-test"),
        );

        assert!(message.contains("image=itzg/minecraft-server:latest"));
        assert!(message.contains("container=sea-test"));
        assert!(message.contains("镜像仓库认证或访问被拒绝"));
    }

    #[test]
    fn render_docker_command_error_prefers_stdout_when_stderr_is_empty() {
        let output = failed_output_with_stdout("manifest unknown: manifest unknown");
        let message = render_docker_command_error(
            "docker manifest inspect",
            &output,
            Some("itzg/minecraft-server:bad-tag"),
            None,
        );

        assert!(message.contains("镜像或标签不存在"));
        assert!(message.contains("manifest unknown"));
    }

    #[test]
    fn render_docker_command_error_falls_back_to_exit_code_when_streams_are_empty() {
        let output = Output {
            status: exit_status_from_raw(7),
            stdout: Vec::new(),
            stderr: Vec::new(),
        };
        let message = render_docker_command_error(
            "docker manifest inspect",
            &output,
            Some("itzg/test:1"),
            None,
        );

        assert!(message.contains("退出码"));
        assert!(message.contains("image=itzg/test:1"));
    }

    #[test]
    fn docker_image_soft_failure_variant_carries_message_and_kind() {
        let outcome = DockerImageInspectOutcome::SoftFailure {
            failure_kind: DockerCommandFailureKind::Network,
            message: "network timeout".to_string(),
        };

        match outcome {
            DockerImageInspectOutcome::SoftFailure { failure_kind, message } => {
                assert_eq!(failure_kind, DockerCommandFailureKind::Network);
                assert_eq!(message, "network timeout");
            }
            DockerImageInspectOutcome::Available(DockerImageAvailability::LocalCached) => {
                panic!("expected soft failure")
            }
            DockerImageInspectOutcome::Available(DockerImageAvailability::RemoteResolvable) => {
                panic!("expected soft failure")
            }
        }
    }

    #[test]
    fn interpret_docker_image_inspect_outputs_prefers_local_cached_success() {
        let outcome = interpret_docker_image_inspect_outputs(
            "itzg/minecraft-server:latest",
            &successful_output("[{\"Id\":\"sha256:abc\"}]"),
            &failed_output("manifest unknown: manifest unknown"),
        )
        .expect("local inspect success should short-circuit");

        assert_eq!(
            outcome,
            DockerImageInspectOutcome::Available(DockerImageAvailability::LocalCached)
        );
    }

    #[test]
    fn interpret_docker_image_inspect_outputs_falls_back_to_remote_resolvable() {
        let outcome = interpret_docker_image_inspect_outputs(
            "itzg/minecraft-server:latest",
            &failed_output("Error: No such image: itzg/minecraft-server:latest"),
            &successful_output("{\"schemaVersion\":2}"),
        )
        .expect("manifest success should mark image as remotely resolvable");

        assert_eq!(
            outcome,
            DockerImageInspectOutcome::Available(DockerImageAvailability::RemoteResolvable)
        );
    }

    #[test]
    fn inspect_docker_image_reference_with_runner_short_circuits_after_local_success() {
        let mut calls = Vec::new();
        let outcome = inspect_docker_image_reference_with_runner(
            "docker",
            "itzg/minecraft-server:latest",
            |docker_path, args, timeout, action| {
                calls.push((
                    docker_path.to_string(),
                    args.iter()
                        .map(|value| value.to_string())
                        .collect::<Vec<_>>(),
                    timeout.as_secs(),
                    action.to_string(),
                ));

                Ok(successful_output("[{\"Id\":\"sha256:abc\"}]"))
            },
        )
        .expect("local inspect success should short-circuit before manifest probe");

        assert_eq!(
            outcome,
            DockerImageInspectOutcome::Available(DockerImageAvailability::LocalCached)
        );
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "docker");
        assert_eq!(calls[0].1, vec!["image", "inspect", "itzg/minecraft-server:latest"]);
        assert_eq!(calls[0].2, DOCKER_IMAGE_INSPECT_TIMEOUT_SECS);
        assert_eq!(calls[0].3, "docker image inspect");
    }

    #[test]
    fn inspect_docker_image_reference_with_runner_interprets_manifest_success_without_real_docker()
    {
        let mut call_index = 0;
        let outcome = inspect_docker_image_reference_with_runner(
            "docker",
            "itzg/minecraft-server:latest",
            |_docker_path, args, _timeout, action| {
                call_index += 1;
                match call_index {
                    1 => {
                        assert_eq!(action, "docker image inspect");
                        assert_eq!(args, ["image", "inspect", "itzg/minecraft-server:latest"]);
                        Ok(failed_output(
                            "Error response from daemon: No such image: itzg/minecraft-server:latest",
                        ))
                    }
                    2 => {
                        assert_eq!(action, "docker manifest inspect");
                        assert_eq!(args, ["manifest", "inspect", "itzg/minecraft-server:latest"]);
                        Ok(successful_output("{\"schemaVersion\":2}"))
                    }
                    _ => panic!("unexpected extra probe"),
                }
            },
        )
        .expect("manifest success should produce remote-resolvable outcome");

        assert_eq!(call_index, 2);
        assert_eq!(
            outcome,
            DockerImageInspectOutcome::Available(DockerImageAvailability::RemoteResolvable)
        );
    }

    #[test]
    fn inspect_docker_image_reference_with_runner_downgrades_runner_errors_to_soft_failure() {
        let mut call_index = 0;
        let outcome = inspect_docker_image_reference_with_runner(
            "docker",
            "itzg/minecraft-server:latest",
            |_docker_path, _args, _timeout, action| {
                call_index += 1;
                match call_index {
                    1 => Err(format!("{} 超时: 5s 内未返回；已按软失败处理", action)),
                    2 => Err(format!("{} 超时: 5s 内未返回；已按软失败处理", action)),
                    _ => panic!("unexpected extra probe"),
                }
            },
        )
        .expect("timeout-style runner errors should downgrade to soft failure");

        assert_eq!(call_index, 2);
        match outcome {
            DockerImageInspectOutcome::SoftFailure { failure_kind, message } => {
                assert_eq!(failure_kind, DockerCommandFailureKind::Timeout);
                assert!(message.contains("Docker 镜像探测超时"));
                assert!(message.contains("docker manifest inspect 超时"));
            }
            other => panic!("expected soft failure, got {:?}", other),
        }
    }

    #[test]
    fn inspect_docker_image_reference_with_runner_uses_stdout_only_manifest_output_for_not_found() {
        let err = inspect_docker_image_reference_with_runner(
            "docker",
            "itzg/minecraft-server:missing",
            |_docker_path, args, _timeout, _action| {
                if args[0] == "image" {
                    Ok(failed_output(
                        "Error response from daemon: No such image: itzg/minecraft-server:missing",
                    ))
                } else {
                    Ok(failed_output_with_stdout("manifest unknown: manifest unknown"))
                }
            },
        )
        .expect_err("stdout-only manifest failure should still map to image-not-found hard error");

        assert!(err.contains("镜像或标签不存在"));
        assert!(err.contains("manifest unknown"));
    }

    #[test]
    fn inspect_docker_image_reference_with_runner_falls_back_to_exit_code_only_soft_failure() {
        let outcome = inspect_docker_image_reference_with_runner(
            "docker",
            "itzg/minecraft-server:latest",
            |_docker_path, args, _timeout, _action| {
                if args[0] == "image" {
                    Ok(failed_output(
                        "Error response from daemon: No such image: itzg/minecraft-server:latest",
                    ))
                } else {
                    Ok(Output {
                        status: exit_status_from_raw(7),
                        stdout: Vec::new(),
                        stderr: Vec::new(),
                    })
                }
            },
        )
        .expect("exit-code-only manifest failure should become soft failure");

        match outcome {
            DockerImageInspectOutcome::SoftFailure { failure_kind, message } => {
                assert_eq!(failure_kind, DockerCommandFailureKind::Other);
                assert!(message.contains("退出码"));
                assert!(message.contains("image=itzg/minecraft-server:latest"));
            }
            other => panic!("expected soft failure, got {:?}", other),
        }
    }

    #[test]
    fn classify_manifest_inspect_outcome_returns_soft_failure_for_network_errors() {
        let outcome = classify_manifest_inspect_outcome(
            "itzg/minecraft-server:latest",
            &failed_output("dial tcp 1.2.3.4:443: connectex: timeout"),
        )
        .expect("network failures should downgrade to soft failure");

        match outcome {
            DockerImageInspectOutcome::SoftFailure { failure_kind, message } => {
                assert_eq!(failure_kind, DockerCommandFailureKind::Network);
                assert!(message.contains("镜像拉取网络不可达或超时"));
                assert!(message.contains("itzg/minecraft-server:latest"));
            }
            other => panic!("expected soft failure, got {:?}", other),
        }
    }

    #[test]
    fn classify_manifest_inspect_outcome_returns_hard_error_for_auth_failures() {
        let err = classify_manifest_inspect_outcome(
            "itzg/minecraft-server:latest",
            &failed_output("unauthorized: authentication required"),
        )
        .expect_err("auth failures should remain hard errors");

        assert!(err.contains("镜像仓库认证或访问被拒绝"));
        assert!(err.contains("authentication required"));
    }

    #[test]
    fn classify_manifest_inspect_outcome_returns_hard_error_for_image_not_found() {
        let err = classify_manifest_inspect_outcome(
            "itzg/minecraft-server:missing",
            &failed_output_with_stdout("manifest unknown: manifest unknown"),
        )
        .expect_err("missing images should remain hard errors");

        assert!(err.contains("镜像或标签不存在"));
        assert!(err.contains("manifest unknown"));
    }

    #[test]
    fn detects_missing_container_errors() {
        assert!(docker_error_indicates_missing_container(
            "Error response from daemon: No such container: sea-test"
        ));
        assert!(docker_error_indicates_missing_container("container sea-test not found"));
        assert!(docker_error_indicates_missing_container("Error: no such object: sea-test"));
        assert!(!docker_error_indicates_missing_container(
            "failed to resolve reference: dial tcp timeout"
        ));
    }

    #[test]
    fn split_docker_image_reference_tag_extracts_embedded_tag() {
        let (image, tag) = split_docker_image_reference_tag("itzg/minecraft-server:java21");

        assert_eq!(image, "itzg/minecraft-server");
        assert_eq!(tag, Some("java21"));
    }

    #[test]
    fn split_docker_image_reference_tag_preserves_registry_port_without_tag() {
        let (image, tag) =
            split_docker_image_reference_tag("registry.local:5000/itzg/minecraft-server");

        assert_eq!(image, "registry.local:5000/itzg/minecraft-server");
        assert_eq!(tag, None);
    }

    #[test]
    fn resolve_docker_image_and_tag_uses_embedded_tag_when_image_tag_is_missing() {
        let (image, tag) = resolve_docker_image_and_tag(
            Some("registry.local:5000/paper:test"),
            None,
            "itzg/minecraft-server",
            "latest",
        )
        .expect("image ref should resolve");

        assert_eq!(image, "registry.local:5000/paper");
        assert_eq!(tag, "test");
    }

    #[test]
    fn resolve_docker_image_and_tag_prefers_explicit_image_tag_over_embedded_tag() {
        let (image, tag) = resolve_docker_image_and_tag(
            Some("itzg/minecraft-server:java21"),
            Some("latest"),
            "itzg/minecraft-server",
            "latest",
        )
        .expect("image ref should resolve");

        assert_eq!(image, "itzg/minecraft-server");
        assert_eq!(tag, "latest");
    }

    #[test]
    fn resolve_docker_image_and_tag_rejects_digest_reference() {
        let err = resolve_docker_image_and_tag(
            Some("itzg/minecraft-server@sha256:deadbeef"),
            None,
            "itzg/minecraft-server",
            "latest",
        )
        .expect_err("digest refs should be rejected for now");

        assert!(err.contains("digest"));
    }

    #[test]
    fn format_docker_image_reference_formats_repo_and_tag() {
        assert_eq!(
            format_docker_image_reference("itzg/minecraft-server", "latest"),
            "itzg/minecraft-server:latest"
        );
    }
}

fn exit_status_from_raw(code: i32) -> std::process::ExitStatus {
    #[cfg(windows)]
    {
        std::os::windows::process::ExitStatusExt::from_raw(code as u32)
    }
    #[cfg(unix)]
    {
        std::os::unix::process::ExitStatusExt::from_raw(code)
    }
}
