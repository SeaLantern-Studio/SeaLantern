import { effectScope, getCurrentInstance, onUnmounted, ref, watch, type Ref } from "vue";

type SettingsRecord = Record<string, unknown>;

interface UseAutoSaveSettingsOptions {
  source: SettingsRecord;
  snapshot: Ref<string>;
  enabled: () => boolean;
  save: (payload: SettingsRecord) => Promise<void>;
  delay?: number;
  onError?: (error: unknown) => void;
}

function cloneSettings(source: SettingsRecord): SettingsRecord {
  return JSON.parse(JSON.stringify(source)) as SettingsRecord;
}

function serializeSettings(settings: SettingsRecord): string {
  return JSON.stringify(settings);
}

export function useAutoSaveSettings(options: UseAutoSaveSettingsOptions) {
  const saving = ref(false);
  const saveVersion = ref(0);
  const committedVersion = ref(0);
  const scope = effectScope();

  let timer: ReturnType<typeof setTimeout> | null = null;
  let inFlight: Promise<void> | null = null;

  async function runSave() {
    if (!options.enabled()) {
      return;
    }

    const payload = cloneSettings(options.source);
    const nextSnapshot = serializeSettings(payload);
    if (nextSnapshot === options.snapshot.value) {
      return;
    }

    if (inFlight) {
      await inFlight;
      if (!options.enabled()) {
        return;
      }
      const currentSnapshot = serializeSettings(cloneSettings(options.source));
      if (currentSnapshot === options.snapshot.value) {
        return;
      }
      return runSave();
    }

    const currentVersion = ++saveVersion.value;
    saving.value = true;
    inFlight = (async () => {
      await options.save(payload);
      if (currentVersion >= committedVersion.value) {
        committedVersion.value = currentVersion;
        options.snapshot.value = nextSnapshot;
      }
    })();

    try {
      await inFlight;
    } catch (error) {
      options.onError?.(error);
    } finally {
      if (inFlight) {
        inFlight = null;
      }
      saving.value = false;
    }
  }

  function schedule() {
    if (!options.enabled()) {
      return;
    }

    const nextSnapshot = serializeSettings(cloneSettings(options.source));
    if (nextSnapshot === options.snapshot.value) {
      return;
    }

    if (timer) {
      clearTimeout(timer);
    }

    timer = setTimeout(() => {
      timer = null;
      void runSave();
    }, options.delay ?? 300);
  }

  async function flush() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }

    await runSave();
  }

  function stop() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
    scope.stop();
  }

  scope.run(() => {
    watch(options.source, schedule, { deep: true });
  });

  if (getCurrentInstance()) {
    onUnmounted(() => {
      stop();
    });
  }

  return {
    saving,
    flush,
    stop,
  };
}
