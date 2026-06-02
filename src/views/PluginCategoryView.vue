<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, watch } from "vue";
import { onBeforeRouteLeave, onBeforeRouteUpdate } from "vue-router";
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import { usePluginStore } from "@stores/pluginStore";
import { i18n } from "@language";
import type { PluginInfo } from "@type/plugin";
import { getLocalizedPluginDescription } from "@type/plugin";
import PluginSettingsFormCard from "@views/plugins/PluginSettingsFormCard.vue";
import { useAutoSaveSettings } from "@views/plugins/useAutoSaveSettings";
import { Palette } from "lucide-vue-next";

type PluginSettingValue = string | number | boolean | null;
type PluginSettingsRecord = Record<string, PluginSettingValue>;

const props = defineProps<{
  pluginId: string;
}>();

const pluginStore = usePluginStore();

const plugin = ref<PluginInfo | null>(null);
const settingsForm = reactive<PluginSettingsRecord>({});
const dependentSettingsForms = reactive<Record<string, PluginSettingsRecord>>({});
const dependentPlugins = ref<PluginInfo[]>([]);
const loading = ref(true);
const isInitializingForms = ref(false);
const mainSettingsSnapshot = ref("{}");
const dependentSettingsSnapshots = reactive<Record<string, string>>({});

const sidebarConfig = computed(() => plugin.value?.manifest.sidebar);
const categoryLabel = computed(
  () => sidebarConfig.value?.label || plugin.value?.manifest.name || "",
);
const showDependents = computed(() => sidebarConfig.value?.show_dependents !== false);
const pluginPresets = computed(() => plugin.value?.manifest.presets ?? null);
const isThemeProvider = computed(() => {
  return plugin.value?.manifest.capabilities?.includes("theme-provider") ?? false;
});

function getDependencyId(dep: string | { id: string; version?: string }): string {
  return typeof dep === "string" ? dep : dep.id;
}

function clearRecord(record: Record<string, unknown>) {
  Object.keys(record).forEach((key) => delete record[key]);
}

function serializeSettings(settings: Record<string, unknown>): string {
  return JSON.stringify(settings);
}

function getDefaultValue(type: string): PluginSettingValue {
  switch (type) {
    case "number":
      return 0;
    case "boolean":
    case "checkbox":
      return false;
    default:
      return "";
  }
}

function getFieldStringValue(value: unknown): string {
  return value == null ? "" : String(value);
}

function getFieldSelectValue(value: unknown): string | number | undefined {
  if (typeof value === "string" || typeof value === "number") {
    return value;
  }
  return undefined;
}

function getFieldOptions(options: Array<{ value: string; label: string }> | undefined) {
  return options ?? [];
}

function updateSettingsField(
  form: PluginSettingsRecord,
  key: string,
  value: string | number | boolean,
) {
  form[key] = value;
}

async function loadDependentPlugins() {
  if (!plugin.value) {
    dependentPlugins.value = [];
    return;
  }

  const candidates = pluginStore.plugins.filter((candidate) => {
    if (candidate.state !== "enabled") {
      return false;
    }
    if (candidate.manifest.id === props.pluginId) {
      return false;
    }

    const dependencies = [
      ...(candidate.manifest.dependencies || []),
      ...(candidate.manifest.optional_dependencies || []),
    ];
    return dependencies.some((dependency) => getDependencyId(dependency) === props.pluginId);
  });

  const settingsPromises = candidates
    .filter((candidate) => candidate.manifest.settings?.length)
    .map(async (candidate) => {
      const savedSettings = await pluginStore.getPluginSettings(candidate.manifest.id);
      const form: PluginSettingsRecord = {
        ...(savedSettings as Record<string, PluginSettingValue>),
      };
      for (const field of candidate.manifest.settings || []) {
        if (form[field.key] === undefined) {
          form[field.key] = field.default ?? getDefaultValue(field.type);
        }
      }
      return { plugin: candidate, form };
    });

  const results = await Promise.all(settingsPromises);
  dependentPlugins.value = results.map((item) => item.plugin);
  clearRecord(dependentSettingsForms);
  clearRecord(dependentSettingsSnapshots);
  for (const { plugin: depPlugin, form } of results) {
    dependentSettingsForms[depPlugin.manifest.id] = form;
    dependentSettingsSnapshots[depPlugin.manifest.id] = serializeSettings(form);
  }
}

const mainAutoSave = useAutoSaveSettings({
  source: settingsForm,
  snapshot: mainSettingsSnapshot,
  enabled: () => Boolean(plugin.value) && !isInitializingForms.value,
  save: async (payload) => {
    const pluginId = plugin.value?.manifest.id;
    if (!pluginId) {
      return;
    }
    await pluginStore.setPluginSettings(pluginId, payload);
  },
});

const dependentAutoSaves = new Map<string, ReturnType<typeof useAutoSaveSettings>>();

