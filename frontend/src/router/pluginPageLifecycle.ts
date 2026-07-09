import { onPageChanged } from "@api/plugin";

let pendingTimers: number[] = [];
let currentPath = "";

function clearPendingTimers() {
  for (const timer of pendingTimers) {
    clearTimeout(timer);
  }
  pendingTimers = [];
}

function schedulePageSignal(path: string, delayMs: number) {
  pendingTimers.push(
    window.setTimeout(() => {
      if (currentPath !== path) {
        return;
      }
      onPageChanged(path).catch(() => {});
    }, delayMs),
  );
}

export function notifyPluginPageLifecycle(path: string) {
  clearPendingTimers();
  currentPath = path;

  onPageChanged(path).catch(() => {});

  // 兼容现有插件协议：页面稳定后只在仍是当前路径时补发一次。
  schedulePageSignal(path, 700);
}
