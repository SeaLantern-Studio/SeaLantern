import type { EChartsOption } from "echarts";
import { ref, computed } from "vue";
import { i18n } from "@language";
import { systemApi, type SystemInfo, type ServerResourceUsage } from "@api/system";

const systemInfo = ref<SystemInfo | null>(null);
const serverSystemInfo = ref<ServerResourceUsage | null>(null);
const cpuUsage = ref(0);
const memUsage = ref(0);
const diskUsage = ref(0);
const cpuHistory = ref<number[]>([]);
const memHistory = ref<number[]>([]);
const serverCpuUsage = ref(0);
const serverMemUsage = ref(0);
const serverDiskUsage = ref(0);
const serverCpuHistory = ref<number[]>([]);
const serverMemHistory = ref<number[]>([]);
const statsViewMode = ref<"detail" | "gauge">("gauge");
const statsLoading = ref(true);
const serverStatsLoading = ref(true);
const serverStatsError = ref(false);

const themeVersion = ref(0);

let themeObserver: MutationObserver | null = null;

let cssVarCache: Map<string, { value: string; timestamp: number }> = new Map();
const CSS_VAR_CACHE_TTL = 100;

const getCssVar = (varName: string, defaultValue: string): string => {
  if (typeof window === "undefined") return defaultValue;

  const now = Date.now();
  const cached = cssVarCache.get(varName);
  if (cached && now - cached.timestamp < CSS_VAR_CACHE_TTL) {
    return cached.value || defaultValue;
  }

  const value = getComputedStyle(document.documentElement).getPropertyValue(varName).trim();
  cssVarCache.set(varName, { value, timestamp: now });
  return value || defaultValue;
};

const invalidateCssVarCache = () => {
  cssVarCache.clear();
};

const getRootFontSize = (): number => {
  if (typeof window === "undefined") return 16;
  const fontSize = getComputedStyle(document.documentElement).fontSize;
  const parsed = parseFloat(fontSize);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 16;
};

const parseFontSize = (varName: string, defaultPx: number): number => {
  const value = getCssVar(varName, `${defaultPx}px`);
  const numMatch = value.match(/^[\d.]+/);
  if (!numMatch) return defaultPx;

  const num = parseFloat(numMatch[0]);
  if (!Number.isFinite(num) || num <= 0) return defaultPx;

  if (value.includes("rem")) {
    return num * getRootFontSize();
  }
  return num;
};

const baseChartConfig: EChartsOption = {
  backgroundColor: "transparent",
  animation: true,
  animationDuration: 300,
  animationEasing: "cubicOut",
};

const createGaugeOption = (rawValue: number, colorVar: string, label: string): EChartsOption => {
  const value = Number.isFinite(rawValue) ? Math.min(100, Math.max(0, rawValue)) : 0;
  const fontSize = parseFontSize("--sl-font-size-sm", 13);
  const fontFamily = getCssVar("--sl-font-mono", "monospace");
  const color = getCssVar(colorVar, "#3b82f6");
  const textColor = getCssVar("--sl-text-primary", "#1f2937");
  const borderColor = getCssVar("--sl-border", "#e5e7eb");

  return {
    ...baseChartConfig,
    series: [
      {
        type: "pie",
        radius: ["65%", "80%"],
        center: ["50%", "45%"],
        avoidLabelOverlap: false,
        silent: true,
        label: {
          show: true,
          position: "center",
          formatter: () => `${value}%`,
          fontSize: fontSize,
          fontWeight: 600,
          fontFamily: fontFamily,
          color: textColor,
        },
        labelLine: {
          show: false,
        },
        data: [
          {
            value: value,
            name: label,
            itemStyle: {
              color: color,
              borderRadius: 3,
            },
          },
          {
            value: 100 - value,
            name: i18n.t("home.remaining"),
            itemStyle: {
              color: borderColor,
            },
            label: {
              show: false,
            },
            emphasis: {
              disabled: true,
            },
          },
        ],
      },
    ],
  };
};

const baseLineConfig: EChartsOption = {
  ...baseChartConfig,
  grid: {
    left: 0,
    right: 0,
    top: 0,
    bottom: 0,
    show: false,
  },
  xAxis: {
    type: "category",
    show: false,
    boundaryGap: false,
  },
  yAxis: {
    type: "value",
    show: false,
    min: 0,
    max: 100,
  },
};

const createLineOption = (data: number[], colorVar: string): EChartsOption => {
  const color = getCssVar(colorVar, "#3b82f6");

  return {
    ...baseLineConfig,
    xAxis: {
      ...baseLineConfig.xAxis,
      data: data.map((_, i) => i),
    },
    series: [
      {
        type: "line",
        data: data,
        smooth: false,
        symbol: "none",
        lineStyle: {
          width: 2,
          color: color,
        },
        areaStyle: {
          color: color,
          opacity: 0.15,
        },
      },
    ],
  };
};

const cpuGaugeOption = computed(() => {
  //@ts-ignore
  const _ = themeVersion.value;
  return createGaugeOption(cpuUsage.value, "--sl-primary", i18n.t("home.cpu"));
});

