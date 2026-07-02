import { computed, onMounted, shallowRef, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { i18n } from "@language";
import { useSettingsStore } from "@stores/settingsStore";

export type SettingsSectionId = "appearance" | "general" | "developer-management";

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

export interface SettingsSectionItem {
  id: SettingsSectionId;
  label: string;
  title: string;
  description: string;
  hash: `#${SettingsSectionId}`;
}

const DEFAULT_SECTION_ID: SettingsSectionId = "appearance";

function buildSettingsSectionItems(): readonly SettingsSectionItem[] {
  return [
    {
      id: "appearance",
      label: i18n.t("settings.next.sections.appearance.label"),
      title: i18n.t("settings.next.sections.appearance.title"),
      description: i18n.t("settings.next.sections.appearance.description"),
      hash: "#appearance",
    },
    {
      id: "general",
      label: i18n.t("settings.next.sections.general.label"),
      title: i18n.t("settings.next.sections.general.title"),
      description: i18n.t("settings.next.sections.general.description"),
      hash: "#general",
    },
    {
      id: "developer-management",
      label: i18n.t("settings.next.sections.developer_management.label"),
      title: i18n.t("settings.next.sections.developer_management.title"),
      description: i18n.t("settings.next.sections.developer_management.description"),
      hash: "#developer-management",
    },
  ] as const;
}

function parseSettingsSectionHash(hash: string): SettingsSectionId | null {
  switch (hash) {
    case "#appearance":
      return "appearance";
    case "#general":
      return "general";
    case "#developer-management":
      return "developer-management";
    default:
      return null;
  }
}

function getSettingsSectionHash(sectionId: SettingsSectionId): `#${SettingsSectionId}` {
  return `#${sectionId}`;
}

export function useSettingsPage() {
  const route = useRoute();
  const router = useRouter();
  const settingsStore = useSettingsStore();

  const bootstrapping = shallowRef(true);
  const refreshing = shallowRef(false);

  async function loadPage(manual = false): Promise<void> {
    if (manual) {
      refreshing.value = true;
    } else {
      bootstrapping.value = true;
    }

    try {
      await settingsStore.loadSettings(manual);
    } finally {
      bootstrapping.value = false;
      refreshing.value = false;
    }
  }

  async function applySection(sectionId: SettingsSectionId, replace = false): Promise<void> {
    const nextHash = getSettingsSectionHash(sectionId);
    if (route.hash === nextHash) {
      return;
    }

    const target = {
      path: route.path,
      query: route.query,
      hash: nextHash,
    };

    if (replace) {
      await router.replace(target);
      return;
    }

    await router.push(target);
  }

  function selectSection(sectionId: SettingsSectionId): void {
    void applySection(sectionId);
  }

  watch(
    () => route.hash,
    (hash) => {
      if (parseSettingsSectionHash(hash)) {
        return;
      }

      void applySection(DEFAULT_SECTION_ID, true);
    },
    { immediate: true },
  );

  const sectionItems = computed(() => buildSettingsSectionItems());
  const currentSectionId = computed<SettingsSectionId>(
    () => parseSettingsSectionHash(route.hash) ?? DEFAULT_SECTION_ID,
  );
  const currentSection = computed<SettingsSectionItem>(() => {
    return (
      sectionItems.value.find((item) => item.id === currentSectionId.value) ?? sectionItems.value[0]
    );
  });
  const hasError = computed(() => Boolean(settingsStore.loadError));

  onMounted(() => {
    void loadPage(false);
  });

  return {
    bootstrapping,
    refreshing,
    sectionItems,
    hasError,
    currentSectionId,
    currentSection,
    loadPage,
    selectSection,
  };
}
