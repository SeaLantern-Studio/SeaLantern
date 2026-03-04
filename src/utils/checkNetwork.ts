import { invoke } from "@tauri-apps/api/core";

export type NetworkStatus = "green" | "yellow" | "red" | "error";
const IPS = [
  "223.5.5.5",
  "223.6.6.6",
  "180.76.76.76",
  "119.29.29.29",
  "1.0.0.1",
  "8.8.4.4",
] as const;
export const STATUS_LIST = ["green", "yellow", "red", "error"] as const;
const conditions = {
  green: {
    lossrate: 20,
    delay: 200,
  },
  yellow: {
    lossrate: 50,
    delay: 1500,
  },
  red: {
    lossrate: 100,
    delay: 60000,
  },
  error: {
    lossrate: 100,
    delay: 60000,
  },
};
async function ping(ip: string, timeout: number = 3000) {
  try {
    // 调用后端的 ping_host 命令
    const delay = await invoke<number>("ping_host", { host: ip, timeout: timeout });
    return delay;
  } catch (error) {
    console.error(`Error pinging ${ip}:`, error);
    return timeout;
  }
}
export async function getNetworkStatus() {
  let lostCount = 0;
  let maxDelay = 0;
  if (!navigator.onLine) {
    return "error" as NetworkStatus;
  }
  const delays = await Promise.all(IPS.map((ip) => ping(ip)));
  for (const delay of delays) {
    if (delay >= 3000) {
      lostCount++;
    } else {
      maxDelay = Math.max(maxDelay, delay);
    }
  }
  const lossrate = (lostCount / IPS.length) * 100;
  for (const status of STATUS_LIST) {
    if (lossrate > conditions[status].lossrate || maxDelay > conditions[status].delay) {
      continue;
    }
    return status as NetworkStatus;
  }
  return "error" as NetworkStatus;
}
