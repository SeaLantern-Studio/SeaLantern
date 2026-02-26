<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from "vue";
import { useRouter } from "vue-router";
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLInput from "@components/common/SLInput.vue";
import SLSelect from "@components/common/SLSelect.vue";
import { usePluginStore } from "@stores/pluginStore";
import { i18n } from "@language";
import type { PluginInfo } from "@type/plugin";
import { getLocalizedPluginName, getLocalizedPluginDescription } from "@type/plugin";
import { ArrowLeft, Puzzle, Link } from "lucide-vue-next";

const props = defineProps<{
  pluginId: string;
}>();

const router = useRouter();
const pluginStore = usePluginStore();

const plugin = ref<PluginInfo | null>(null);
const settingsForm = reactive<Record<string, string | number | boolean | null>>({});
const saving = ref(false);
const loading = ref(true);

const dependentPlugins = ref<PluginInfo[]>([]);
const dependentSettingsForms = reactive<Record<string, Record<string, any>>>({});

const pluginPresets = computed(() => {
  return plugin.value?.manifest.presets ?? null;
});

const isThemeProvider = computed(() => {
  return plugin.value?.manifest.capabilities?.includes("theme-provider") ?? false;
});

async function loadPlugin() {
  loading.value = true;
  try {
    if (!pluginStore.plugins.length) {
      await pluginStore.loadPlugins();
    }
    plugin.value = pluginStore.plugins.find((p) => p.manifest.id === props.pluginId) || null;

    if (plugin.value) {
      const savedSettings = await pluginStore.getPluginSettings(props.pluginId);
      Object.keys(settingsForm).forEach((key) => delete settingsForm[key]);
      if (plugin.value.manifest.settings) {
        for (const field of plugin.value.manifest.settings) {
          settingsForm[field.key] =
            savedSettings[field.key] ?? field.default ?? getDefaultValue(field.type);
        }
      }

      await loadDependentPlugins();
    }
  } finally {
    loading.value = false;
  }
}

async function loadDependentPlugins() {
  dependentPlugins.value = [];

  Object.keys(dependentSettingsForms).forEach((key) => delete dependentSettingsForms[key]);

  const candidates = pluginStore.plugins.filter((p) => {
    if (p.state !== "enabled") return false;
    if (p.manifest.id === props.pluginId) return false;
    const deps = p.manifest.dependencies || [];
    return deps.some((dep: string | { id: string }) => {
      const depId = typeof dep === "string" ? dep : dep.id;
      return depId === props.pluginId;
    });
  });

  const settingsPromises = candidates
    .filter((p) => p.manifest.settings?.length)
    .map(async (p) => {
      const depSettings = await pluginStore.getPluginSettings(p.manifest.id);
      const form: Record<string, any> = {};
      for (const field of p.manifest.settings!) {
        form[field.key] = depSettings[field.key] ?? field.default ?? getDefaultValue(field.type);
      }
      return { plugin: p, form };
    });

  const results = await Promise.all(settingsPromises);
  for (const { plugin: depPlugin, form } of results) {
    dependentPlugins.value.push(depPlugin);
    dependentSettingsForms[depPlugin.manifest.id] = form;
  }
}

function getDefaultValue(type: string): any {
  switch (type) {
    case "boolean":
      return false;
    case "number":
      return 0;
    case "select":
      return "";
    default:
      return "";
  }
}

