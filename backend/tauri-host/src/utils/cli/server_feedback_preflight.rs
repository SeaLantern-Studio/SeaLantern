use crate::utils::cli::server_args::CliServerCommand;
use crate::utils::cli::server_setup::{RuntimePreflightError, RuntimePreflightStage};
use crate::utils::cli::server_shared::{resolve_cli_target_hint, CliServerRuntimeKind};
use crate::utils::docker_cli::{
    classify_docker_command_failure, format_docker_image_reference, resolve_docker_image_and_tag,
    DockerCommandFailureKind,
};

fn render_requested_docker_image_ref(command: &CliServerCommand) -> String {
    resolve_docker_image_and_tag(
        command.image.as_deref(),
        command.image_tag.as_deref(),
        "itzg/minecraft-server",
        "latest",
    )
    .map(|(image, tag)| format_docker_image_reference(&image, &tag))
    .unwrap_or_else(|_| {
        let image = command
            .image
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("itzg/minecraft-server");
        let tag = command
            .image_tag
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("latest");
        if image.contains(':') && command.image_tag.is_none() {
            image.to_string()
        } else {
            format_docker_image_reference(image, tag)
        }
    })
}

pub(super) fn render_runtime_preflight_failure_hint_lines(
    command: &CliServerCommand,
    runtime_kind: CliServerRuntimeKind,
    error: &str,
) -> Vec<String> {
    let error_lower = error.to_ascii_lowercase();
    let stage = match runtime_kind {
        CliServerRuntimeKind::Local => RuntimePreflightStage::LocalJava,
        CliServerRuntimeKind::Docker => {
            if error_lower.contains("sealantern_servers_container_root")
                || error_lower.contains("sealantern_servers_host_root")
                || error_lower.contains("宿主路径映射")
            {
                RuntimePreflightStage::DockerDataDir
            } else if error_lower.contains("--docker-backend")
                || error_lower.contains("docker-backend")
                || error_lower.contains("backend cli")
            {
                RuntimePreflightStage::DockerBackend
            } else if classify_docker_command_failure(error)
                == DockerCommandFailureKind::DaemonUnavailable
            {
                RuntimePreflightStage::DockerEnvironment
            } else {
                RuntimePreflightStage::DockerImage
            }
        }
    };

    render_runtime_preflight_failure_hint_lines_from_error(
        command,
        &RuntimePreflightError::new(runtime_kind, stage, error.to_string(), None),
    )
}

