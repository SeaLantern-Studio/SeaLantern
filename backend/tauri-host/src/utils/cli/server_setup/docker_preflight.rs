use crate::services::global::i18n_service;
use crate::utils::cli::server_shared::{trace_cli_action, trace_cli_error};
#[cfg(test)]
use crate::utils::docker_cli::interpret_docker_image_inspect_outputs_for_tests;
use crate::utils::docker_cli::{
    docker_executable_path, inspect_docker_image_reference_with_soft_failures,
    DockerImageAvailability, DockerImageInspectOutcome,
};
use crate::utils::logger;
use sea_lantern_docker_core::{
    format_docker_image_reference, validate_docker_itzg_image_compatibility, DockerCommandMode,
};
use std::collections::HashMap;
use std::process::Output;

fn cli_docker_t(key: &str) -> String {
    i18n_service().t(key)
}

fn cli_docker_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    i18n_service().t_with_options(key, &m)
}

fn cli_docker_t2(key: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    i18n_service().t_with_options(key, &m)
}

#[cfg(test)]
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

pub(crate) fn ensure_docker_environment() -> Result<(), String> {
    let docker_path = docker_executable_path()?;
    let output = std::process::Command::new(docker_path)
        .arg("info")
        .output()
        .map_err(|e| cli_docker_t1("cli.server_setup.docker.info_failed", e.to_string()))?;

    ensure_docker_environment_with_output(&output)
}

fn ensure_docker_environment_with_output(output: &Output) -> Result<(), String> {
    if output.status.success() {
        trace_cli_action("docker_environment_ready", "docker info succeeded");
        return Ok(());
    }

    let error = render_docker_environment_error(output);
    trace_cli_error("docker_environment_unavailable", "docker info", &error);
    Err(error)
}

fn render_docker_environment_error(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        cli_docker_t("cli.server_setup.docker.environment_unavailable_default")
    } else {
        cli_docker_t1("cli.server_setup.docker.environment_unavailable", stderr)
    }
}

pub(crate) fn preflight_docker_image_reference(image: &str, image_tag: &str) -> Result<(), String> {
    preflight_docker_image_reference_with(
        image,
        image_tag,
        inspect_docker_image_reference_with_soft_failures,
    )
}

fn preflight_docker_image_reference_with<F>(
    image: &str,
    image_tag: &str,
    inspect_image: F,
) -> Result<(), String>
where
    F: FnOnce(&str) -> Result<DockerImageInspectOutcome, String>,
{
    validate_docker_itzg_image_compatibility(image)?;

    let image_ref = format_docker_image_reference(image, image_tag);
    let outcome = inspect_image(&image_ref)?;

    handle_docker_image_preflight_outcome(&image_ref, outcome)
}

fn handle_docker_image_preflight_outcome(
    image_ref: &str,
    outcome: DockerImageInspectOutcome,
) -> Result<(), String> {
    match outcome {
        DockerImageInspectOutcome::Available(DockerImageAvailability::LocalCached) => {
            trace_cli_action(
                "docker_image_preflight",
                &format!("image_ref={} availability=local_cached", image_ref),
            );
            Ok(())
        }
        DockerImageInspectOutcome::Available(DockerImageAvailability::RemoteResolvable) => {
            trace_cli_action(
                "docker_image_preflight",
                &format!("image_ref={} availability=remote_resolvable", image_ref),
            );
            Ok(())
        }
        DockerImageInspectOutcome::SoftFailure { failure_kind, message } => {
            trace_cli_action(
                "docker_image_preflight_soft_failure",
                &format!("image_ref={} failure_kind={:?}", image_ref, failure_kind),
            );
            logger::log_warn_ctx(
                "cli.server_setup.docker_preflight",
                "handle_docker_image_preflight_outcome",
                &format!(
                    "soft failure downgraded; continuing server creation: image_ref={} failure_kind={:?} detail={}",
                    image_ref, failure_kind, message
                ),
            );
            println!(
                "{}",
                cli_docker_t1("cli.server_setup.docker.image_preflight_soft_failure", image_ref),
            );
            Ok(())
        }
    }
}

pub(crate) fn preflight_docker_command_mode_support(
    image: &str,
    image_tag: &str,
    command_mode: &DockerCommandMode,
) -> Result<(), String> {
    preflight_docker_command_mode_support_with(
        image,
        image_tag,
        command_mode,
        inspect_docker_image_reference_with_soft_failures,
        ensure_docker_stdio_image_support,
    )
}

