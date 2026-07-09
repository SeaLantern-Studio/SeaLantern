export const SERVER_DELETE_MODES = ["record-only", "record-and-files"] as const;

export type ServerDeleteMode = (typeof SERVER_DELETE_MODES)[number];

export const DEFAULT_SERVER_DELETE_MODE: ServerDeleteMode = "record-and-files";

export function isServerDeleteMode(value: string): value is ServerDeleteMode {
  return SERVER_DELETE_MODES.includes(value as ServerDeleteMode);
}
