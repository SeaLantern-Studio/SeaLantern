import { computed, onMounted, shallowRef } from "vue";
import { checkDeveloperMode } from "@api/logging";
import { isBrowserEnv } from "@api/tauri";
import { useAuthStore } from "@stores/authStore";
import { useSettingsStore } from "@stores/settingsStore";

export type SettingsCoverageTone = "next" | "hybrid" | "classic";

export interface SettingsSummaryChip {
  label: string;
  tone: "primary" | "neutral" | "warning";
}

export interface SettingsSummaryFact {
  label: string;
  value: string;
}

export interface SettingsEntryAction {
  id: string;
  label: string;
  variant?: "primary" | "secondary" | "ghost";
  disabled?: boolean;
}

export interface SettingsEntryItem {
  id: string;
  eyebrow: string;
  title: string;
  description: string;
  coverageLabel: string;
  coverageTone: SettingsCoverageTone;
  metaItems: string[];
  notes: string[];
  actions: SettingsEntryAction[];
}

export interface SettingsCoverageItem {
  title: string;
  description: string;
  tone: SettingsCoverageTone;
}

function formatAuthSource(source: string | null): string {
  switch (source) {
    case "session":
      return "当前标签页凭据";
    case "local":
      return "已记住此浏览器";
    case "env":
      return "预置环境凭据";
    case "runtime":
      return "运行时注入凭据";
    default:
      return "已认证";
  }
}

function formatRuntimeMode(): string {
  return isBrowserEnv() ? "Browser / Headless Web" : "Desktop App";
}

function formatThemeLabel(theme: string): string {
  switch (theme) {
    case "auto":
      return "跟随系统";
    case "dark":
      return "深色";
    case "light":
      return "浅色";
    default:
      return theme;
  }
}

function formatWindowEffectLabel(effect: string): string {
  switch (effect) {
    case "off":
      return "关闭";
    case "blur":
      return "模糊";
    case "acrylic":
      return "Acrylic";
    case "mica":
      return "Mica";
    case "tabbed":
      return "Tabbed";
    case "vibrancy":
      return "Vibrancy";
    default:
      return effect;
  }
}

function navigateToPath(path: string): void {
  window.location.assign(path);
}