fn preflight_docker_command_mode_support_with<FI, FP>(
    image: &str,
    image_tag: &str,
    command_mode: &DockerCommandMode,
    inspect_image: FI,
    ensure_stdio_support: FP,
) -> Result<(), String>
where
    FI: FnOnce(&str) -> Result<DockerImageInspectOutcome, String>,
    FP: FnOnce(&str) -> Result<(), String>,
{
    if *command_mode != DockerCommandMode::DockerStdio {
        return Ok(());
    }

    let image_ref = format_docker_image_reference(image, image_tag);
    let outcome = inspect_image(&image_ref)?;

    handle_docker_stdio_preflight_outcome(&image_ref, outcome, ensure_stdio_support)
}

fn handle_docker_stdio_preflight_outcome<F>(
    image_ref: &str,
    outcome: DockerImageInspectOutcome,
    ensure_stdio_support: F,
) -> Result<(), String>
where
    F: FnOnce(&str) -> Result<(), String>,
{
    match outcome {
        DockerImageInspectOutcome::Available(DockerImageAvailability::LocalCached) => {
            ensure_stdio_support(image_ref)
        }
        DockerImageInspectOutcome::Available(DockerImageAvailability::RemoteResolvable) => {
            trace_cli_action(
                "docker_stdio_preflight_remote_only",
                &format!("image_ref={}", image_ref),
            );
            logger::log_warn_ctx(
                "cli.server_setup.docker_preflight",
                "handle_docker_stdio_preflight_outcome",
                &format!(
                    "skipping image internal probe: image_ref={} availability=remote_resolvable",
                    image_ref
                ),
            );
            Ok(())
        }
        DockerImageInspectOutcome::SoftFailure { failure_kind, message } => {
            trace_cli_action(
                "docker_stdio_preflight_soft_failure",
                &format!("image_ref={} failure_kind={:?}", image_ref, failure_kind),
            );
            logger::log_warn_ctx(
                "cli.server_setup.docker_preflight",
                "handle_docker_stdio_preflight_outcome",
                &format!(
                    "image cache status unresolved; skipping internal command probe: image_ref={} failure_kind={:?} detail={}",
                    image_ref, failure_kind, message
                ),
            );
            Ok(())
        }
    }
}

fn ensure_docker_stdio_image_support(image_ref: &str) -> Result<(), String> {
    let docker_path = docker_executable_path()?;
    let output = std::process::Command::new(docker_path)
        .arg("run")
        .arg("--rm")
        .arg("--entrypoint")
        .arg("sh")
        .arg(image_ref)
        .arg("-lc")
        .arg("command -v mc-send-to-console >/dev/null 2>&1")
        .output()
        .map_err(|err| {
            cli_docker_t1("cli.server_setup.docker.stdio_probe_failed", err.to_string())
        })?;

    ensure_docker_stdio_image_support_with_output(image_ref, &output)
}

fn ensure_docker_stdio_image_support_with_output(
    image_ref: &str,
    output: &Output,
) -> Result<(), String> {
    if output.status.success() {
        trace_cli_action("docker_stdio_preflight_supported", &format!("image_ref={}", image_ref));
        return Ok(());
    }

    Err(render_docker_stdio_probe_error(image_ref, output))
}

fn render_docker_stdio_probe_error(image_ref: &str, output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let raw = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        cli_docker_t1(
            "cli.server_setup.docker.command_exit_code",
            format!("{:?}", output.status.code()),
        )
    };

    cli_docker_t2("cli.server_setup.docker.stdio_image_unsupported", image_ref, raw)
}

#[cfg(test)]
pub(crate) fn preflight_docker_image_reference_from_outputs_for_tests(
    image: &str,
    image_tag: &str,
    local_output: &Output,
    manifest_output: &Output,
) -> Result<(), String> {
    preflight_docker_image_reference_with(image, image_tag, |image_ref| {
        interpret_docker_image_inspect_outputs_for_tests(image_ref, local_output, manifest_output)
    })
}