const saving = computed(() => {
  if (mainAutoSave.saving.value) {
    return true;
  }
  return Array.from(dependentAutoSaves.values()).some((entry) => entry.saving.value);
});

function syncDependentAutoSaves() {
  const activeIds = new Set(dependentPlugins.value.map((depPlugin) => depPlugin.manifest.id));

  for (const [pluginId, autoSave] of dependentAutoSaves.entries()) {
    if (!activeIds.has(pluginId)) {
      autoSave.stop();
      dependentAutoSaves.delete(pluginId);
    }
  }

  for (const depPlugin of dependentPlugins.value) {
    const pluginId = depPlugin.manifest.id;
    if (dependentAutoSaves.has(pluginId)) {
      continue;
    }

    if (!dependentSettingsForms[pluginId]) {
      dependentSettingsForms[pluginId] = {};
    }
    if (dependentSettingsSnapshots[pluginId] === undefined) {
      dependentSettingsSnapshots[pluginId] = serializeSettings(dependentSettingsForms[pluginId]);
    }

    dependentAutoSaves.set(
      pluginId,
      useAutoSaveSettings({
        source: dependentSettingsForms[pluginId],
        snapshot: computed({
          get: () => dependentSettingsSnapshots[pluginId] ?? "{}",
          set: (value) => {
            dependentSettingsSnapshots[pluginId] = value;
          },
        }),
        enabled: () => !isInitializingForms.value,
        save: async (payload) => {
          await pluginStore.setPluginSettings(pluginId, payload);
        },
      }),
    );
  }
}

async function flushPendingAutoSaves() {
  await mainAutoSave.flush();
  await Promise.all(Array.from(dependentAutoSaves.values()).map((entry) => entry.flush()));
}

async function loadPluginData() {
  loading.value = true;
  await flushPendingAutoSaves();
  isInitializingForms.value = true;

  clearRecord(settingsForm);
  clearRecord(dependentSettingsForms);
  clearRecord(dependentSettingsSnapshots);
  dependentPlugins.value = [];

  try {
    const found = pluginStore.plugins.find((item) => item.manifest.id === props.pluginId) ?? null;
    plugin.value = found;
    if (!found) {
      return;
    }

    const savedSettings = await pluginStore.getPluginSettings(props.pluginId);
    Object.assign(settingsForm, savedSettings || {});
    for (const field of found.manifest.settings || []) {
      if (settingsForm[field.key] === undefined) {
        settingsForm[field.key] = field.default ?? getDefaultValue(field.type);
      }
    }

    if (showDependents.value) {
      await loadDependentPlugins();
    }
  } finally {
    syncDependentAutoSaves();
    mainSettingsSnapshot.value = serializeSettings({ ...settingsForm });
    isInitializingForms.value = false;
    loading.value = false;
  }
}

async function applyPreset(presetKey: string) {
  const presets = pluginPresets.value;
  const pluginId = plugin.value?.manifest.id;
  if (!presets || !presets[presetKey] || !pluginId) {
    return;
  }

  const presetData = presets[presetKey];
  const settingsToSave: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(presetData)) {
    if (key === "name") {
      continue;
    }
    settingsForm[key] = value;
    settingsToSave[key] = value;
  }

  settingsForm.preset = presetKey;
  settingsToSave.preset = presetKey;

  await pluginStore.setPluginSettings(pluginId, settingsToSave);
  mainSettingsSnapshot.value = serializeSettings({ ...settingsForm });
}

function resetToDefault() {
  if (!plugin.value?.manifest.settings) {
    return;
  }
  for (const field of plugin.value.manifest.settings) {
    settingsForm[field.key] = field.default ?? getDefaultValue(field.type);
  }
}

onMounted(() => {
  if (pluginStore.plugins.length > 0) {
    void loadPluginData();
  }
});

watch(
  () => pluginStore.plugins,
  (newPlugins) => {
    if (newPlugins.length > 0 && !plugin.value) {
      void loadPluginData();
    }
  },
  { deep: false },
);

watch(
  () => props.pluginId,
  () => {
    void loadPluginData();
  },
);

onBeforeRouteLeave(async () => {
  await flushPendingAutoSaves();
});

onBeforeRouteUpdate(async () => {
  await flushPendingAutoSaves();
});

onUnmounted(() => {
  for (const autoSave of dependentAutoSaves.values()) {
    autoSave.stop();
  }
  dependentAutoSaves.clear();
});
</script>

