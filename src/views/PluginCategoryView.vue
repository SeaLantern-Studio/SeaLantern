<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from "vue";
import { useRoute } from "vue-router";
import SLCard from "../components/common/SLCard.vue";
import SLButton from "../components/common/SLButton.vue";
import SLFormField from "../components/common/SLFormField.vue";
import SLInput from "../components/common/SLInput.vue";
import SLSwitch from "../components/common/SLSwitch.vue";
import SLSelect from "../components/common/SLSelect.vue";
import SLTextarea from "../components/common/SLTextarea.vue";
import SLCheckbox from "../components/common/SLCheckbox.vue";
import { usePluginStore } from "../stores/pluginStore";
import { i18n } from "../language";
import type { PluginInfo, PluginSettingField } from "../types/plugin";
import { getLocalizedPluginName, getLocalizedPluginDescription } from "../types/plugin";
import { Palette, Puzzle } from "lucide-vue-next";

const props = defineProps<{
  pluginId: string;
}>();

const route = useRoute();
const pluginStore = usePluginStore();

const plugin = ref<PluginInfo | null>(null);
const settingsForm = reactive<Record<string, any>>({});
const saving = ref(false);
const loading = ref(true);

const dependentPlugins = ref<PluginInfo[]>([]);
const dependentSettingsForms = reactive<Record<string, Record<string, any>>>({});

const sidebarConfig = computed(() => {
  return plugin.value?.manifest.sidebar;
});

const categoryLabel = computed(() => {
  return sidebarConfig.value?.label || plugin.value?.manifest.name || "";
});

const showDependents = computed(() => {
  return sidebarConfig.value?.show_dependents !== false;
});

function getDependencyId(dep: string | { id: string; version?: string }): string {
  return typeof dep === "string" ? dep : dep.id;
}

async function loadPluginData() {
  loading.value = true;
  const found = pluginStore.plugins.find((p) => p.manifest.id === props.pluginId);
  if (found) {
    plugin.value = found;

    const settings = await pluginStore.getPluginSettings(props.pluginId);
    Object.assign(settingsForm, settings || {});

    if (found.manifest.settings) {
      for (const field of found.manifest.settings) {
        if (settingsForm[field.key] === undefined) {
          settingsForm[field.key] = field.default ?? getDefaultValue(field.type);
        }
      }
    }

    if (showDependents.value) {
      await loadDependentPlugins();
    }
  }
  loading.value = false;
}

async function loadDependentPlugins() {
  if (!plugin.value) return;

  const deps: PluginInfo[] = [];
  for (const p of pluginStore.plugins) {
    if (p.state !== "enabled") continue;
    if (p.manifest.id === props.pluginId) continue;

    const allDeps = [
      ...(p.manifest.dependencies || []),
      ...(p.manifest.optional_dependencies || []),
    ];
    const dependsOnCurrent = allDeps.some((dep) => getDependencyId(dep) === props.pluginId);

    if (dependsOnCurrent && p.manifest.settings?.length) {
      deps.push(p);

      const depSettings = await pluginStore.getPluginSettings(p.manifest.id);
      dependentSettingsForms[p.manifest.id] = { ...depSettings };

      for (const field of p.manifest.settings) {
        if (dependentSettingsForms[p.manifest.id][field.key] === undefined) {
          dependentSettingsForms[p.manifest.id][field.key] =
            field.default ?? getDefaultValue(field.type);
        }
      }
    }
  }
  dependentPlugins.value = deps;
}

function getDefaultValue(type: string): any {
  switch (type) {
    case "string":
      return "";
    case "textarea":
      return "";
    case "number":
      return 0;
    case "boolean":
      return false;
    case "checkbox":
      return false;
    case "select":
      return "";
    default:
      return "";
  }
}

const pluginPresets = computed(() => {
  return plugin.value?.manifest.presets ?? null;
});

const isThemeProvider = computed(() => {
  return plugin.value?.manifest.capabilities?.includes("theme-provider") ?? false;
});

async function applyPreset(presetKey: string) {
  const presets = pluginPresets.value;
  if (!presets || !presets[presetKey]) return;
  const presetData = presets[presetKey];
  const pluginId = plugin.value?.manifest.id;
  if (!pluginId) return;
  for (const [key, value] of Object.entries(presetData)) {
    if (key !== "name") {
      settingsForm[key] = value;
      await pluginStore.setPluginSettings(pluginId, { [key]: value });
    }
  }

  settingsForm["preset"] = presetKey;
  await pluginStore.setPluginSettings(pluginId, { preset: presetKey });
  await pluginStore.applyThemeProviderSettings(pluginId);
}