pub(super) fn render_runtime_preflight_failure_hint_lines_from_error(
    command: &CliServerCommand,
    error: &RuntimePreflightError,
) -> Vec<String> {
    let target_hint = resolve_cli_target_hint(command);
    let mut lines = vec![format!(
        "可继续执行: sealantern server help    # 复查 {} runtime 参数",
        error.runtime_kind.as_runtime_label()
    )];

    match error.runtime_kind {
        CliServerRuntimeKind::Local => {
            lines.push("本地建议: 可显式指定 --java <path>，或改用 --java %env:JAVA_HOME% / --java %env:Path%".to_string());
            if command.java_from_env_only {
                lines.push("本地建议: 当前使用 --J，仅允许从 JAVA_HOME/PATH 解析 Java；如环境未配置可改用 --java 显式指定".to_string());
            } else {
                lines.push(
                    "本地建议: 如环境变量和默认配置都未命中，CLI 会提示是否执行全局 Java 扫描"
                        .to_string(),
                );
            }
            if matches!(error.stage, RuntimePreflightStage::LocalJava)
                || error.message.to_ascii_lowercase().contains("java")
            {
                lines.push(format!(
                    "本地建议: 修复 Java 后可重试 sealantern server start {} 或重新执行 create flow",
                    target_hint
                ));
            }
        }
        CliServerRuntimeKind::Docker => {
            let error_lower = error.message.to_ascii_lowercase();
            let failure_kind = classify_docker_command_failure(&error.message);
            lines.push(format!(
                "Docker 建议: 环境恢复后可重试 sealantern server {} --runtime docker ...",
                target_hint
            ));

            match error.stage {
                RuntimePreflightStage::DockerEnvironment => {
                    lines.push("Docker 建议: 先执行 sealantern docker doctor".to_string());
                    lines.push(
                        "Docker 建议: 请确认 Docker Desktop / Engine 已启动，且当前用户可访问 daemon"
                            .to_string(),
                    );
                }
                RuntimePreflightStage::DockerBackend => {
                    lines.push(
                        "Docker 建议: 当前 CLI 主路径只接入了 docker CLI backend；请使用 --docker-backend cli 或直接省略该参数"
                            .to_string(),
                    );
                    lines.push(
                        "Docker 建议: 如需后续扩展 Engine API，可保留该记录格式，但当前版本不会实际运行 engine_api"
                            .to_string(),
                    );
                }
                RuntimePreflightStage::DockerDataDir => {
                    lines.push(
                        "Docker 建议: 当前更像容器路径映射缺失，请配置 SEALANTERN_SERVERS_CONTAINER_ROOT / SEALANTERN_SERVERS_HOST_ROOT，或显式传入 --data-dir"
                            .to_string(),
                    );
                }
                RuntimePreflightStage::DockerImage => {
                    let image_ref = render_requested_docker_image_ref(command);

                    if error_lower.contains("镜像名看起来不兼容")
                        || error_lower.contains("minecraft server 容器")
                    {
                        lines.push(
                            "Docker 建议: 当前 docker_itzg runtime 只接受 Minecraft server 语义镜像；请改用 itzg/minecraft-server，或保持私有镜像最终名仍为 */minecraft-server"
                                .to_string(),
                        );
                    }

                    if error_lower.contains("docker_stdio")
                        || error_lower.contains("mc-send-to-console")
                    {
                        lines.push(
                            "Docker 建议: 当前镜像不适合 --command-mode docker_stdio；如无强依赖，优先切回 --command-mode rcon"
                                .to_string(),
                        );
                        lines.push(
                            "Docker 建议: 若坚持使用 docker_stdio，请确认镜像内提供 mc-send-to-console，并遵循 itzg 的 CREATE_CONSOLE_IN_PIPE 语义"
                                .to_string(),
                        );
                    }

                    lines.push(format!(
                        "Docker 建议: 若本地未缓存该镜像、且后续需要联网拉取，可在网络恢复后执行 sealantern docker pull {}",
                        image_ref
                    ));
                    lines.push(
                        "Docker 建议: 若你本地已缓存该镜像，或正在使用私有/离线镜像源，可直接继续 create/start/compose 流程"
                            .to_string(),
                    );

                    match failure_kind {
                        DockerCommandFailureKind::DaemonUnavailable => {
                            lines.push("Docker 建议: 先执行 sealantern docker doctor".to_string());
                            lines.push(
                                "Docker 建议: 请确认 Docker Desktop / Engine 已启动，且当前用户可访问 daemon"
                                    .to_string(),
                            );
                        }
                        DockerCommandFailureKind::Network => {
                            lines.push(
                                "Docker 建议: 当前更像网络或代理问题，请检查镜像仓库连通性、代理配置和 TLS/证书环境"
                                    .to_string(),
                            );
                        }
                        DockerCommandFailureKind::Timeout => {
                            lines.push(
                                "Docker 建议: 当前镜像探测命令超时，CLI 会按软失败继续；若宿主机还没有该镜像，建议待网络恢复后再执行 docker pull 预热"
                                    .to_string(),
                            );
                            lines.push(
                                "Docker 建议: 如网络较慢或 registry 响应不稳定，请稍后重试，或先用 sealantern docker doctor 检查宿主环境"
                                    .to_string(),
                            );
                        }
                        DockerCommandFailureKind::Auth => {
                            lines.push(
                                "Docker 建议: 当前更像 registry 认证/权限问题，请先登录对应仓库并确认镜像可见性"
                                    .to_string(),
                            );
                        }
                        DockerCommandFailureKind::ImageNotFound => {
                            lines.push(
                                "Docker 建议: 当前更像镜像名或标签错误，请检查 --image / --image-tag 是否存在"
                                    .to_string(),
                            );
                        }
                        DockerCommandFailureKind::Other => {
                            if error_lower.contains("manifest") || error_lower.contains("pull") {
                                lines.push(format!(
                                    "Docker 建议: 如需联网验证镜像可用性，可执行 sealantern docker pull {}",
                                    image_ref
                                ));
                            }
                        }
                    }
                }
                RuntimePreflightStage::LocalJava => {}
            }
        }
    }

    dedupe_lines(lines)
}

fn dedupe_lines(lines: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut deduped = Vec::new();

    for line in lines {
        let normalized = line.trim().to_string();
        if normalized.is_empty() {
            continue;
        }
        if seen.insert(normalized.clone()) {
            deduped.push(normalized);
        }
    }

    deduped
}

