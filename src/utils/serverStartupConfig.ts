import type { CpuPolicyConfig, JvmPresetConfig, JvmPresetId } from "@type/server";

export const MVP_JVM_PRESET_IDS: JvmPresetId[] = ["none", "g1_basic", "aikar_g1"];

const JVM_PRESET_ARGS: Record<JvmPresetId, string[]> = {
  none: [],
  g1_basic: [
    "-XX:+UseG1GC",
    "-XX:+ParallelRefProcEnabled",
    "-XX:MaxGCPauseMillis=200",
    "-XX:+UnlockExperimentalVMOptions",
  ],
  aikar_g1: [
    "-XX:+UseG1GC",
    "-XX:+ParallelRefProcEnabled",
    "-XX:MaxGCPauseMillis=200",
    "-XX:+UnlockExperimentalVMOptions",
    "-XX:+DisableExplicitGC",
    "-XX:+AlwaysPreTouch",
    "-XX:G1NewSizePercent=30",
    "-XX:G1MaxNewSizePercent=40",
    "-XX:G1HeapRegionSize=8M",
    "-XX:G1ReservePercent=20",
    "-XX:G1HeapWastePercent=5",
    "-XX:G1MixedGCCountTarget=4",
    "-XX:InitiatingHeapOccupancyPercent=15",
    "-XX:G1MixedGCLiveThresholdPercent=90",
    "-XX:G1RSetUpdatingPauseTimePercent=5",
    "-XX:SurvivorRatio=32",
    "-XX:+PerfDisableSharedMem",
    "-XX:MaxTenuringThreshold=1",
  ],
  throughput_basic: [
    "-XX:+UseParallelGC",
    "-XX:+UseAdaptiveSizePolicy",
    "-XX:MaxGCPauseMillis=500",
  ],
  paper_recommended_lite: [
    "-XX:+UseG1GC",
    "-XX:+ParallelRefProcEnabled",
    "-XX:MaxGCPauseMillis=150",
    "-XX:+UnlockExperimentalVMOptions",
    "-XX:+DisableExplicitGC",
    "-Dusing.aikars.flags=https://mcflags.emc.gs",
  ],
};

export function createDefaultCpuPolicy(): CpuPolicyConfig {
  return {
    mode: "off",
    count: null,
    explicit_set: null,
    sync_active_processor_count: true,
  };
}

export function createDefaultJvmPreset(): JvmPresetConfig {
  return {
    preset: "none",
  };
}

export function serializeJvmArgsText(text: string): string[] {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

export function deserializeJvmArgs(args: string[] | null | undefined): string {
  if (!Array.isArray(args) || args.length === 0) {
    return "";
  }

  return args.join("\n");
}

export function normalizeCpuPolicy(cpuPolicy: CpuPolicyConfig | null | undefined): CpuPolicyConfig {
  return {
    ...createDefaultCpuPolicy(),
    ...(cpuPolicy ?? {}),
  };
}

function isJvmPresetId(value: unknown): value is JvmPresetId {
  return (
    value === "none" ||
    value === "g1_basic" ||
    value === "aikar_g1" ||
    value === "throughput_basic" ||
    value === "paper_recommended_lite"
  );
}

export function normalizeJvmPreset(jvmPreset: JvmPresetConfig | null | undefined): JvmPresetConfig {
  const preset = jvmPreset?.preset;
  if (isJvmPresetId(preset)) {
    return { preset };
  }

  return createDefaultJvmPreset();
}

export function getJvmPresetArgs(preset: JvmPresetId): string[] {
  return [...JVM_PRESET_ARGS[preset]];
}

export function getJvmPresetPreviewArgs(preset: JvmPresetId, maxItems = 4): string[] {
  return getJvmPresetArgs(preset).slice(0, maxItems);
}