async function saveSettings() {
  if (!plugin.value) return;
  saving.value = true;
  try {
    await pluginStore.setPluginSettings(props.pluginId, { ...settingsForm });
    if (isThemeProvider.value) {
      await pluginStore.applyThemeProviderSettings(props.pluginId);
    }

    for (const depPlugin of dependentPlugins.value) {
      const depForm = dependentSettingsForms[depPlugin.manifest.id];
      if (depForm) {
        await pluginStore.setPluginSettings(depPlugin.manifest.id, {
          ...depForm,
        });
        if (pluginStore.hasCapability(depPlugin.manifest.id, "theme-widgets-provider")) {
          await pluginStore.applyThemeWidgetsProviderSettings(depPlugin.manifest.id);
        }
      }
    }
  } finally {
    saving.value = false;
  }
}

async function resetToDefault() {
  if (!plugin.value?.manifest.settings) return;
  for (const field of plugin.value.manifest.settings) {
    settingsForm[field.key] = field.default ?? getDefaultValue(field.type);
  }
}

onMounted(() => {
  if (pluginStore.plugins.length > 0) {
    loadPluginData();
  }
});

watch(
  () => pluginStore.plugins,
  (newPlugins) => {
    if (newPlugins.length > 0 && !plugin.value) {
      loadPluginData();
    }
  },
  { deep: false },
);

watch(
  () => props.pluginId,
  () => {
    loadPluginData();
  },
);

let autoSaveTimer: ReturnType<typeof setTimeout> | null = null;
watch(
  settingsForm,
  async () => {
    if (!plugin.value) return;
    if (autoSaveTimer) clearTimeout(autoSaveTimer);
    autoSaveTimer = setTimeout(async () => {
      await pluginStore.setPluginSettings(props.pluginId, { ...settingsForm });
      if (isThemeProvider.value) {
        await pluginStore.applyThemeProviderSettings(props.pluginId);
      }
    }, 300);
  },
  { deep: true },
);
</script>

<template>
  <div class="category-view">
    <div v-if="loading" class="loading-state">
      <div class="loading-spinner"></div>
      <span>加载中...</span>
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
        <h3 class="section-title">预设主题</h3>
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

      <SLCard v-if="plugin.manifest.settings?.length" class="settings-card main-settings">
        <h3 class="section-title">{{ plugin.manifest.name }} 设置</h3>
        <div class="settings-form">
          <SLFormField
            v-for="field in plugin.manifest.settings"
            :key="field.key"
            :label="field.label"
            :hint="field.description"
          >
            <template v-if="field.type === 'string'">
              <SLInput v-model="settingsForm[field.key]" />
            </template>
            <template v-else-if="field.type === 'color'">
              <div class="color-row-inline">
                <span class="color-row-value">{{ settingsForm[field.key] }}</span>
                <input type="color" v-model="settingsForm[field.key]" class="color-row-picker" />
              </div>
            </template>
            <template v-else-if="field.type === 'textarea'">
              <SLTextarea
                v-model="settingsForm[field.key]"
                :rows="field.rows"
                :maxlength="field.maxlength"
              />
            </template>
            <template v-else-if="field.type === 'number'">
              <SLInput
                type="number"
                :model-value="String(settingsForm[field.key])"
                @update:model-value="settingsForm[field.key] = Number($event)"
              />
            </template>
            <template v-else-if="field.type === 'boolean'">
              <SLSwitch v-model="settingsForm[field.key]" />
            </template>
            <template v-else-if="field.type === 'checkbox'">
              <SLCheckbox v-model="settingsForm[field.key]" />
            </template>
            <template v-else-if="field.type === 'select' && field.display === 'button-group'">
              <div class="btn-group">
                <button
                  v-for="opt in field.options"
                  :key="opt.value"
                  class="btn-group-item"
                  :class="{ active: settingsForm[field.key] === opt.value }"
                  @click="settingsForm[field.key] = opt.value"
                >
                  {{ opt.label }}
                </button>
              </div>
            </template>
            <template v-else-if="field.type === 'select'">
              <SLSelect v-model="settingsForm[field.key]" :options="field.options || []" />
            </template>
          </SLFormField>
        </div>
      </SLCard>

      <template v-if="showDependents && dependentPlugins.length > 0">
        <div class="dependent-section-header">
          <h2>关联插件</h2>
          <p>以下插件依赖 {{ plugin.manifest.name }}</p>
        </div>

        <SLCard
          v-for="depPlugin in dependentPlugins"
          :key="depPlugin.manifest.id"
          class="settings-card dependent-settings"
        >
          <div class="dependent-header">
            <img
              v-if="pluginStore.icons[depPlugin.manifest.id]"
              :src="pluginStore.icons[depPlugin.manifest.id]"
              class="dependent-icon"
              :alt="depPlugin.manifest.name"
            />
            <Puzzle v-else class="dependent-icon" :size="20" />
            <div class="dependent-info">
              <h3>{{ depPlugin.manifest.name }}</h3>
              <span class="dependent-version">v{{ depPlugin.manifest.version }}</span>
            </div>
          </div>
          <div class="settings-form">
            <SLFormField
              v-for="field in depPlugin.manifest.settings"
              :key="field.key"
              :label="field.label"
              :hint="field.description"
            >
              <template v-if="field.type === 'string'">
                <SLInput v-model="dependentSettingsForms[depPlugin.manifest.id][field.key]" />
              </template>
              <template v-else-if="field.type === 'color'">
                <div class="color-row-inline">
                  <span class="color-row-value">{{
                    dependentSettingsForms[depPlugin.manifest.id][field.key]
                  }}</span>
                  <input
                    type="color"
                    v-model="dependentSettingsForms[depPlugin.manifest.id][field.key]"
                    class="color-row-picker"
                  />
                </div>
              </template>
              <template v-else-if="field.type === 'textarea'">
                <SLTextarea
                  v-model="dependentSettingsForms[depPlugin.manifest.id][field.key]"
                  :rows="field.rows"
                  :maxlength="field.maxlength"
                />
              </template>
              <template v-else-if="field.type === 'number'">
                <SLInput
                  type="number"
                  :model-value="String(dependentSettingsForms[depPlugin.manifest.id][field.key])"
                  @update:model-value="
                    dependentSettingsForms[depPlugin.manifest.id][field.key] = Number($event)
                  "
                />
              </template>
              <template v-else-if="field.type === 'boolean'">
                <SLSwitch v-model="dependentSettingsForms[depPlugin.manifest.id][field.key]" />
              </template>
              <template v-else-if="field.type === 'checkbox'">
                <SLCheckbox v-model="dependentSettingsForms[depPlugin.manifest.id][field.key]" />
              </template>
              <template v-else-if="field.type === 'select' && field.display === 'button-group'">
                <div class="btn-group">
                  <button
                    v-for="opt in field.options"
                    :key="opt.value"
                    class="btn-group-item"
                    :class="{
                      active:
                        dependentSettingsForms[depPlugin.manifest.id][field.key] === opt.value,
                    }"
                    @click="dependentSettingsForms[depPlugin.manifest.id][field.key] = opt.value"
                  >
                    {{ opt.label }}
                  </button>
                </div>
              </template>
              <template v-else-if="field.type === 'select'">
                <SLSelect
                  v-model="dependentSettingsForms[depPlugin.manifest.id][field.key]"
                  :options="field.options || []"
                />
              </template>
            </SLFormField>
          </div>
        </SLCard>
      </template>

      <div class="action-buttons">
        <SLButton variant="secondary" @click="resetToDefault">重置为默认</SLButton>
        <span class="auto-save-hint">{{ saving ? "保存中..." : "已自动保存" }}</span>
      </div>
    </template>

    <div v-else class="not-found">
      <p>未找到插件</p>
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
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 4px 0;
}