<template>
  <div class="category-view">
    <div v-if="loading" class="loading-state">
      <div class="loading-spinner"></div>
      <span>{{ i18n.t("common.loading") }}</span>
    </div>

    <template v-else-if="plugin">
      <header class="category-header">
        <div class="header-icon">
          <img
            v-if="pluginStore.icons[plugin.manifest.id]"
            :src="pluginStore.icons[plugin.manifest.id]"
            :alt="plugin.manifest.name"
          />
          <Palette v-else :size="24" />
        </div>
        <div class="header-info">
          <h1>{{ categoryLabel }}</h1>
          <p class="header-desc">
            {{ getLocalizedPluginDescription(plugin.manifest, i18n.getLocale()) }}
          </p>
        </div>
      </header>

      <SLCard v-if="isThemeProvider && pluginPresets" class="settings-card">
        <h3 class="section-title">{{ i18n.t("plugins.preset_theme") }}</h3>
        <div class="presets-grid">
          <button
            v-for="(presetData, presetKey) in pluginPresets"
            :key="presetKey"
            class="preset-btn"
            :class="{ active: settingsForm['preset'] === String(presetKey) }"
            @click="applyPreset(String(presetKey))"
          >
            <div class="preset-swatches">
              <span
                class="preset-swatch"
                :style="{ background: (presetData as any).accent_primary }"
              ></span>
              <span
                class="preset-swatch"
                :style="{ background: (presetData as any).accent_secondary }"
              ></span>
            </div>
            <span class="preset-name">{{ (presetData as any).name ?? presetKey }}</span>
          </button>
        </div>
      </SLCard>

      <PluginSettingsFormCard
        v-if="plugin.manifest.settings?.length"
        :title="`${plugin.manifest.name} ${i18n.t('plugins.settings')}`"
        :fields="plugin.manifest.settings"
        :form="settingsForm"
        :get-field-string-value="getFieldStringValue"
        :get-field-select-value="getFieldSelectValue"
        :get-field-options="getFieldOptions"
        @update-field="updateSettingsField(settingsForm, $event[0], $event[1])"
      />

      <template v-if="showDependents && dependentPlugins.length > 0">
        <div class="dependent-section-header">
          <h2>{{ i18n.t("plugins.related_plugins") }}</h2>
          <p>{{ i18n.t("plugins.related_plugins_desc", { name: plugin.manifest.name }) }}</p>
        </div>

        <PluginSettingsFormCard
          v-for="depPlugin in dependentPlugins"
          :key="depPlugin.manifest.id"
          dependent
          :title="`${depPlugin.manifest.name} ${i18n.t('plugins.settings')}`"
          :description="i18n.t('plugins.related_plugins_desc', { name: plugin.manifest.name })"
          :fields="depPlugin.manifest.settings || []"
          :form="dependentSettingsForms[depPlugin.manifest.id]"
          :get-field-string-value="getFieldStringValue"
          :get-field-select-value="getFieldSelectValue"
          :get-field-options="getFieldOptions"
          @update-field="
            updateSettingsField(dependentSettingsForms[depPlugin.manifest.id], $event[0], $event[1])
          "
        />
      </template>

      <div class="action-buttons">
        <SLButton variant="secondary" @click="resetToDefault">{{
          i18n.t("plugins.reset_default")
        }}</SLButton>
        <span class="auto-save-hint">{{
          saving ? i18n.t("plugins.saving") : i18n.t("plugins.auto_saved")
        }}</span>
      </div>
    </template>

    <div v-else class="not-found">
      <p>{{ i18n.t("plugins.not_found") }}</p>
    </div>
  </div>
</template>

<style scoped>
.category-view {
  padding: 24px;
  max-width: 900px;
  margin: 0 auto;
  font-family: var(--font-sans);
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  color: var(--sl-text-secondary);
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 12px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.category-header {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 24px;
}

.header-icon {
  width: 64px;
  height: 64px;
  border-radius: var(--radius-lg);
  overflow: hidden;
  background: var(--bg-tertiary);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.header-icon img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.header-icon svg {
  width: 32px;
  height: 32px;
  color: var(--sl-text-secondary);
}

.header-info h1 {
  font-size: var(--sl-font-size-3xl);
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 4px 0;
}

.header-desc {
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-base);
  margin: 0;
}

.dependent-section-header {
  margin: 32px 0 16px 0;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border);
}

.dependent-section-header h2 {
  font-size: var(--sl-font-size-xl);
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 4px 0;
}

.dependent-section-header p {
  font-size: var(--sl-font-size-base);
  color: var(--sl-text-secondary);
  margin: 0;
}

.action-buttons {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 12px;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--border);
}

.auto-save-hint {
  font-size: var(--sl-font-size-sm);
  color: var(--sl-text-secondary);
  opacity: 0.7;
}

.not-found {
  text-align: center;
  padding: 48px;
  color: var(--sl-text-secondary);
}

.presets-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.preset-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: var(--sl-radius-md, 10px);
  cursor: pointer;
  transition: all 0.2s;
  color: var(--sl-text-primary);
  min-width: 80px;
}

.preset-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.15);
}

.preset-btn.active {
  border-color: var(--sl-accent, #60a5fa);
  background: rgba(96, 165, 250, 0.1);
}

.preset-swatches {
  display: flex;
  gap: 4px;
}

.preset-swatch {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  display: inline-block;
}

.preset-name {
  font-size: var(--sl-font-size-xs);
}
</style>
