import { onUnmounted, ref } from "vue";

interface UseSerialPollingOptions {
  intervalMs: number;
  immediate?: boolean;
  task: () => Promise<void> | void;
}

export function useSerialPolling(options: UseSerialPollingOptions) {
  const isRunning = ref(false);

  let timer: ReturnType<typeof setTimeout> | null = null;
  let inFlight: Promise<void> | null = null;
  let disposed = false;

  function clearTimer() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
  }

  function scheduleNextRun(delay = options.intervalMs) {
    if (!isRunning.value || disposed) {
      return;
    }

    clearTimer();
    timer = setTimeout(() => {
      void runOnce();
    }, delay);
  }

  async function runOnce(): Promise<void> {
    if (disposed) {
      return;
    }

    if (inFlight) {
      return inFlight;
    }

    clearTimer();
    inFlight = Promise.resolve(options.task()).finally(() => {
      inFlight = null;
      if (isRunning.value && !disposed) {
        scheduleNextRun();
      }
    });

    return inFlight;
  }

  function start() {
    if (disposed || isRunning.value) {
      return;
    }

    isRunning.value = true;
    if (options.immediate ?? true) {
      void runOnce();
      return;
    }

    scheduleNextRun();
  }

  function stop() {
    isRunning.value = false;
    clearTimer();
  }

  onUnmounted(() => {
    disposed = true;
    stop();
  });

  return {
    isRunning,
    runOnce,
    start,
    stop,
  };
}