async function applyPreset(presetKey: string) {
  const presets = pluginPresets.value;
  if (!presets || !presets[presetKey]) return;
  const presetData = presets[presetKey];
  const pluginId = plugin.value?.manifest.id;
  if (!pluginId) return;

  const settingsToSave: Record<string, any> = {};
  for (const [key, value] of Object.entries(presetData)) {
    if (key !== "name") {
      settingsForm[key] = value as string | number | boolean;
      settingsToSave[key] = value;
    }
  }
  await pluginStore.setPluginSettings(pluginId, settingsToSave);
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

    const depPromises = dependentPlugins.value.map(async (depPlugin) => {
      const depForm = dependentSettingsForms[depPlugin.manifest.id];
      if (depForm) {
        await pluginStore.setPluginSettings(depPlugin.manifest.id, { ...depForm });
        if (pluginStore.hasCapability(depPlugin.manifest.id, "theme-widgets-provider")) {
          await pluginStore.applyThemeWidgetsProviderSettings(depPlugin.manifest.id);
        }
      }
    });
    await Promise.all(depPromises);
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

function goBack() {
  router.back();
}

onMounted(() => {
  loadPlugin();
});

watch(
  () => props.pluginId,
  () => {
    loadPlugin();
  },
);
</script>

<template>
  <div class="plugin-page-view">
    <div class="page-header">
      <SLButton variant="ghost" @click="goBack">
        <ArrowLeft :size="20" />
        <span>{{ i18n.t("plugins.back") }}</span>
      </SLButton>
      <h1 class="page-title" v-if="plugin">{{ plugin.manifest.name }}</h1>
    </div>

    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <span>{{ i18n.t("common.loading") }}</span>
    </div>

    <div v-else-if="!plugin" class="empty-state">
      <p>{{ i18n.t("plugins.not_found") }}</p>
    </div>

    <div v-else class="plugin-content">
      <SLCard class="info-card">
        <div class="plugin-info">
          <div class="plugin-icon" v-if="pluginStore.icons[plugin.manifest.id]">
            <img :src="pluginStore.icons[plugin.manifest.id]" :alt="plugin.manifest.name" />
          </div>
          <div class="plugin-icon placeholder" v-else>
            <Puzzle :size="32" :stroke-width="1.5" />
          </div>
          <div class="plugin-details">
            <h2>
              {{ getLocalizedPluginName(plugin.manifest, i18n.getLocale()) }}
            </h2>
            <p class="description">
              {{ getLocalizedPluginDescription(plugin.manifest, i18n.getLocale()) }}
            </p>
            <div class="meta">
              <span class="version">v{{ plugin.manifest.version }}</span>
              <span class="author">{{ plugin.manifest.author.name }}</span>
            </div>
          </div>
        </div>
      </SLCard>

      <SLCard v-if="isThemeProvider && pluginPresets" class="presets-card">
        <h3 class="section-title">{{ i18n.t("plugins.preset_theme") }}</h3>
        <div class="presets-grid">
          <SLButton
            v-for="(presetData, presetKey) in pluginPresets"
            :key="presetKey"
            variant="secondary"
            class="preset-btn"
            @click="applyPreset(String(presetKey))"
          >
            <span class="preset-name">{{ (presetData as any).name ?? presetKey }}</span>
          </SLButton>
        </div>
      </SLCard>

      <template v-if="plugin.manifest.settings?.length">
        <SLCard class="settings-card">
          <h3 class="section-title">{{ i18n.t("plugins.plugin_settings") }}</h3>
          <div class="settings-form">
            <div v-for="field in plugin.manifest.settings" :key="field.key" class="form-field">
              <label class="field-label">
                {{ field.label }}
                <span v-if="field.description" class="field-desc">{{ field.description }}</span>
              </label>
              <template v-if="field.type === 'string'">
                <SLInput v-model="settingsForm[field.key]" />
              </template>
              <template v-else-if="field.type === 'number'">
                <SLInput type="number" v-model="settingsForm[field.key]" />
              </template>
              <template v-else-if="field.type === 'boolean'">
                <SLSwitch
                  :modelValue="Boolean(settingsForm[field.key])"
                  @update:modelValue="settingsForm[field.key] = $event"
                  size="sm"
                />
              </template>
              <template v-else-if="field.type === 'select'">
                <SLSelect v-model="settingsForm[field.key]" :options="field.options" />
              </template>
            </div>
          </div>
        </SLCard>
      </template>

      <template v-if="dependentPlugins.length > 0">
        <SLCard
          v-for="depPlugin in dependentPlugins"
          :key="depPlugin.manifest.id"
          class="settings-card dependent-settings"
        >
          <h3 class="section-title">
            <Link class="dependent-icon" :size="16" />
            {{ depPlugin.manifest.name }} {{ i18n.t("plugins.settings") }}
          </h3>
          <p class="dependent-desc">
            {{ i18n.t("plugins.depends_on", { name: plugin?.manifest.name }) }}
          </p>
          <div class="settings-form">
            <div v-for="field in depPlugin.manifest.settings" :key="field.key" class="form-field">
              <label class="field-label">
                {{ field.label }}
                <span v-if="field.description" class="field-desc">{{ field.description }}</span>
              </label>
              <template v-if="field.type === 'string'">
                <SLInput v-model="dependentSettingsForms[depPlugin.manifest.id][field.key]" />
              </template>
              <template v-else-if="field.type === 'number'">
                <SLInput
                  type="number"
                  v-model="dependentSettingsForms[depPlugin.manifest.id][field.key]"
                />
              </template>
              <template v-else-if="field.type === 'boolean'">
                <SLSwitch
                  :modelValue="Boolean(dependentSettingsForms[depPlugin.manifest.id][field.key])"
                  @update:modelValue="
                    dependentSettingsForms[depPlugin.manifest.id][field.key] = $event
                  "
                  size="sm"
                />
              </template>
              <template v-else-if="field.type === 'select'">
                <SLSelect
                  v-model="dependentSettingsForms[depPlugin.manifest.id][field.key]"
                  :options="field.options"
                />
              </template>
            </div>
          </div>
        </SLCard>
      </template>

      <div class="action-buttons">
        <SLButton variant="secondary" @click="resetToDefault">{{
          i18n.t("plugins.reset_default")
        }}</SLButton>
        <SLButton variant="primary" :loading="saving" @click="saveSettings">{{
          i18n.t("plugins.save_settings")
        }}</SLButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.plugin-page-view {
  padding: 24px;
  max-width: 900px;
  margin: 0 auto;
  font-family: var(--sl-font-sans);
}

.page-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 24px;
}