const memGaugeOption = computed(() => {
  //@ts-ignore
  const _ = themeVersion.value;
  return createGaugeOption(memUsage.value, "--sl-success", i18n.t("home.memory"));
});

const diskGaugeOption = computed(() => {
  //@ts-ignore
  const _ = themeVersion.value;
  return createGaugeOption(diskUsage.value, "--sl-warning", i18n.t("home.disk"));
});

const cpuLineOption = computed(() => {
  void themeVersion.value;
  return createLineOption(cpuHistory.value, "--sl-primary");
});

const memLineOption = computed(() => {
  void themeVersion.value;
  return createLineOption(memHistory.value, "--sl-success");
});

function pushHistory(target: number[], value: number) {
  target.push(value);
  if (target.length > 30) target.shift();
}

function applySystemStatsInfo(info: SystemInfo) {
  systemInfo.value = info;
  cpuUsage.value = Math.min(100, Math.max(0, Math.round(info.cpu.usage)));
  memUsage.value = Math.min(100, Math.max(0, Math.round(info.memory.usage)));
  diskUsage.value = Math.min(100, Math.max(0, Math.round(info.disk.usage)));
  pushHistory(cpuHistory.value, cpuUsage.value);
  pushHistory(memHistory.value, memUsage.value);
  statsLoading.value = false;
}

function applyServerStatsInfo(info: ServerResourceUsage) {
  serverSystemInfo.value = info;
  serverCpuUsage.value = Math.min(100, Math.max(0, Math.round(info.cpu.usage)));
  serverMemUsage.value = Math.min(100, Math.max(0, Math.round(info.memory.usage)));
  serverDiskUsage.value = Math.min(100, Math.max(0, Math.round(info.disk.usage)));
  pushHistory(serverCpuHistory.value, serverCpuUsage.value);
  pushHistory(serverMemHistory.value, serverMemUsage.value);
  serverStatsError.value = false;
  serverStatsLoading.value = false;
}

async function fetchSystemInfo() {
  try {
    const info = await systemApi.getSystemInfo();
    applySystemStatsInfo(info);
  } catch (e) {
    console.error("Failed to fetch system info:", e);
    statsLoading.value = false;
  }
}

async function fetchServerResourceUsage(serverId: string) {
  try {
    const info = await systemApi.getServerResourceUsage(serverId);
    applyServerStatsInfo(info);
  } catch (e) {
    console.error("Failed to fetch server resource usage:", e);
    serverStatsError.value = true;
    serverStatsLoading.value = false;
  }
}

function resetStatsHistory() {
  serverCpuHistory.value = [];
  serverMemHistory.value = [];
}

function startThemeObserver() {
  stopThemeObserver();

  themeObserver = new MutationObserver((mutations) => {
    mutations.forEach((mutation) => {
      if (
        mutation.type === "attributes" &&
        (mutation.attributeName === "data-theme" || mutation.attributeName === "data-senior")
      ) {
        invalidateCssVarCache();
        themeVersion.value++;
      }
    });
  });

  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["data-theme", "data-senior"],
  });
}

function stopThemeObserver() {
  if (themeObserver) {
    themeObserver.disconnect();
    themeObserver = null;
  }
}

function cleanupStatsResources() {
  stopThemeObserver();
}

const serverCpuGaugeOption = computed(() => {
  //@ts-ignore
  const _ = themeVersion.value;
  return createGaugeOption(serverCpuUsage.value, "--sl-primary", i18n.t("home.cpu"));
});

const serverMemGaugeOption = computed(() => {
  //@ts-ignore
  const _ = themeVersion.value;
  return createGaugeOption(serverMemUsage.value, "--sl-success", i18n.t("home.memory"));
});

const serverDiskGaugeOption = computed(() => {
  //@ts-ignore
  const _ = themeVersion.value;
  return createGaugeOption(serverDiskUsage.value, "--sl-warning", i18n.t("home.disk"));
});

const serverCpuLineOption = computed(() => {
  void themeVersion.value;
  return createLineOption(serverCpuHistory.value, "--sl-primary");
});

const serverMemLineOption = computed(() => {
  void themeVersion.value;
  return createLineOption(serverMemHistory.value, "--sl-success");
});

export {
  systemInfo,
  serverSystemInfo,
  cpuUsage,
  memUsage,
  diskUsage,
  cpuHistory,
  memHistory,
  serverCpuUsage,
  serverMemUsage,
  serverDiskUsage,
  serverCpuHistory,
  serverMemHistory,
  statsViewMode,
  statsLoading,
  serverStatsLoading,
  serverStatsError,
  themeVersion,
  cpuGaugeOption,
  memGaugeOption,
  diskGaugeOption,
  cpuLineOption,
  memLineOption,
  serverCpuGaugeOption,
  serverMemGaugeOption,
  serverDiskGaugeOption,
  serverCpuLineOption,
  serverMemLineOption,
  getCssVar,
  getRootFontSize,
  parseFontSize,
  createGaugeOption,
  createLineOption,
  fetchSystemInfo,
  fetchServerResourceUsage,
  resetStatsHistory,
  startThemeObserver,
  stopThemeObserver,
  cleanupStatsResources,
};
