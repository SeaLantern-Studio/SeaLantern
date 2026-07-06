import type { ServerStatus } from "@type/common";

export const ACTIVE_SERVER_STATUSES = [
  "Running",
  "Starting",
] as const satisfies readonly ServerStatus[];

export type ActiveServerStatus = (typeof ACTIVE_SERVER_STATUSES)[number];

const activeServerStatusSet = new Set<ServerStatus>(ACTIVE_SERVER_STATUSES);

export function isActiveServerStatus(status: ServerStatus): status is ActiveServerStatus {
  return activeServerStatusSet.has(status);
}