.back-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.back-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.page-title {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px;
  color: var(--text-secondary);
  gap: 16px;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-color);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.plugin-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.info-card {
  padding: 20px;
}

.plugin-info {
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

.plugin-icon {
  width: 64px;
  height: 64px;
  border-radius: var(--radius-md);
  overflow: hidden;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.plugin-icon img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.plugin-icon.placeholder {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

.plugin-details h2 {
  margin: 0 0 8px;
  font-size: 1.25rem;
  color: var(--text-primary);
}

.plugin-details .description {
  margin: 0 0 12px;
  color: var(--text-secondary);
  font-size: 0.9rem;
  line-height: 1.5;
}

.plugin-details .meta {
  display: flex;
  gap: 12px;
  font-size: 0.8rem;
}

.plugin-details .version {
  color: var(--accent-primary);
  background: rgba(96, 165, 250, 0.1);
  padding: 2px 8px;
  border-radius: var(--radius-sm);
}

.plugin-details .author {
  color: var(--text-secondary);
}

.section-title {
  margin: 0 0 16px;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
}

.presets-card {
  padding: 20px;
}

.presets-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 12px;
}

.preset-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background: var(--bg-secondary);
  border: 2px solid var(--border-color);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s;
}

.preset-btn:hover {
  border-color: var(--accent-primary);
  background: var(--bg-tertiary);
}

.preset-btn.active {
  border-color: var(--accent-primary);
  background: rgba(96, 165, 250, 0.1);
}

.preset-colors {
  display: flex;
  gap: 4px;
}

.color-dot {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.preset-name {
  font-size: 0.8rem;
  color: var(--text-primary);
}

.colors-card {
  padding: 20px;
}

.color-settings {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.color-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
}

.color-label {
  font-size: 0.9rem;
  color: var(--text-primary);
}

.color-inputs {
  display: flex;
  align-items: center;
  gap: 8px;
}

.color-text {
  width: 140px;
  padding: 6px 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-family: monospace;
  font-size: 0.85rem;
}

.color-text:focus {
  outline: none;
  border-color: var(--accent-primary);
}

.color-picker-wrapper {
  position: relative;
  width: 32px;
  height: 32px;
}

.color-picker {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: pointer;
}

.color-preview {
  width: 100%;
  height: 100%;
  border-radius: var(--radius-sm);
  border: 2px solid var(--border-color);
  pointer-events: none;
}

.effects-card {
  padding: 20px;
}

.effect-settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.effect-row {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.effect-label {
  font-size: 0.9rem;
  color: var(--text-primary);
}

.effect-options {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.effect-btn {
  padding: 6px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: 0.85rem;
  cursor: pointer;
  transition: all 0.2s;
}

.effect-btn:hover {
  border-color: var(--accent-primary);
  color: var(--text-primary);
}

.effect-btn.active {
  background: var(--accent-primary);
  border-color: var(--accent-primary);
  color: white;
}

.settings-card {
  padding: 20px;
}

.settings-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 0.9rem;
  color: var(--text-primary);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.field-desc {
  font-size: 0.8rem;
  color: var(--text-secondary);
  font-weight: normal;
}

.field-input,
.field-select {
  padding: 10px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 0.9rem;
}

.field-input:focus,
.field-select:focus {
  outline: none;
  border-color: var(--accent-primary);
}

.import-export-card {
  padding: 20px;
}

.import-export-actions {
  display: flex;
  gap: 12px;
}

.ie-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 0.9rem;
  cursor: pointer;
  transition: all 0.2s;
}

.ie-btn:hover {
  background: var(--bg-tertiary);
  border-color: var(--accent-primary);
}

.ie-btn svg {
  flex-shrink: 0;
}

.dependent-settings {
  border-left: 3px solid var(--accent-secondary);
}

.dependent-settings .section-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.dependent-icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  color: var(--accent-secondary);
}

.dependent-desc {
  font-size: 0.85rem;
  color: var(--text-secondary);
  margin: -8px 0 16px 0;
}

.dependent-settings .field-select {
  width: 100%;
  padding: 10px 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 0.9rem;
  cursor: pointer;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 24 24' fill='none' stroke='%2394a3b8' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
  padding-right: 36px;
}

.dependent-settings .field-select:hover {
  border-color: var(--accent-primary);
}

.dependent-settings .field-select:focus {
  outline: none;
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 2px rgba(96, 165, 250, 0.2);
}

.dependent-settings .field-select option {
  background: var(--bg-secondary);
  color: var(--text-primary);
  padding: 8px;
}

.action-buttons {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding-top: 8px;
}
</style>