export function useSettingsPage() {
  const settingsStore = useSettingsStore();
  const authStore = useAuthStore();

  const bootstrapping = shallowRef(true);
  const refreshing = shallowRef(false);
  const developerRouteAvailable = shallowRef<boolean | null>(null);
  const developerRouteError = shallowRef<string | null>(null);

  async function loadPage(manual = false): Promise<void> {
    if (manual) {
      refreshing.value = true;
    } else {
      bootstrapping.value = true;
    }

    developerRouteError.value = null;

    try {
      await settingsStore.loadSettings(manual);

      const developerResult = await checkDeveloperMode();
      developerRouteAvailable.value = developerResult;
    } catch (error) {
      developerRouteAvailable.value = null;
      developerRouteError.value = error instanceof Error ? error.message : String(error);
    } finally {
      bootstrapping.value = false;
      refreshing.value = false;
    }
  }

  function performAction(actionId: string): void {
    switch (actionId) {
      case "refresh":
        void loadPage(true);
        return;
      case "open-paint":
        navigateToPath("/paint");
        return;
      case "open-plugins":
        navigateToPath("/plugins");
        return;
      case "open-market":
        navigateToPath("/market");
        return;
      case "open-auth":
        navigateToPath("/auth");
        return;
      case "open-developer":
        navigateToPath("/developer");
        return;
      default:
        return;
    }
  }

  const instanceHost = computed(() => window.location.host || "unknown host");
  const runtimeModeLabel = computed(() => formatRuntimeMode());
  const authSourceLabel = computed(() => formatAuthSource(authStore.source));
  const effectiveThemeLabel = computed(() =>
    formatThemeLabel(settingsStore.settings.theme || "auto"),
  );
  const windowEffectLabel = computed(() =>
    formatWindowEffectLabel(settingsStore.settings.window_effect || "off"),
  );
  const developerModeEnabled = computed(() => Boolean(settingsStore.settings.developer_mode));
  const aggregatedEntryCount = 4;

  const summaryChips = computed<SettingsSummaryChip[]>(() => {
    const chips: SettingsSummaryChip[] = [
      { label: runtimeModeLabel.value, tone: "primary" },
      { label: authSourceLabel.value, tone: "neutral" },
    ];

    if (settingsStore.settings.minimal_mode) {
      chips.push({ label: "简洁模式已开启", tone: "warning" });
    }

    if (developerModeEnabled.value) {
      chips.push({ label: "开发者模式已开启", tone: "neutral" });
    }

    return chips;
  });

  const summaryFacts = computed<SettingsSummaryFact[]>(() => [
    { label: "当前实例", value: instanceHost.value },
    {
      label: "主题状态",
      value: `${effectiveThemeLabel.value} / ${settingsStore.getEffectiveTheme()}`,
    },
    { label: "界面语言", value: settingsStore.settings.language || "zh-CN" },
    { label: "已聚合入口", value: `${aggregatedEntryCount} 组` },
  ]);

  const entryItems = computed<SettingsEntryItem[]>(() => {
    const developerDescription = developerModeEnabled.value
      ? developerRouteAvailable.value === false
        ? "设置里已经开启开发者模式，但当前访问环境仍未开放 classic /developer 页面。"
        : "开发者能力仍以 classic 页面为主，next 这里先提供状态说明和跳转入口。"
      : "是否展示开发工具、日志面板与测试入口，仍由 classic 设置里的开发者模式控制。";

    return [
      {
        id: "appearance",
        eyebrow: "Appearance",
        title: "外观与主题摘要",
        description:
          "next 先展示当前主题、字号、窗口材质和简洁模式状态，不在这轮重做 classic 的外观编辑器。",
        coverageLabel: "classic 编辑",
        coverageTone: "classic",
        metaItems: [
          `主题：${effectiveThemeLabel.value}`,
          `字号：${settingsStore.settings.font_size || 14}px`,
          `窗口材质：${windowEffectLabel.value}`,
          settingsStore.settings.minimal_mode ? "简洁模式：开启" : "简洁模式：关闭",
        ],
        notes: [
          "背景图、色彩方案、窗口材质和文本颜色仍沿用 classic 现有实现。",
          "next 这里只做读取与入口聚合，不改 classic 外观数据流。",
        ],
        actions: [{ id: "open-paint", label: "打开 classic 个性化页", variant: "primary" }],
      },
      {
        id: "plugins",
        eyebrow: "Plugins",
        title: "插件相关设置入口",
        description:
          "已安装插件管理已经在 next `/plugins` 承载；插件市场、插件详情页和具体设置页仍保持 classic 路径。",
        coverageLabel: "next + classic",
        coverageTone: "hybrid",
        metaItems: [
          "next：已安装插件列表与启停管理",
          "classic：插件市场、插件详情、插件设置页",
          developerModeEnabled.value ? "开发者模式：可能暴露更多调试入口" : "开发者模式：当前关闭",
        ],
        notes: [
          "这轮不引入新的插件设置协议，也不把 classic 插件配置系统搬进 next。",
          "next 设置页只负责聚合已经稳定存在的插件入口。",
        ],
        actions: [
          { id: "open-plugins", label: "打开 next 插件页", variant: "primary" },
          { id: "open-market", label: "打开 classic 市场", variant: "secondary" },
        ],
      },
      {
        id: "session",
        eyebrow: "Session",
        title: "Browser / Headless 会话说明",
        description:
          "当前页面只复用 next 既有认证语义，展示访问模式、实例上下文和凭据来源，不改认证协议。",
        coverageLabel: "next 直承载",
        coverageTone: "next",
        metaItems: [
          `访问模式：${runtimeModeLabel.value}`,
          `凭据来源：${authSourceLabel.value}`,
          authStore.hasSavedCredential ? "浏览器中存在已保存凭据" : "浏览器中未保存凭据",
          `实例：${instanceHost.value}`,
        ],
        notes: [
          "如果需要重新验证当前浏览器会话，可直接回到 next `/auth`。",
          "这一组内容以说明态为主，不额外扩展 backend 或会话协议。",
        ],
        actions: [{ id: "open-auth", label: "重新验证会话", variant: "secondary" }],
      },
      {
        id: "developer",
        eyebrow: "Advanced",
        title: "开发者 / 高级入口",
        description: developerDescription,
        coverageLabel: "classic 入口",
        coverageTone: "classic",
        metaItems: [
          developerModeEnabled.value ? "设置状态：开发者模式已开启" : "设置状态：开发者模式未开启",
          developerRouteAvailable.value === null
            ? "classic /developer：未确认"
            : developerRouteAvailable.value
              ? "classic /developer：当前可进入"
              : "classic /developer：当前不可进入",
          developerRouteError.value
            ? `检查结果：${developerRouteError.value}`
            : "入口检查：已复用现有 gate 语义",
        ],
        notes: [
          "开发者模式开关本身仍在 classic 设置系统里维护。",
          "next 这轮不做 developer 平台化，也不搬迁 DeveloperView。",
        ],
        actions: [
          {
            id: "open-developer",
            label: "打开 classic 开发工具",
            variant: "secondary",
            disabled: !developerModeEnabled.value || developerRouteAvailable.value === false,
          },
        ],
      },
    ];
  });

  const nextCoverageItems = computed<SettingsCoverageItem[]>(() => [
    {
      title: "当前上下文与访问模式",
      description: "复用现有 auth / browser 语义，说明当前实例、凭据来源和浏览器会话状态。",
      tone: "next",
    },
    {
      title: "外观与插件入口聚合",
      description: "把已经稳定的 next 页面或 classic 子页入口收敛到一个设置入口页中。",
      tone: "hybrid",
    },
    {
      title: "刷新与重新整理入口",
      description: "允许在当前页重新读取 settings 状态和开发者入口 gate。",
      tone: "next",
    },
  ]);

  const classicCoverageItems = computed<SettingsCoverageItem[]>(() => [
    {
      title: "通用设置 / 数据目录 / 插件目录",
      description: "仍由 classic SettingsView 体系维护，这轮不迁移对应编辑器与表单。",
      tone: "classic",
    },
    {
      title: "服务器默认项 / 网络设置",
      description: "仍保留 classic 现有卡片与保存逻辑，不在 next 里重建。",
      tone: "classic",
    },
    {
      title: "开发者模式开关",
      description: "是否开启开发者页面，继续沿用 classic 设置和现有 backend gate 判定。",
      tone: "classic",
    },
  ]);

  const pageErrorMessage = computed(() => settingsStore.loadError || developerRouteError.value);

  onMounted(() => {
    void loadPage(false);
  });

  return {
    bootstrapping,
    refreshing,
    pageErrorMessage,
    summaryChips,
    summaryFacts,
    entryItems,
    nextCoverageItems,
    classicCoverageItems,
    loadPage,
    performAction,
  };
}