#[cfg(test)]
mod tests {
    use super::{
        render_runtime_preflight_failure_hint_lines,
        render_runtime_preflight_failure_hint_lines_from_error,
    };
    use crate::utils::cli::server_args::CliServerCommand;
    use crate::utils::cli::server_setup::{RuntimePreflightError, RuntimePreflightStage};
    use crate::utils::cli::server_shared::CliServerRuntimeKind;

    #[test]
    fn render_runtime_preflight_failure_hint_lines_include_docker_recovery_actions() {
        let command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            image: Some("itzg/minecraft-server".to_string()),
            image_tag: Some("latest".to_string()),
            ..Default::default()
        };

        let lines = render_runtime_preflight_failure_hint_lines(
            &command,
            CliServerRuntimeKind::Docker,
            "docker manifest inspect 失败: manifest unknown",
        )
        .join("\n");

        assert!(lines.contains("sealantern docker pull itzg/minecraft-server:latest"));
        assert!(lines.contains("--runtime docker"));
    }

    #[test]
    fn render_runtime_preflight_failure_hint_lines_include_local_java_guidance() {
        let command = CliServerCommand {
            positional_name: Some("fabric-1.20.1".to_string()),
            java_from_env_only: true,
            ..Default::default()
        };

        let lines = render_runtime_preflight_failure_hint_lines(
            &command,
            CliServerRuntimeKind::Local,
            "--J 模式下未在 JAVA_HOME 或 PATH 中找到 Java",
        )
        .join("\n");

        assert!(lines.contains("--java <path>"));
        assert!(lines.contains("--J，仅允许从 JAVA_HOME/PATH 解析 Java"));
        assert!(lines.contains("sealantern server start fabric-1.20.1"));
    }

    #[test]
    fn render_runtime_preflight_failure_hint_lines_distinguish_docker_network_failures() {
        let command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            image: Some("itzg/minecraft-server".to_string()),
            image_tag: Some("latest".to_string()),
            ..Default::default()
        };

        let lines = render_runtime_preflight_failure_hint_lines(
            &command,
            CliServerRuntimeKind::Docker,
            "docker pull 失败: dial tcp 1.2.3.4:443: connectex: timeout",
        )
        .join("\n");

        assert!(lines.contains("网络或代理问题"));
        assert!(lines.contains("docker pull itzg/minecraft-server:latest"));
    }

    #[test]
    fn render_runtime_preflight_failure_hint_lines_distinguish_docker_auth_failures() {
        let command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            image: Some("private/repo".to_string()),
            image_tag: Some("latest".to_string()),
            ..Default::default()
        };

        let lines = render_runtime_preflight_failure_hint_lines(
            &command,
            CliServerRuntimeKind::Docker,
            "docker manifest inspect 失败: unauthorized: authentication required",
        )
        .join("\n");

        assert!(lines.contains("registry 认证/权限问题"));
        assert!(lines.contains("private/repo:latest"));
    }

    #[test]
    fn render_runtime_preflight_failure_hint_lines_distinguish_path_mapping_failures() {
        let command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            ..Default::default()
        };

        let lines = render_runtime_preflight_failure_hint_lines(
            &command,
            CliServerRuntimeKind::Docker,
            "当前 Sea Lantern 运行在容器可见路径下，且未配置 SEALANTERN_SERVERS_CONTAINER_ROOT / SEALANTERN_SERVERS_HOST_ROOT；请显式传入 --data-dir，或配置宿主路径映射",
        )
        .join("\n");

        assert!(lines.contains("SEALANTERN_SERVERS_CONTAINER_ROOT / SEALANTERN_SERVERS_HOST_ROOT"));
        assert!(lines.contains("--data-dir"));
    }

    #[test]
    fn render_runtime_preflight_failure_hint_lines_from_error_uses_stage_specific_docker_environment_guidance(
    ) {
        let command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            image: Some("itzg/minecraft-server".to_string()),
            image_tag: Some("latest".to_string()),
            ..Default::default()
        };

        let lines = render_runtime_preflight_failure_hint_lines_from_error(
            &command,
            &RuntimePreflightError::new(
                CliServerRuntimeKind::Docker,
                RuntimePreflightStage::DockerEnvironment,
                "执行 docker info 失败: daemon unavailable".to_string(),
                None,
            ),
        )
        .join("\n");

        assert!(lines.contains("sealantern docker doctor"));
        assert!(lines.contains("Docker Desktop / Engine 已启动"));
        assert!(!lines.contains("docker pull itzg/minecraft-server:latest"));
    }

    #[test]
    fn render_runtime_preflight_failure_hint_lines_from_error_uses_stage_specific_docker_backend_guidance(
    ) {
        let command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            runtime: Some("docker".to_string()),
            docker_backend: Some("engine_api".to_string()),
            ..Default::default()
        };

        let lines = render_runtime_preflight_failure_hint_lines_from_error(
            &command,
            &RuntimePreflightError::new(
                CliServerRuntimeKind::Docker,
                RuntimePreflightStage::DockerBackend,
                "当前仅支持 --docker-backend cli；engine_api 尚未接入真实运行链路，请移除该参数或改为 --docker-backend cli"
                    .to_string(),
                Some("requested_backend=engine_api".to_string()),
            ),
        )
        .join("\n");

        assert!(lines.contains("只接入了 docker CLI backend"));
        assert!(lines.contains("--docker-backend cli"));
        assert!(lines.contains("不会实际运行 engine_api"));
    }

    #[test]
    fn render_runtime_preflight_failure_hint_lines_from_error_uses_stage_specific_docker_image_guidance(
    ) {
        let command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            image: Some("private/repo".to_string()),
            image_tag: Some("latest".to_string()),
            ..Default::default()
        };

        let lines = render_runtime_preflight_failure_hint_lines_from_error(
            &command,
            &RuntimePreflightError::new(
                CliServerRuntimeKind::Docker,
                RuntimePreflightStage::DockerImage,
                "docker manifest inspect 失败: unauthorized: authentication required".to_string(),
                Some("image=private/repo:latest".to_string()),
            ),
        )
        .join("\n");

        assert!(lines.contains("docker pull private/repo:latest"));
        assert!(lines.contains("registry 认证/权限问题"));
    }

    #[test]
    fn render_runtime_preflight_failure_hint_lines_supports_embedded_image_tag_reference() {
        let command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            image: Some("registry.local:5000/paper:java21".to_string()),
            ..Default::default()
        };

        let lines = render_runtime_preflight_failure_hint_lines(
            &command,
            CliServerRuntimeKind::Docker,
            "docker manifest inspect 失败: dial tcp 1.2.3.4:443: connectex: timeout",
        )
        .join("\n");

        assert!(lines.contains("sealantern docker pull registry.local:5000/paper:java21"));
        assert!(lines.contains("私有/离线镜像源"));
    }

    #[test]
    fn render_runtime_preflight_failure_hint_lines_explains_incompatible_docker_runtime_image() {
        let command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            image: Some("naloveyuki/liteyukibot-web".to_string()),
            image_tag: Some("latest".to_string()),
            ..Default::default()
        };

        let lines = render_runtime_preflight_failure_hint_lines(
            &command,
            CliServerRuntimeKind::Docker,
            "当前 docker runtime 目标是 Minecraft server 容器，但镜像名看起来不兼容: naloveyuki/liteyukibot-web。请使用 itzg/minecraft-server 或你自己的 */minecraft-server 镜像名；如果这是私有镜像/镜像代理，也请保持最终镜像名仍为 minecraft-server",
        )
        .join("\n");

        assert!(lines.contains("只接受 Minecraft server 语义镜像"));
        assert!(lines.contains("itzg/minecraft-server"));
    }

    #[test]
    fn render_runtime_preflight_failure_hint_lines_explains_docker_stdio_incompatible_image() {
        let command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            image: Some("local/minecraft-server".to_string()),
            image_tag: Some("smoke-rcon".to_string()),
            command_mode: Some("docker_stdio".to_string()),
            ..Default::default()
        };

        let lines = render_runtime_preflight_failure_hint_lines(
            &command,
            CliServerRuntimeKind::Docker,
            "当前镜像不支持 --command-mode docker_stdio: image=local/minecraft-server:smoke-rcon 未检测到 mc-send-to-console。请改用 --command-mode rcon，或改用兼容 itzg 语义且内置 mc-send-to-console 的镜像。",
        )
        .join("\n");

        assert!(lines.contains("--command-mode docker_stdio"));
        assert!(lines.contains("优先切回 --command-mode rcon"));
        assert!(lines.contains("mc-send-to-console"));
    }
}
