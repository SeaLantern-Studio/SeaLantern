export type ServerRuntimeKind = "local" | "docker_itzg";

export type LocalStartupMode = "starter" | "jar" | "bat" | "sh" | "ps1" | "custom";
export type LocalTerminalMode = "pipe_managed" | "pty_managed";

export type CpuPolicyMode = "off" | "count" | "explicit";

export interface CpuPolicyConfig {
  mode: CpuPolicyMode;
  count?: number | null;
  explicit_set?: string | null;
  sync_active_processor_count: boolean;
}

export type JvmPresetId =
  | "none"
  | "g1_basic"
  | "aikar_g1"
  | "throughput_basic"
  | "paper_recommended_lite";

export interface JvmPresetConfig {
  preset: JvmPresetId;
}

export interface LocalRuntimeConfig {
  kind: "local";
  jar_path: string;
  startup_mode: LocalStartupMode;
  custom_command?: string | null;
  java_path: string;
  jvm_args: string[];
  terminal_mode: LocalTerminalMode;
  cpu_policy: CpuPolicyConfig;
  jvm_preset: JvmPresetConfig;
}

export interface PublishedPort {
  host_port: number;
  container_port: number;
  protocol: string;
}

export interface VolumeMount {
  source: string;
  target: string;
  read_only: boolean;
}

export type DockerBackendKind = "cli" | "engine_api";
export type DockerCommandMode = "rcon" | "docker_stdio";

export interface RconConfig {
  host: string;
  port: number;
  password: string;
}

export interface DockerItzgRuntimeConfig {
  kind: "docker_itzg";
  image: string;
  image_tag: string;
  container_name: string;
  type_value: string;
  version: string;
  data_dir_mount: string;
  published_game_port: number;
  env: Record<string, string>;
  extra_ports: PublishedPort[];
  volume_mounts: VolumeMount[];
  docker_backend_kind: DockerBackendKind;
  command_mode: DockerCommandMode;
  rcon?: RconConfig | null;
  jvm_args: string[];
  cpu_policy: CpuPolicyConfig;
  jvm_preset: JvmPresetConfig;
}

export type ServerRuntimeConfig = LocalRuntimeConfig | DockerItzgRuntimeConfig;

/**
 * 服务器实例类型
 */
export interface ServerInstance {
  id: string;
  name: string;
  aliases: string[];
  core_type: string;
  core_version: string;
  mc_version: string;
  path: string;
  port: number;
  max_memory: number;
  min_memory: number;
  created_at: number;
  last_started_at: number | null;
  runtime_kind: ServerRuntimeKind;
  runtime: ServerRuntimeConfig;
}

export function getLocalRuntime(server: ServerInstance): LocalRuntimeConfig | null {
  return server.runtime.kind === "local" ? server.runtime : null;
}

export function getServerJarPath(server: ServerInstance): string {
  return getLocalRuntime(server)?.jar_path ?? "";
}

export function getServerStartupMode(server: ServerInstance): LocalStartupMode {
  return getLocalRuntime(server)?.startup_mode ?? "jar";
}

export function getServerJavaPath(server: ServerInstance): string {
  return getLocalRuntime(server)?.java_path ?? "";
}

/**
 * 服务器命令类型
 */
export interface ServerCommand {
  id: string;
  name: string;
  command: string;
}

export interface LocalLaunchDetail {
  startup_mode: LocalStartupMode;
  java_path: string;
  launch_target: string;
  effective_jvm_args: string[];
  command_preview: string;
}

export interface DockerLaunchDetail {
  runtime_kind: "docker_itzg";
  image: string;
  image_tag: string;
  container_name: string;
  cpuset_applied: string | null;
  jvm_preset: JvmPresetId | string;
  jvm_opts_preview: string | null;
  jvm_xx_opts_preview: string | null;
  jvm_opts_args_count: number;
  jvm_xx_opts_args_count: number;
  jvm_opts_overridden_by_runtime_env: boolean;
  jvm_xx_opts_overridden_by_runtime_env: boolean;
  active_processor_count_status:
    | "disabled"
    | "injected"
    | "skipped_by_jvm_args"
    | "skipped_by_runtime_env_override"
    | string;
  active_processor_count_value: number | null;
  docker_args_preview: string[];
  command_preview: string;
}
