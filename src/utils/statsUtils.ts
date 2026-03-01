import type { EChartsOption } from "echarts";
import { ref, computed } from "vue";
import { i18n } from "@language";
import { systemApi, type SystemInfo } from "@api/system";

const systemInfo = ref<SystemInfo | null>(null);
const cpuUsage = ref(0);
const memUsage = ref(0);
const diskUsage = ref(0);
const cpuHistory = ref<number[]>([]);
const memHistory = ref<number[]>([]);
const statsViewMode = ref<"detail" | "gauge">("gauge");
const statsLoading = ref(true);

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

/**
 * 获取当前根字体大小（px）
 * @returns 根字体大小
 */
const getRootFontSize = (): number => {
  if (typeof window === "undefined") return 16;
  const fontSize = getComputedStyle(document.documentElement).fontSize;
  const parsed = parseFloat(fontSize);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 16;
};

/**
 * 解析 CSS 字体大小（支持 px 和 rem 单位）
 * @param varName CSS 变量名
 * @param defaultPx 默认像素值
 * @returns 解析后的像素值
 */
const parseFontSize = (varName: string, defaultPx: number): number => {
  const value = getCssVar(varName, `${defaultPx}px`);
  // 移除单位并解析为数字
  const numMatch = value.match(/^[\d.]+/);
  if (!numMatch) return defaultPx;

  const num = parseFloat(numMatch[0]);
  if (!Number.isFinite(num) || num <= 0) return defaultPx;

  // 如果是 rem，使用实际的根字体大小进行换算
  if (value.includes("rem")) {
    return num * getRootFontSize();
  }
  return num;
};

// ECharts 公共基础配置
const baseChartConfig: EChartsOption = {
  backgroundColor: "transparent",
  animation: true,
  animationDuration: 300,
  animationEasing: "cubicOut",
};

/**
 * 创建仪表盘图表配置
 * @param rawValue 原始值
 * @param colorVar 颜色变量
 * @param label 标签
 * @returns ECharts 配置
 */
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

// 折线图公共配置
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

/**
 * 创建折线图配置
 * @param data 数据数组
 * @param colorVar 颜色变量
 * @returns ECharts 配置
 */
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

// 计算属性
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
  // 强制重新计算，当主题变化时
  void themeVersion.value;
  return createLineOption(cpuHistory.value, "--sl-primary");
});

const memLineOption = computed(() => {
  // 强制重新计算，当主题变化时
  void themeVersion.value;
  return createLineOption(memHistory.value, "--sl-success");
});

/**
 * 获取系统信息
 * @returns Promise<void>
 */
async function fetchSystemInfo() {
  try {
    const info = await systemApi.getSystemInfo();
    systemInfo.value = info;
    // clamp CPU usage to 0-100 (sysinfo can sometimes return >100%)
    cpuUsage.value = Math.min(100, Math.max(0, Math.round(info.cpu.usage)));
    memUsage.value = Math.min(100, Math.max(0, Math.round(info.memory.usage)));
    diskUsage.value = Math.min(100, Math.max(0, Math.round(info.disk.usage)));
    cpuHistory.value.push(cpuUsage.value);
    memHistory.value.push(memUsage.value);
    if (cpuHistory.value.length > 30) cpuHistory.value.shift();
    if (memHistory.value.length > 30) memHistory.value.shift();
    statsLoading.value = false;
  } catch (e) {
    console.error("Failed to fetch system info:", e);
    statsLoading.value = false;
  }
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

/**
 * 停止主题和无障碍模式变化监听
 */
function stopThemeObserver() {
  if (themeObserver) {
    themeObserver.disconnect();
    themeObserver = null;
  }
}

/**
 * 清理系统状态相关资源
 */
function cleanupStatsResources() {
  stopThemeObserver();
}

export {
  // 响应式数据
  systemInfo,
  cpuUsage,
  memUsage,
  diskUsage,
  cpuHistory,
  memHistory,
  statsViewMode,
  statsLoading,
  themeVersion,

  // 计算属性
  cpuGaugeOption,
  memGaugeOption,
  diskGaugeOption,
  cpuLineOption,
  memLineOption,

  // 工具函数
  getCssVar,
  getRootFontSize,
  parseFontSize,
  createGaugeOption,
  createLineOption,
  fetchSystemInfo,
  startThemeObserver,
  stopThemeObserver,
  cleanupStatsResources,
};
