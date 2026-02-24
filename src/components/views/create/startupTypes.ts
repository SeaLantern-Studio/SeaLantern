export type StartupMode = "starter" | "jar" | "bat" | "sh" | "ps1" | "custom";

export interface StartupCandidate {
  id: string;
  mode: StartupMode;
  label: string;
  detail: string;
  path: string;
  recommended: number;
}