.header-desc {
  color: var(--sl-text-secondary);
  font-size: 0.875rem;
  margin: 0;
}

.settings-card {
  margin-bottom: 16px;
}

.section-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 16px 0;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border);
}

.settings-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.color-row-inline {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
}

.color-row-value {
  font-size: 13px;
  color: var(--sl-text-secondary);
  font-family: monospace;
}

.color-row-picker {
  width: 36px;
  height: 36px;
  padding: 2px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  flex-shrink: 0;
}

.dependent-section-header {
  margin: 32px 0 16px 0;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border);
}

.dependent-section-header h2 {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 4px 0;
}

.dependent-section-header p {
  font-size: 0.875rem;
  color: var(--sl-text-secondary);
  margin: 0;
}

.dependent-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border);
}

.dependent-icon {
  width: 32px;
  height: 32px;
  border-radius: var(--sl-radius-sm);
  object-fit: cover;
}

.dependent-icon svg {
  color: var(--sl-text-secondary);
}

.dependent-info h3 {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0;
}

.dependent-version {
  font-size: 0.75rem;
  color: var(--sl-text-secondary);
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
  font-size: 13px;
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
  font-size: 12px;
}

.color-rows {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.color-row {
  display: flex;
  align-items: center;
  padding: 10px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  gap: 12px;
}

.color-row:last-child {
  border-bottom: none;
}

.color-row-label {
  flex: 1;
  font-size: 14px;
  color: var(--sl-text-primary);
}

.color-row-value {
  font-size: 13px;
  color: var(--sl-text-secondary);
  font-family: monospace;
  min-width: 80px;
  text-align: right;
}

.color-row-picker {
  width: 36px;
  height: 36px;
  padding: 2px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  flex-shrink: 0;
}

.effect-rows {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.effect-row {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.effect-row-label {
  font-size: 14px;
  color: var(--sl-text-primary);
}

.btn-group {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.btn-group-item {
  padding: 6px 14px;
  font-size: 13px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: var(--sl-radius-md, 8px);
  color: var(--sl-text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-group-item:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--sl-text-primary);
}

.btn-group-item.active {
  background: rgba(96, 165, 250, 0.15);
  border-color: var(--sl-accent, #60a5fa);
  color: var(--sl-accent, #60a5fa);
}
</style>
