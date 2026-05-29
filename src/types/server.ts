export type ServerRuntimeKind = "local" | "docker_itzg";

export type LocalStartupMode = "starter" | "jar" | "bat" | "sh" | "ps1" | "custom";

export interface LocalRuntimeConfig {
  kind: "local";
  jar_path: string;
  startup_mode: LocalStartupMode;
  custom_command?: string | null;
  java_path: string;
  jvm_args: string[];
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
