/**
 * 服务器实例类型
 */
export interface ServerInstance {
  id: string;
  name: string;
  core_type: string;
  core_version: string;
  mc_version: string;
  path: string;
  jar_path: string;
  startup_mode: "starter" | "jar" | "bat" | "sh" | "ps1" | "custom";
  custom_command?: string | null;
  java_path: string;
  max_memory: number;
  min_memory: number;
  jvm_args: string[];
  port: number;
  created_at: number;
  last_started_at: number | null;
}

/**
 * 服务器命令类型
 */
export interface ServerCommand {
  id: string;
  name: string;
  command: string;
}
