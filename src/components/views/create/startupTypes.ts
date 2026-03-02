export type StartupMode = "starter" | "jar" | "bat" | "sh" | "ps1" | "custom";

export interface StartupCandidate {
  // 前端唯一标识，用于 Step3 选择态保存。
  id: string;
  // 启动模式：starter/jar/script/custom。
  mode: StartupMode;
  // UI 主标题。
  label: string;
  // UI 副标题（核心类型、Main-Class 或路径说明）。
  detail: string;
  // 启动文件的完整路径（custom 为空）。
  path: string;
  // 推荐优先级，数字越小优先级越高。
  recommended: number;
}