#[cfg(test)]
pub(crate) fn preflight_docker_command_mode_support_from_outputs_for_tests<F>(
    image: &str,
    image_tag: &str,
    command_mode: &DockerCommandMode,
    local_output: &Output,
    manifest_output: &Output,
    ensure_stdio_support: F,
) -> Result<(), String>
where
    F: FnOnce(&str) -> Result<(), String>,
{
    preflight_docker_command_mode_support_with(
        image,
        image_tag,
        command_mode,
        |image_ref| {
            interpret_docker_image_inspect_outputs_for_tests(
                image_ref,
                local_output,
                manifest_output,
            )
        },
        ensure_stdio_support,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_docker_environment_with_output, ensure_docker_stdio_image_support_with_output,
        handle_docker_image_preflight_outcome, handle_docker_stdio_preflight_outcome,
        preflight_docker_command_mode_support, preflight_docker_command_mode_support_with,
        preflight_docker_image_reference, preflight_docker_image_reference_with,
        render_docker_environment_error, DockerCommandMode,
    };
    use crate::utils::docker_cli::{
        interpret_docker_image_inspect_outputs_for_tests, DockerCommandFailureKind,
        DockerImageAvailability, DockerImageInspectOutcome,
    };
    use std::cell::{Cell, RefCell};
    use std::process::Output;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn output_with_status(code: i32, stdout: &str, stderr: &str) -> Output {
        Output {
            status: super::exit_status_from_raw(code),
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    fn local_success_output() -> Output {
        output_with_status(0, "[{\"Id\":\"sha256:abc\"}]", "")
    }

    fn manifest_success_output() -> Output {
        output_with_status(0, "{\"schemaVersion\":2}", "")
    }

    fn failed_output(stderr: &str) -> Output {
        output_with_status(1, "", stderr)
    }

    fn failed_output_with_stdout(stdout: &str) -> Output {
        output_with_status(1, stdout, "")
    }

    fn run_facade_visible_preflight_path_from_outputs<F>(
        image: &str,
        image_tag: &str,
        command_mode: DockerCommandMode,
        local_output: &Output,
        manifest_output: &Output,
        ensure_stdio_support: F,
    ) -> Result<(), String>
    where
        F: FnOnce(&str) -> Result<(), String>,
    {
        super::preflight_docker_image_reference_from_outputs_for_tests(
            image,
            image_tag,
            local_output,
            manifest_output,
        )?;

        super::preflight_docker_command_mode_support_from_outputs_for_tests(
            image,
            image_tag,
            &command_mode,
            local_output,
            manifest_output,
            ensure_stdio_support,
        )
    }

    #[test]
    fn render_docker_environment_error_prefers_stderr_detail() {
        let message = render_docker_environment_error(&output_with_status(
            1,
            "",
            "Cannot connect to the Docker daemon",
        ));

        assert!(message.contains("docker 环境不可用"));
        assert!(message.contains("Cannot connect to the Docker daemon"));
    }

    #[test]
    fn ensure_docker_environment_with_output_uses_default_message_when_stderr_is_empty() {
        let err = ensure_docker_environment_with_output(&output_with_status(1, "", ""))
            .expect_err("failing docker info should surface a user-facing error");

        assert!(err.contains("Docker Desktop"));
        assert!(err.contains("Docker Engine"));
    }

    #[test]
    fn preflight_docker_image_reference_rejects_incompatible_image_before_probe() {
        let err = preflight_docker_image_reference("example/custom-web", "latest")
            .expect_err("non minecraft image should fail before inspect");

        assert!(err.contains("example/custom-web"));
        assert!(err.contains("minecraft-server"));
    }

    #[test]
    fn handle_docker_image_preflight_outcome_allows_soft_failure_path() {
        let outcome = DockerImageInspectOutcome::SoftFailure {
            failure_kind: DockerCommandFailureKind::Network,
            message: "dial tcp timeout".to_string(),
        };

        handle_docker_image_preflight_outcome("itzg/minecraft-server:latest", outcome)
            .expect("soft failure should not block docker record creation");
    }

    #[test]
    fn preflight_docker_image_reference_with_consumes_local_cached_outcome() {
        let seen_image_ref = RefCell::new(None::<String>);

        preflight_docker_image_reference_with("itzg/minecraft-server", "latest", |image_ref| {
            seen_image_ref.replace(Some(image_ref.to_string()));
            Ok(DockerImageInspectOutcome::Available(DockerImageAvailability::LocalCached))
        })
        .expect("local cached images should pass preflight");

        assert_eq!(seen_image_ref.into_inner().as_deref(), Some("itzg/minecraft-server:latest"));
    }

    #[test]
    fn preflight_docker_image_reference_with_consumes_remote_resolvable_outcome() {
        let seen_image_ref = RefCell::new(None::<String>);

        preflight_docker_image_reference_with("itzg/minecraft-server", "java21", |image_ref| {
            seen_image_ref.replace(Some(image_ref.to_string()));
            Ok(DockerImageInspectOutcome::Available(DockerImageAvailability::RemoteResolvable))
        })
        .expect("remote resolvable images should pass preflight");

        assert_eq!(seen_image_ref.into_inner().as_deref(), Some("itzg/minecraft-server:java21"));
    }

    #[test]
    fn preflight_docker_image_reference_with_allows_soft_failure_outcome() {
        preflight_docker_image_reference_with("itzg/minecraft-server", "latest", |_image_ref| {
            Ok(DockerImageInspectOutcome::SoftFailure {
                failure_kind: DockerCommandFailureKind::Timeout,
                message: "docker manifest inspect timeout".to_string(),
            })
        })
        .expect("soft failure should still allow preflight to continue");
    }

    #[test]
    fn preflight_docker_image_reference_with_propagates_user_visible_hard_failure() {
        let err = preflight_docker_image_reference_with(
            "itzg/minecraft-server",
            "missing",
            |_image_ref| {
                Err(
                    "docker manifest inspect 失败: image=itzg/minecraft-server:missing 镜像或标签不存在"
                        .to_string(),
                )
            },
        )
        .expect_err("hard inspect failures should remain user-visible errors");

        assert!(err.contains("镜像或标签不存在"));
        assert!(err.contains("itzg/minecraft-server:missing"));
    }

    #[test]
    fn preflight_docker_image_reference_with_preserves_local_success_from_docker_cli_interpretation(
    ) {
        preflight_docker_image_reference_with("itzg/minecraft-server", "latest", |image_ref| {
            interpret_docker_image_inspect_outputs_for_tests(
                image_ref,
                &local_success_output(),
                &failed_output("manifest unknown: manifest unknown"),
            )
        })
        .expect("local inspect success should stay non-blocking after preflight consumption");
    }

    #[test]
    fn preflight_docker_image_reference_with_preserves_remote_resolvable_from_docker_cli_interpretation(
    ) {
        preflight_docker_image_reference_with("itzg/minecraft-server", "latest", |image_ref| {
            interpret_docker_image_inspect_outputs_for_tests(
                image_ref,
                &failed_output(
                    "Error response from daemon: No such image: itzg/minecraft-server:latest",
                ),
                &manifest_success_output(),
            )
        })
        .expect("manifest success should stay non-blocking after preflight consumption");
    }

    #[test]
    fn preflight_docker_image_reference_with_preserves_soft_failure_from_docker_cli_interpretation()
    {
        preflight_docker_image_reference_with("itzg/minecraft-server", "latest", |image_ref| {
            interpret_docker_image_inspect_outputs_for_tests(
                image_ref,
                &failed_output(
                    "Error response from daemon: No such image: itzg/minecraft-server:latest",
                ),
                &failed_output("dial tcp 1.2.3.4:443: connectex: timeout"),
            )
        })
        .expect("network soft failure should stay non-blocking after preflight consumption");
    }

    #[test]
    fn preflight_docker_image_reference_with_preserves_hard_failure_from_docker_cli_interpretation()
    {
        let err = preflight_docker_image_reference_with(
            "itzg/minecraft-server",
            "missing",
            |image_ref| {
                interpret_docker_image_inspect_outputs_for_tests(
                    image_ref,
                    &failed_output(
                        "Error response from daemon: No such image: itzg/minecraft-server:missing",
                    ),
                    &failed_output_with_stdout("manifest unknown: manifest unknown"),
                )
            },
        )
        .expect_err(
            "image-not-found hard failure should remain user-visible after preflight consumption",
        );

        assert!(err.contains("镜像或标签不存在"));
        assert!(err.contains("manifest unknown"));
    }

    #[test]
    fn preflight_docker_command_mode_support_skips_probe_for_non_stdio_mode() {
        preflight_docker_command_mode_support(
            "example/non-minecraft-image",
            "latest",
            &DockerCommandMode::Rcon,
        )
        .expect("non-stdio mode should return before any image probe");
    }

    #[test]
    fn preflight_docker_command_mode_support_with_consumes_local_cached_and_runs_probe() {
        let seen_image_ref = RefCell::new(None::<String>);
        let probe_calls = Cell::new(0);

        preflight_docker_command_mode_support_with(
            "itzg/minecraft-server",
            "latest",
            &DockerCommandMode::DockerStdio,
            |image_ref| {
                seen_image_ref.replace(Some(image_ref.to_string()));
                Ok(DockerImageInspectOutcome::Available(DockerImageAvailability::LocalCached))
            },
            |image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                assert_eq!(image_ref, "itzg/minecraft-server:latest");
                Ok(())
            },
        )
        .expect("local cached docker_stdio image should run probe and pass");

        assert_eq!(seen_image_ref.into_inner().as_deref(), Some("itzg/minecraft-server:latest"));
        assert_eq!(probe_calls.get(), 1);
    }

    #[test]
    fn preflight_docker_command_mode_support_with_skips_probe_for_remote_resolvable_outcome() {
        let probe_calls = Cell::new(0);

        preflight_docker_command_mode_support_with(
            "itzg/minecraft-server",
            "latest",
            &DockerCommandMode::DockerStdio,
            |_image_ref| {
                Ok(DockerImageInspectOutcome::Available(DockerImageAvailability::RemoteResolvable))
            },
            |_image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                Err("probe should have been skipped".to_string())
            },
        )
        .expect("remote resolvable images should skip stdio probe");

        assert_eq!(probe_calls.get(), 0);
    }

    #[test]
    fn preflight_docker_command_mode_support_with_skips_probe_for_soft_failure_outcome() {
        let probe_calls = Cell::new(0);

        preflight_docker_command_mode_support_with(
            "itzg/minecraft-server",
            "latest",
            &DockerCommandMode::DockerStdio,
            |_image_ref| {
                Ok(DockerImageInspectOutcome::SoftFailure {
                    failure_kind: DockerCommandFailureKind::Network,
                    message: "dial tcp timeout".to_string(),
                })
            },
            |_image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                Err("probe should have been skipped".to_string())
            },
        )
        .expect("soft failures should skip stdio probe and continue");

        assert_eq!(probe_calls.get(), 0);
    }

    #[test]
    fn preflight_docker_command_mode_support_with_propagates_hard_failure_before_probe() {
        let probe_calls = Cell::new(0);
        let err = preflight_docker_command_mode_support_with(
            "itzg/minecraft-server",
            "missing",
            &DockerCommandMode::DockerStdio,
            |_image_ref| {
                Err(
                    "docker manifest inspect 失败: image=itzg/minecraft-server:missing 镜像或标签不存在"
                        .to_string(),
                )
            },
            |_image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                Ok(())
            },
        )
        .expect_err("hard inspect failure should stop docker_stdio preflight");

        assert!(err.contains("镜像或标签不存在"));
        assert_eq!(probe_calls.get(), 0);
    }

    #[test]
    fn preflight_docker_command_mode_support_with_propagates_stdio_probe_error_for_local_cached() {
        let err = preflight_docker_command_mode_support_with(
            "itzg/minecraft-server",
            "java21",
            &DockerCommandMode::DockerStdio,
            |_image_ref| {
                Ok(DockerImageInspectOutcome::Available(
                    DockerImageAvailability::LocalCached,
                ))
            },
            |image_ref| {
                Err(format!(
                    "当前镜像不支持 --command-mode docker_stdio: image={} 未检测到 mc-send-to-console",
                    image_ref
                ))
            },
        )
        .expect_err("local cached stdio probe failures should remain user-visible");

        assert!(err.contains("docker_stdio"));
        assert!(err.contains("mc-send-to-console"));
        assert!(err.contains("itzg/minecraft-server:java21"));
    }

    #[test]
    fn preflight_docker_command_mode_support_with_preserves_local_success_probe_requirement_from_docker_cli_interpretation(
    ) {
        let probe_calls = Cell::new(0);

        preflight_docker_command_mode_support_with(
            "itzg/minecraft-server",
            "latest",
            &DockerCommandMode::DockerStdio,
            |image_ref| {
                interpret_docker_image_inspect_outputs_for_tests(
                    image_ref,
                    &local_success_output(),
                    &failed_output("manifest unknown: manifest unknown"),
                )
            },
            |_image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                Ok(())
            },
        )
        .expect("local success should still require stdio probe after preflight consumption");

        assert_eq!(probe_calls.get(), 1);
    }

    #[test]
    fn preflight_docker_command_mode_support_with_preserves_remote_resolvable_skip_from_docker_cli_interpretation(
    ) {
        let probe_calls = Cell::new(0);

        preflight_docker_command_mode_support_with(
            "itzg/minecraft-server",
            "latest",
            &DockerCommandMode::DockerStdio,
            |image_ref| {
                interpret_docker_image_inspect_outputs_for_tests(
                    image_ref,
                    &failed_output(
                        "Error response from daemon: No such image: itzg/minecraft-server:latest",
                    ),
                    &manifest_success_output(),
                )
            },
            |_image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                Err("probe should have been skipped".to_string())
            },
        )
        .expect("remote resolvable outcome should still skip stdio probe");

        assert_eq!(probe_calls.get(), 0);
    }

    #[test]
    fn preflight_docker_command_mode_support_with_preserves_soft_failure_skip_from_docker_cli_interpretation(
    ) {
        let probe_calls = Cell::new(0);

        preflight_docker_command_mode_support_with(
            "itzg/minecraft-server",
            "latest",
            &DockerCommandMode::DockerStdio,
            |image_ref| {
                interpret_docker_image_inspect_outputs_for_tests(
                    image_ref,
                    &failed_output(
                        "Error response from daemon: No such image: itzg/minecraft-server:latest",
                    ),
                    &failed_output("dial tcp 1.2.3.4:443: connectex: timeout"),
                )
            },
            |_image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                Err("probe should have been skipped".to_string())
            },
        )
        .expect("soft failure outcome should still skip stdio probe");

        assert_eq!(probe_calls.get(), 0);
    }

    #[test]
    fn preflight_docker_command_mode_support_with_preserves_hard_failure_from_docker_cli_interpretation(
    ) {
        let probe_calls = Cell::new(0);
        let err = preflight_docker_command_mode_support_with(
            "itzg/minecraft-server",
            "missing",
            &DockerCommandMode::DockerStdio,
            |image_ref| {
                interpret_docker_image_inspect_outputs_for_tests(
                    image_ref,
                    &failed_output(
                        "Error response from daemon: No such image: itzg/minecraft-server:missing",
                    ),
                    &failed_output_with_stdout("manifest unknown: manifest unknown"),
                )
            },
            |_image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                Ok(())
            },
        )
        .expect_err("hard failure should still stop stdio preflight before probe");

        assert!(err.contains("镜像或标签不存在"));
        assert_eq!(probe_calls.get(), 0);
    }

    #[test]
    fn facade_visible_preflight_path_allows_rcon_soft_failure_without_probe() {
        let probe_calls = Cell::new(0);

        run_facade_visible_preflight_path_from_outputs(
            "itzg/minecraft-server",
            "latest",
            DockerCommandMode::Rcon,
            &failed_output(
                "Error response from daemon: No such image: itzg/minecraft-server:latest",
            ),
            &failed_output("dial tcp 1.2.3.4:443: connectex: timeout"),
            |_image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                Err("probe should have been skipped".to_string())
            },
        )
        .expect("rcon user path should stay non-blocking across soft image preflight failures");

        assert_eq!(probe_calls.get(), 0);
    }

    #[test]
    fn facade_visible_preflight_path_allows_stdio_remote_resolvable_and_skips_probe() {
        let probe_calls = Cell::new(0);

        run_facade_visible_preflight_path_from_outputs(
            "itzg/minecraft-server",
            "latest",
            DockerCommandMode::DockerStdio,
            &failed_output(
                "Error response from daemon: No such image: itzg/minecraft-server:latest",
            ),
            &manifest_success_output(),
            |_image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                Err("probe should have been skipped".to_string())
            },
        )
        .expect("remote-resolvable docker_stdio user path should pass while skipping probe");

        assert_eq!(probe_calls.get(), 0);
    }

    #[test]
    fn facade_visible_preflight_path_requires_probe_before_allowing_local_cached_stdio() {
        let probe_calls = Cell::new(0);

        run_facade_visible_preflight_path_from_outputs(
            "itzg/minecraft-server",
            "latest",
            DockerCommandMode::DockerStdio,
            &local_success_output(),
            &failed_output("manifest unknown: manifest unknown"),
            |image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                assert_eq!(image_ref, "itzg/minecraft-server:latest");
                Ok(())
            },
        )
        .expect("local-cached docker_stdio user path should only pass after stdio probe");

        assert_eq!(probe_calls.get(), 1);
    }

    #[test]
    fn facade_visible_preflight_path_surfaces_stdio_probe_hint_for_local_cached_image() {
        let err = run_facade_visible_preflight_path_from_outputs(
            "itzg/minecraft-server",
            "java21",
            DockerCommandMode::DockerStdio,
            &local_success_output(),
            &failed_output("manifest unknown: manifest unknown"),
            |image_ref| {
                Err(format!(
                    "当前镜像不支持 --command-mode docker_stdio: image={} 未检测到 mc-send-to-console",
                    image_ref
                ))
            },
        )
        .expect_err("local-cached docker_stdio probe failures should remain user-visible");

        assert!(err.contains("docker_stdio"));
        assert!(err.contains("mc-send-to-console"));
        assert!(err.contains("itzg/minecraft-server:java21"));
    }

    #[test]
    fn facade_visible_preflight_path_blocks_hard_image_failure_before_probe() {
        let probe_calls = Cell::new(0);

        let err = run_facade_visible_preflight_path_from_outputs(
            "itzg/minecraft-server",
            "missing",
            DockerCommandMode::DockerStdio,
            &failed_output(
                "Error response from daemon: No such image: itzg/minecraft-server:missing",
            ),
            &failed_output_with_stdout("manifest unknown: manifest unknown"),
            |_image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                Ok(())
            },
        )
        .expect_err("hard image failures should still block the facade-visible user path");

        assert!(err.contains("镜像或标签不存在"));
        assert!(err.contains("manifest unknown"));
        assert_eq!(probe_calls.get(), 0);
    }

    #[test]
    fn handle_docker_stdio_preflight_outcome_probes_only_for_local_cached_images() {
        let calls = AtomicUsize::new(0);
        handle_docker_stdio_preflight_outcome(
            "itzg/minecraft-server:latest",
            DockerImageInspectOutcome::Available(DockerImageAvailability::LocalCached),
            |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        )
        .expect("local cached image should run stdio compatibility probe");

        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn handle_docker_stdio_preflight_outcome_skips_probe_for_remote_only_images() {
        let calls = AtomicUsize::new(0);
        handle_docker_stdio_preflight_outcome(
            "itzg/minecraft-server:latest",
            DockerImageInspectOutcome::Available(DockerImageAvailability::RemoteResolvable),
            |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        )
        .expect("remote-only image should skip stdio compatibility probe");

        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn handle_docker_stdio_preflight_outcome_skips_probe_for_soft_failures() {
        let calls = AtomicUsize::new(0);
        handle_docker_stdio_preflight_outcome(
            "itzg/minecraft-server:latest",
            DockerImageInspectOutcome::SoftFailure {
                failure_kind: DockerCommandFailureKind::Timeout,
                message: "docker manifest inspect timeout".to_string(),
            },
            |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        )
        .expect("soft failure should skip stdio compatibility probe");

        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn ensure_docker_stdio_image_support_with_output_surfaces_stderr_details() {
        let err = ensure_docker_stdio_image_support_with_output(
            "itzg/minecraft-server:java21",
            &output_with_status(1, "", "mc-send-to-console: not found"),
        )
        .expect_err("missing stdio helper should surface a user-facing error");

        assert!(err.contains("docker_stdio"));
        assert!(err.contains("mc-send-to-console"));
        assert!(err.contains("itzg/minecraft-server:java21"));
    }

    #[test]
    fn ensure_docker_stdio_image_support_with_output_falls_back_to_exit_code() {
        let err = ensure_docker_stdio_image_support_with_output(
            "itzg/minecraft-server:java21",
            &output_with_status(7, "", ""),
        )
        .expect_err("empty stdio probe output should still include exit code context");

        assert!(err.contains("退出码"));
        assert!(err.contains("java21"));
    }
}
