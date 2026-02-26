<script setup lang="ts">
import { onMounted, onUnmounted, ref, reactive, computed } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useRouter } from "vue-router";
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import SLModal from "@components/common/SLModal.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLCheckbox from "@components/common/SLCheckbox.vue";
import SLInput from "@components/common/SLInput.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLMenu from "@components/common/SLMenu.vue";
import PluginPermissionPanel from "@components/plugin/PluginPermissionPanel.vue";
import SLPermissionDialog from "@components/plugin/SLPermissionDialog.vue";
import { usePluginStore } from "@stores/pluginStore";
import { i18n } from "@language";
import type { PluginState, PluginInfo, MissingDependency, BatchInstallResult } from "@type/plugin";
import {
  hasDangerousPermissions,
  getLocalizedPluginName,
  getLocalizedPluginDescription,
} from "@type/plugin";
import {
  Upload,
  Layers,
  ShieldAlert,
  MoreVertical,
  Github,
  Settings,
  X,
  Trash2,
  Trash,
  RefreshCw,
} from "lucide-vue-next";

const router = useRouter();
const pluginStore = usePluginStore();
const isDragging = ref(false);
const searchQuery = ref("");

const filteredPlugins = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return pluginStore.plugins;
  return pluginStore.plugins.filter((p) => {
    const id = p.manifest.id.toLowerCase();
    const name = getLocalizedPluginName(p.manifest, i18n.getLocale()).toLowerCase();
    const stateStr = (typeof p.state === "string" ? p.state : "error").toLowerCase();
    return id.includes(q) || name.includes(q) || stateStr.includes(q);
  });
});
const isInstalling = ref(false);
let unlistenDragDrop: (() => void) | null = null;

const showDependencyModal = ref(false);
const missingDependencies = ref<MissingDependency[]>([]);
const installedPluginName = ref("");

const showSettingsModal = ref(false);
const currentSettingsPlugin = ref<PluginInfo | null>(null);
const settingsForm = reactive<Record<string, any>>({});
const savingSettings = ref(false);

const checkingUpdate = ref<string | null>(null);
const checkingAllUpdates = ref(false);

const batchMode = ref(false);
const selectedPlugins = ref<Set<string>>(new Set());

const showBatchDeleteDialog = ref(false);

const confirmDialog = ref<{
  show: boolean;
  title: string;
  message: string;
  onConfirm: () => void | Promise<void>;
}>({
  show: false,
  title: "",
  message: "",
  onConfirm: () => {},
});

const alertDialog = ref({
  show: false,
  title: "",
  message: "",
});

const showBatchResultModal = ref(false);
const batchInstallResult = ref<BatchInstallResult | null>(null);

const permissionWarning = ref({
  show: false,
  pluginId: "",
  pluginName: "",
  permissions: [] as string[],
});

function closeConfirmDialog() {
  confirmDialog.value.show = false;
}

function executeConfirmDialog() {
  const fn = confirmDialog.value.onConfirm;
  if (fn) Promise.resolve(fn()).catch((e) => console.error(e));
}

function showAlert(title: string, message: string) {
  alertDialog.value = {
    show: true,
    title,
    message,
  };
}

function closeAlertDialog() {
  alertDialog.value.show = false;
}

onMounted(async () => {
  if (pluginStore.plugins.length === 0 && !pluginStore.loading) {
    pluginStore.loadPlugins();
  }

  unlistenDragDrop = await getCurrentWebview().onDragDropEvent(async (event) => {
    if (event.payload.type === "over") {
      isDragging.value = true;
    } else if (event.payload.type === "drop") {
      isDragging.value = false;
      const paths = event.payload.paths;
      if (paths && paths.length > 0) {
        const validPaths = paths.filter(
          (p) => p.endsWith(".zip") || p.endsWith("manifest.json") || !p.includes("."),
        );
        if (validPaths.length > 0) {
          await handleBatchInstall(validPaths);
        }
      }
    } else {
      isDragging.value = false;
    }
  });
});

onUnmounted(() => {
  if (unlistenDragDrop) {
    unlistenDragDrop();
  }
});

async function handleInstall(filePath: string) {
  isInstalling.value = true;
  try {
    const result = await pluginStore.installFromZip(filePath);

    if (result.missing_dependencies && result.missing_dependencies.length > 0) {
      installedPluginName.value = result.plugin.manifest.name;
      missingDependencies.value = result.missing_dependencies;
      showDependencyModal.value = true;
    }
  } catch (e) {
    console.error("[Plugin] Install failed:", e);
  } finally {
    isInstalling.value = false;
  }
}

async function handleBatchInstall(paths: string[]) {
  if (paths.length === 1) {
    await handleInstall(paths[0]);
    return;
  }

  isInstalling.value = true;
  try {
    const result = await pluginStore.installBatch(paths);
    batchInstallResult.value = result;
    showBatchResultModal.value = true;
  } finally {
    isInstalling.value = false;
  }
}

async function handleSelectFile() {
  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "Plugin",
        extensions: ["zip", "json"],
      },
    ],
  });

  if (selected) {
    const paths = Array.isArray(selected) ? selected : [selected];
    if (paths.length > 0) {
      await handleBatchInstall(paths);
    }
  }
}

async function handleSelectFolder() {
  const selected = await open({
    directory: true,
    multiple: true,
  });

  if (selected) {
    const paths = Array.isArray(selected) ? selected : [selected];
    if (paths.length > 0) {
      await handleBatchInstall(paths);
    }
  }
}

function handleRefresh() {
  pluginStore.refreshPlugins();
}

async function handleToggle(id: string, currentEnabled: boolean) {
  pluginStore.error = null;

  if (!currentEnabled) {
    const plugin = pluginStore.plugins.find((p) => p.manifest.id === id);
    const permissions = plugin?.manifest.permissions || [];
    if (hasDangerousPermissions(permissions)) {
      permissionWarning.value = {
        show: true,
        pluginId: id,
        pluginName: plugin?.manifest.name || id,
        permissions: permissions,
      };
      return;
    }
  }

  await doTogglePlugin(id, !currentEnabled);
}

async function confirmPermissionWarning() {
  const { pluginId } = permissionWarning.value;
  permissionWarning.value.show = false;
  await doTogglePlugin(pluginId, true);
}

function cancelPermissionWarning() {
  permissionWarning.value.show = false;
}

async function doTogglePlugin(id: string, enable: boolean) {
  const result = await pluginStore.togglePlugin(id, enable);

  if (!result.success && result.error) {
    showAlert(i18n.t("plugins.enable_failed"), result.error);
  } else if (result.disabledPlugins && result.disabledPlugins.length > 0) {
    const plugin = pluginStore.plugins.find((p) => p.manifest.id === id);
    const pluginName = plugin?.manifest.name || id;
    const disabledNames = result.disabledPlugins.map((depId) => {
      const dep = pluginStore.plugins.find((p) => p.manifest.id === depId);
      return dep?.manifest.name || depId;
    });
    showAlert(
      i18n.t("plugins.plugin_disabled"),
      i18n.t("plugins.plugin_disabled_desc", { name: pluginName, deps: disabledNames.join(", ") }),
    );
  }
}

function getStatusColor(state: PluginState): string {
  if (typeof state === "object" && "error" in state) {
    return "var(--sl-error)";
  }
  switch (state) {
    case "enabled":
      return "var(--sl-success)";
    case "disabled":
      return "var(--sl-text-tertiary)";
    case "loaded":
      return "var(--sl-info)";
    default:
      return "var(--sl-text-secondary)";
  }
}

function getStatusLabel(state: PluginState): string {
  if (typeof state === "object" && "error" in state) {
    return i18n.t("plugins.status.error");
  }
  switch (state) {
    case "enabled":
      return i18n.t("plugins.status.enabled");
    case "disabled":
      return i18n.t("plugins.status.disabled");
    case "loaded":
      return i18n.t("plugins.status.loaded");
    default:
      return String(state);
  }
}

function getPermissionLabel(perm: string): string {
  return i18n.t(`plugins.permission.${perm}`) !== `plugins.permission.${perm}`
    ? i18n.t(`plugins.permission.${perm}`)
    : perm;
}

function getPermissionDesc(perm: string): string {
  return i18n.t(`plugins.permission.${perm}_desc`) !== `plugins.permission.${perm}_desc`
    ? i18n.t(`plugins.permission.${perm}_desc`)
    : "";
}

function isPluginEnabled(state: PluginState): boolean {
  return state === "enabled";
}

function hasSettings(plugin: PluginInfo): boolean {
  return !!(plugin.manifest.settings && plugin.manifest.settings.length > 0);
}

function hasMissingRequiredDependencies(plugin: PluginInfo): boolean {
  if (plugin.missing_dependencies) {
    const stillMissing = plugin.missing_dependencies.filter((d) => {
      if (!d.required) return false;
      const depPlugin = pluginStore.plugins.find((p) => p.manifest.id === d.id);
      return !depPlugin || depPlugin.state !== "enabled";
    });
    if (stillMissing.length > 0) return true;
  }

  if (plugin.manifest.dependencies && plugin.manifest.dependencies.length > 0) {
    for (const dep of plugin.manifest.dependencies) {
      const depId = typeof dep === "string" ? dep : dep.id;
      const depPlugin = pluginStore.plugins.find((p) => p.manifest.id === depId);
      if (!depPlugin || depPlugin.state !== "enabled") {
        return true;
      }
    }
  }
  return false;
}

function getMissingRequiredDependencies(plugin: PluginInfo): MissingDependency[] {
  const missing: MissingDependency[] = [];

  if (plugin.missing_dependencies) {
    for (const d of plugin.missing_dependencies.filter((d) => d.required)) {
      const depPlugin = pluginStore.plugins.find((p) => p.manifest.id === d.id);
      if (!depPlugin || depPlugin.state !== "enabled") {
        missing.push(d);
      }
    }
  }

  if (plugin.manifest.dependencies) {
    for (const dep of plugin.manifest.dependencies) {
      const depId = typeof dep === "string" ? dep : dep.id;

      if (missing.some((m) => m.id === depId)) continue;
      const depPlugin = pluginStore.plugins.find((p) => p.manifest.id === depId);
      if (!depPlugin || depPlugin.state !== "enabled") {
        missing.push({ id: depId, required: true });
      }
    }
  }
  return missing;
}

function getMissingOptionalDependencies(plugin: PluginInfo): MissingDependency[] {
  const missing: MissingDependency[] = [];

  if (plugin.missing_dependencies) {
    for (const d of plugin.missing_dependencies.filter((d) => !d.required)) {
      const depPlugin = pluginStore.plugins.find((p) => p.manifest.id === d.id);
      if (!depPlugin || depPlugin.state !== "enabled") {
        missing.push(d);
      }
    }
  }

  if (plugin.manifest.optional_dependencies) {
    for (const dep of plugin.manifest.optional_dependencies) {
      const depId = typeof dep === "string" ? dep : dep.id;

      if (missing.some((m) => m.id === depId)) continue;
      const depPlugin = pluginStore.plugins.find((p) => p.manifest.id === depId);
      if (!depPlugin || depPlugin.state !== "enabled") {
        missing.push({ id: depId, required: false });
      }
    }
  }
  return missing;
}

function hasMissingOptionalDependencies(plugin: PluginInfo): boolean {
  return getMissingOptionalDependencies(plugin).length > 0;
}

function getDependencyTooltip(plugin: PluginInfo): string {
  const requiredMissing = getMissingRequiredDependencies(plugin);
  const optionalMissing = getMissingOptionalDependencies(plugin);
  const parts: string[] = [];

  if (requiredMissing.length > 0) {
    const names = requiredMissing.map((d) => getDepDisplayName(d.id)).join(", ");
    parts.push(i18n.t("plugins.dep_tooltip.required", { names }));
  }
  if (optionalMissing.length > 0) {
    const names = optionalMissing.map((d) => getDepDisplayName(d.id)).join(", ");
    parts.push(i18n.t("plugins.dep_tooltip.optional", { names }));
  }

  return parts.join("\n");
}

function showMissingDependenciesModal(plugin: PluginInfo) {
  installedPluginName.value = plugin.manifest.name;
  const required = getMissingRequiredDependencies(plugin);
  const optional = getMissingOptionalDependencies(plugin);
  missingDependencies.value = [...required, ...optional];
  showDependencyModal.value = true;
}

function getDepDisplayName(depId: string): string {
  const depPlugin = pluginStore.plugins.find((p) => p.manifest.id === depId);
  return depPlugin ? depPlugin.manifest.name : depId;
}

function getDepStatus(depId: string): string {
  const depPlugin = pluginStore.plugins.find((p) => p.manifest.id === depId);
  if (!depPlugin) return "not-installed";
  if (depPlugin.state !== "enabled") return "not-enabled";
  return "ok";
}

function getDepStatusLabel(depId: string): string {
  const depPlugin = pluginStore.plugins.find((p) => p.manifest.id === depId);
  if (!depPlugin) return i18n.t("plugins.dep_status.not_installed");
  if (depPlugin.state !== "enabled") return i18n.t("plugins.dep_status.disabled");
  return i18n.t("plugins.dep_status.enabled");
}

interface DependencyDetail {
  id: string;
  name: string;
  version?: string;
  status: "enabled" | "disabled" | "not-installed";
  statusLabel: string;
}

function getDependencyDetails(plugin: PluginInfo): DependencyDetail[] {
  if (!plugin.manifest.dependencies || plugin.manifest.dependencies.length === 0) {
    return [];
  }
  return plugin.manifest.dependencies.map((dep) => {
    const depId = typeof dep === "string" ? dep : dep.id;
    const version = typeof dep === "object" ? dep.version : undefined;
    const depPlugin = pluginStore.plugins.find((p) => p.manifest.id === depId);
    let status: "enabled" | "disabled" | "not-installed";
    let statusLabel: string;
    if (!depPlugin) {
      status = "not-installed";
      statusLabel = i18n.t("plugins.dep_status.not_installed");
    } else if (depPlugin.state !== "enabled") {
      status = "disabled";
      statusLabel = i18n.t("plugins.dep_status.installed_not_enabled");
    } else {
      status = "enabled";
      statusLabel = i18n.t("plugins.dep_status.enabled");
    }
    return {
      id: depId,
      name: depPlugin?.manifest.name || depId,
      version,
      status,
      statusLabel,
    };
  });
}

function getOptionalDependencyDetails(plugin: PluginInfo): DependencyDetail[] {
  if (
    !plugin.manifest.optional_dependencies ||
    plugin.manifest.optional_dependencies.length === 0
  ) {
    return [];
  }
  return plugin.manifest.optional_dependencies.map((dep) => {
    const depId = typeof dep === "string" ? dep : dep.id;
    const version = typeof dep === "object" ? dep.version : undefined;
    const depPlugin = pluginStore.plugins.find((p) => p.manifest.id === depId);
    let status: "enabled" | "disabled" | "not-installed";
    let statusLabel: string;
    if (!depPlugin) {
      status = "not-installed";
      statusLabel = i18n.t("plugins.dep_status.not_installed");
    } else if (depPlugin.state !== "enabled") {
      status = "disabled";
      statusLabel = i18n.t("plugins.dep_status.installed_not_enabled");
    } else {
      status = "enabled";
      statusLabel = i18n.t("plugins.dep_status.enabled");
    }
    return {
      id: depId,
      name: depPlugin?.manifest.name || depId,
      version,
      status,
      statusLabel,
    };
  });
}

interface DependentPlugin {
  id: string;
  name: string;
  required: boolean;
}

function getDependentPlugins(plugin: PluginInfo): DependentPlugin[] {
  const dependents: DependentPlugin[] = [];
  const pluginId = plugin.manifest.id;

  for (const p of pluginStore.plugins) {
    if (p.manifest.id === pluginId) continue;

    if (p.manifest.dependencies) {
      for (const dep of p.manifest.dependencies) {
        const depId = typeof dep === "string" ? dep : dep.id;
        if (depId === pluginId) {
          dependents.push({
            id: p.manifest.id,
            name: p.manifest.name,
            required: true,
          });
          break;
        }
      }
    }

    if (!dependents.find((d) => d.id === p.manifest.id) && p.manifest.optional_dependencies) {
      for (const dep of p.manifest.optional_dependencies) {
        const depId = typeof dep === "string" ? dep : dep.id;
        if (depId === pluginId) {
          dependents.push({
            id: p.manifest.id,
            name: p.manifest.name,
            required: false,
          });
          break;
        }
      }
    }
  }

  return dependents;
}

async function openSettings(plugin: PluginInfo) {
  currentSettingsPlugin.value = plugin;
  const savedSettings = await pluginStore.getPluginSettings(plugin.manifest.id);
  Object.keys(settingsForm).forEach((key) => delete settingsForm[key]);
  if (plugin.manifest.settings) {
    for (const field of plugin.manifest.settings) {
      settingsForm[field.key] =
        savedSettings[field.key] ?? field.default ?? getDefaultValue(field.type);
    }
  }
  showSettingsModal.value = true;
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

function closeSettings() {
  showSettingsModal.value = false;
  currentSettingsPlugin.value = null;
}

async function saveSettings() {
  if (!currentSettingsPlugin.value) return;
  savingSettings.value = true;
  try {
    await pluginStore.setPluginSettings(currentSettingsPlugin.value.manifest.id, {
      ...settingsForm,
    });

    if (pluginStore.hasCapability(currentSettingsPlugin.value.manifest.id, "theme-provider")) {
      await pluginStore.applyThemeProviderSettings(currentSettingsPlugin.value.manifest.id);
    }
    closeSettings();
  } finally {
    savingSettings.value = false;
  }
}

function getPluginMenuItems(pluginId: string) {
  return [
    {
      id: "check_update",
      label: i18n.t("plugins.menu.check_update"),
      icon: RefreshCw,
      disabled: checkingUpdate.value === pluginId,
    },
    { id: "divider", label: "", divider: true },
    {
      id: "delete",
      label: i18n.t("plugins.menu.delete"),
      icon: Trash2,
      danger: true,
    },
  ];
}

async function handleMenuSelect(item: { id: string | number }, pluginId: string) {
  switch (item.id) {
    case "check_update":
      await handleCheckUpdate(pluginId);
      break;
    case "delete":
      await handleDelete(pluginId);
      break;
  }
}

async function handleCheckUpdate(pluginId: string) {
  checkingUpdate.value = pluginId;
  try {
    const update = await pluginStore.checkUpdate(pluginId);
    if (update) {
      showAlert(
        i18n.t("plugins.new_version_found"),
        `${i18n.t("plugins.latest_version")}: ${update.latest_version}\n${i18n.t("plugins.current_version")}: ${update.current_version}`,
      );
    } else {
      showAlert(i18n.t("plugins.check_update"), i18n.t("plugins.already_latest"));
    }
  } finally {
    checkingUpdate.value = null;
  }
}

const pendingDeletePluginId = ref<string | null>(null);
const showSingleDeleteDialog = ref(false);
const singleDeletePluginName = ref("");

async function handleDelete(pluginId: string) {
  const plugin = pluginStore.plugins.find((p) => p.manifest.id === pluginId);
  if (plugin?.state === "enabled") {
    showAlert(i18n.t("plugins.cannot_delete_enabled"), plugin.manifest.name);
    return;
  }

  pendingDeletePluginId.value = pluginId;
  singleDeletePluginName.value = plugin?.manifest?.name || pluginId;
  showSingleDeleteDialog.value = true;
}

async function executeSingleDelete(deleteData: boolean) {
  showSingleDeleteDialog.value = false;
  if (!pendingDeletePluginId.value) return;
  try {
    await pluginStore.deletePlugin(pendingDeletePluginId.value, deleteData);
  } catch (e) {
    showAlert(i18n.t("common.message_unknown_error"), String(e));
  } finally {
    pendingDeletePluginId.value = null;
  }
}

function toggleBatchMode() {
  batchMode.value = !batchMode.value;
  if (!batchMode.value) {
    selectedPlugins.value.clear();
    selectedPlugins.value = new Set(selectedPlugins.value);
  }
}

function togglePluginSelection(pluginId: string) {
  if (selectedPlugins.value.has(pluginId)) {
    selectedPlugins.value.delete(pluginId);
  } else {
    selectedPlugins.value.add(pluginId);
  }
  selectedPlugins.value = new Set(selectedPlugins.value);
}

function selectAll() {
  selectedPlugins.value = new Set(pluginStore.plugins.map((p) => p.manifest.id));
}

function deselectAll() {
  selectedPlugins.value.clear();
  selectedPlugins.value = new Set(selectedPlugins.value);
}

function invertSelection() {
  const newSet = new Set<string>();
  for (const p of pluginStore.plugins) {
    if (!selectedPlugins.value.has(p.manifest.id)) {
      newSet.add(p.manifest.id);
    }
  }
  selectedPlugins.value = newSet;
}

function isAllSelected(): boolean {
  return (
    pluginStore.plugins.length > 0 && selectedPlugins.value.size === pluginStore.plugins.length
  );
}

function showBatchDeleteConfirm() {
  showBatchDeleteDialog.value = true;
}

async function executeBatchDelete(deleteData: boolean) {
  showBatchDeleteDialog.value = false;
  const ids = Array.from(selectedPlugins.value);

  const enabledNames = ids
    .map((id) => pluginStore.plugins.find((p) => p.manifest.id === id))
    .filter((p) => p?.state === "enabled")
    .map((p) => p!.manifest.name);
  if (enabledNames.length > 0) {
    showAlert(i18n.t("plugins.cannot_delete_enabled"), enabledNames.join(", "));
    return;
  }

  try {
    await pluginStore.deletePlugins(ids, deleteData);
    selectedPlugins.value.clear();
    batchMode.value = false;
  } catch (e) {
    showAlert(i18n.t("common.message_unknown_error"), String(e));
  }
}

async function handleCheckAllUpdates() {
  checkingAllUpdates.value = true;
  try {
    const updates = await pluginStore.checkAllUpdates();
    if (updates.length > 0) {
      showAlert(
        i18n.t("plugins.check_update"),
        i18n.t("plugins.updates_available", { count: updates.length }),
      );
    } else {
      showAlert(i18n.t("plugins.check_update"), i18n.t("plugins.all_plugins_latest"));
    }
  } finally {
    checkingAllUpdates.value = false;
  }
}

function openRepository(url: string) {
  openUrl(url);
}

function goToMarket() {
  showDependencyModal.value = false;
  router.push("/plugins?tab=market");
}
</script>

<template>
  <div class="plugins-view">
    <div class="plugins-toolbar">
      <div class="toolbar-left">
        <input
          v-model="searchQuery"
          type="text"
          class="plugin-search"
          :placeholder="i18n.t('plugins.search_placeholder')"
        />
      </div>
      <div class="toolbar-right">
        <SLButton :variant="batchMode ? 'primary' : 'secondary'" size="sm" @click="toggleBatchMode">
          {{ i18n.t("plugins.batch_mode") }}
        </SLButton>
        <SLButton
          variant="secondary"
          size="sm"
          :loading="checkingAllUpdates"
          @click="handleCheckAllUpdates"
        >
          {{ i18n.t("plugins.check_updates") }}
        </SLButton>
        <SLButton
          variant="secondary"
          size="sm"
          :loading="pluginStore.loading"
          @click="handleRefresh"
        >
          {{ i18n.t("plugins.refresh") }}
        </SLButton>
      </div>
    </div>

    <div class="upload-zone" :class="{ 'is-dragging': isDragging, 'is-installing': isInstalling }">
      <div v-if="isInstalling" class="upload-loading">
        <div class="loading-spinner"></div>
        <span class="upload-text">{{ i18n.t("plugins.installing") }}</span>
      </div>
      <div v-else class="upload-content">
        <Upload class="upload-icon" :size="32" :stroke-width="1.5" />
        <span class="upload-text">{{ i18n.t("plugins.drag_hint") }}</span>
        <div class="upload-buttons">
          <SLButton variant="secondary" size="sm" @click="handleSelectFile">{{
            i18n.t("plugins.select_file")
          }}</SLButton>
          <SLButton variant="secondary" size="sm" @click="handleSelectFolder">{{
            i18n.t("plugins.select_folder")
          }}</SLButton>
        </div>
      </div>
    </div>

    <div v-if="pluginStore.error" class="error-banner">
      <span class="error-icon">!</span>
      <span class="error-text">{{ pluginStore.error }}</span>
    </div>

    <div v-if="pluginStore.loading && pluginStore.plugins.length === 0" class="loading-state">
      <div class="loading-spinner"></div>
      <span class="loading-text">{{ i18n.t("plugins.loading_plugins") }}</span>
    </div>

    <div v-else-if="!pluginStore.loading && pluginStore.plugins.length === 0" class="empty-state">
      <div class="empty-icon">
        <Layers :size="48" :stroke-width="1.5" />
      </div>
      <h3 class="empty-title">{{ i18n.t("plugins.no_plugins") }}</h3>
      <p class="empty-desc">{{ i18n.t("plugins.no_plugins_desc") }}</p>
    </div>

    <div v-else>
      <div v-if="batchMode" class="batch-action-bar">
        <div class="batch-action-left">
          <span class="selected-count">{{
            i18n.t("plugins.selected_count", { count: selectedPlugins.size })
          }}</span>
        </div>
        <div class="batch-action-right">
          <SLButton variant="secondary" size="sm" @click="selectAll">
            {{ i18n.t("plugins.select_all") }}
          </SLButton>
          <SLButton variant="secondary" size="sm" @click="invertSelection">
            {{ i18n.t("plugins.invert_selection") }}
          </SLButton>
          <SLButton variant="secondary" size="sm" @click="deselectAll">
            {{ i18n.t("plugins.deselect_all") }}
          </SLButton>
          <SLButton
            variant="danger"
            size="sm"
            :disabled="selectedPlugins.size === 0"
            @click="showBatchDeleteConfirm"
          >
            {{ i18n.t("plugins.batch_delete") }}
          </SLButton>
        </div>
      </div>
      <div class="plugin-grid">
        <SLCard
          v-for="plugin in filteredPlugins"
          :key="plugin.manifest.id"
          class="plugin-card"
          :class="{ 'plugin-card--selected': batchMode && selectedPlugins.has(plugin.manifest.id) }"
        >
          <div class="plugin-content">
            <label v-if="batchMode" class="plugin-checkbox" @click.stop>
              <SLCheckbox
                :modelValue="selectedPlugins.has(plugin.manifest.id)"
                @update:modelValue="togglePluginSelection(plugin.manifest.id)"
              />
            </label>

            <div class="plugin-card-actions">
              <div
                v-if="pluginStore.updates[plugin.manifest.id]"
                class="update-badge"
                :title="i18n.t('plugins.update_available')"
              >
                <ShieldAlert :size="12" />
              </div>

              <div
                v-if="hasMissingRequiredDependencies(plugin)"
                class="dependency-indicator dependency-indicator--required"
                :title="getDependencyTooltip(plugin)"
                @click.stop="showMissingDependenciesModal(plugin)"
              ></div>
              <div
                v-else-if="hasMissingOptionalDependencies(plugin)"
                class="dependency-indicator dependency-indicator--optional"
                :title="getDependencyTooltip(plugin)"
                @click.stop="showMissingDependenciesModal(plugin)"
              ></div>

              <PluginPermissionPanel
                :plugin-id="plugin.manifest.id"
                :permissions="plugin.manifest.permissions || []"
              />

              <SLMenu
                :items="getPluginMenuItems(plugin.manifest.id)"
                position="bottom-end"
                @select="handleMenuSelect($event, plugin.manifest.id)"
              >
                <SLButton variant="ghost" icon-only size="sm">
                  <MoreVertical :size="16" />
                </SLButton>
              </SLMenu>
            </div>
            <div class="plugin-main">
              <div class="plugin-icon">
                <img
                  v-if="pluginStore.icons[plugin.manifest.id]"
                  :src="pluginStore.icons[plugin.manifest.id]"
                  alt="plugin icon"
                  class="plugin-icon-img"
                />
                <Layers v-else :size="32" :stroke-width="1.5" class="plugin-icon-default" />
              </div>
              <div class="plugin-info">
                <div class="plugin-header">
                  <div class="plugin-title-row">
                    <h3 class="plugin-name">
                      {{ getLocalizedPluginName(plugin.manifest, i18n.getLocale()) }}
                    </h3>
                    <span class="plugin-version">v{{ plugin.manifest.version }}</span>
                  </div>
                  <div class="plugin-author-row">
                    <span v-if="plugin.manifest.author" class="plugin-author">
                      by {{ plugin.manifest.author.name }}
                    </span>
                    <SLButton
                      v-if="plugin.manifest.repository"
                      variant="ghost"
                      icon-only
                      size="sm"
                      @click.stop="openRepository(plugin.manifest.repository)"
                      :title="i18n.t('plugins.open_repository')"
                    >
                      <Github :size="14" />
                    </SLButton>
                  </div>
                </div>
                <p v-if="plugin.manifest.description" class="plugin-description">
                  {{ getLocalizedPluginDescription(plugin.manifest, i18n.getLocale()) }}
                </p>
                <p
                  v-if="typeof plugin.state === 'object' && 'error' in plugin.state"
                  class="plugin-error-message"
                >
                  {{ plugin.state.error }}
                </p>
              </div>
            </div>

            <div class="plugin-footer">
              <span
                class="plugin-status"
                :style="{ color: getStatusColor(plugin.state) }"
                :title="
                  typeof plugin.state === 'object' && 'error' in plugin.state
                    ? plugin.state.error
                    : undefined
                "
              >
                {{ getStatusLabel(plugin.state) }}
              </span>
              <div class="plugin-actions">
                <SLButton
                  v-if="hasSettings(plugin)"
                  variant="ghost"
                  icon-only
                  size="sm"
                  @click="openSettings(plugin)"
                  :title="i18n.t('plugins.settings')"
                >
                  <Settings :size="16" />
                </SLButton>
                <SLSwitch
                  :modelValue="isPluginEnabled(plugin.state)"
                  :disabled="
                    hasMissingRequiredDependencies(plugin) && !isPluginEnabled(plugin.state)
                  "
                  :title="
                    hasMissingRequiredDependencies(plugin) && !isPluginEnabled(plugin.state)
                      ? i18n.t('plugins.missing_required_deps')
                      : ''
                  "
                  @update:modelValue="
                    handleToggle(plugin.manifest.id, isPluginEnabled(plugin.state))
                  "
                  size="sm"
                />
              </div>
            </div>
          </div>
        </SLCard>
      </div>
    </div>

    <Teleport to="body">
      <div v-if="showSettingsModal" class="modal-overlay" @click.self="closeSettings">
        <div class="settings-modal glass">
          <div class="modal-header">
            <h2 class="modal-title">
              {{ i18n.t("plugins.settings_title", { name: currentSettingsPlugin?.manifest.name }) }}
            </h2>
            <SLButton variant="ghost" icon-only class="modal-close" @click="closeSettings">
              <X :size="20" />
            </SLButton>
          </div>
          <div class="modal-body">
            <div
              v-for="field in currentSettingsPlugin?.manifest.settings"
              :key="field.key"
              class="setting-field"
            >
              <label class="setting-label">
                {{ field.label }}
                <span v-if="field.description" class="setting-desc">{{ field.description }}</span>
              </label>
              <SLInput v-if="field.type === 'string'" v-model="settingsForm[field.key]" />
              <div v-else-if="field.type === 'color'" class="setting-color-field">
                <input
                  type="color"
                  v-model="settingsForm[field.key]"
                  class="setting-color-picker"
                />
                <SLInput v-model="settingsForm[field.key]" />
              </div>
              <SLInput
                v-else-if="field.type === 'number'"
                v-model="settingsForm[field.key]"
                type="number"
              />
              <label v-else-if="field.type === 'boolean'" class="setting-toggle">
                <SLSwitch
                  :modelValue="Boolean(settingsForm[field.key])"
                  @update:modelValue="settingsForm[field.key] = $event"
                  size="sm"
                />
              </label>
              <SLSelect
                v-else-if="field.type === 'select'"
                v-model="settingsForm[field.key]"
                :options="field.options"
              />
            </div>

            <div class="plugin-details-section">
              <div v-if="currentSettingsPlugin?.manifest.permissions?.length" class="detail-block">
                <h4 class="detail-title">{{ i18n.t("plugins.permissions") }}</h4>
                <div class="permission-tags">
                  <span
                    v-for="perm in currentSettingsPlugin.manifest.permissions"
                    :key="perm"
                    class="permission-tag"
                    :title="getPermissionDesc(perm)"
                    >{{ getPermissionLabel(perm) }}</span
                  >
                </div>
              </div>

              <div
                v-if="
                  currentSettingsPlugin && getDependencyDetails(currentSettingsPlugin).length > 0
                "
                class="detail-block"
              >
                <h4 class="detail-title">{{ i18n.t("plugins.dependencies") }}</h4>
                <ul class="dependency-list">
                  <li
                    v-for="dep in getDependencyDetails(currentSettingsPlugin)"
                    :key="dep.id"
                    class="dependency-item"
                  >
                    <span class="dep-name">{{ dep.name }}</span>
                    <span v-if="dep.version" class="dep-version">{{ dep.version }}</span>
                    <span :class="['dep-status', `dep-status--${dep.status}`]">{{
                      dep.statusLabel
                    }}</span>
                  </li>
                </ul>
              </div>

              <div
                v-if="
                  currentSettingsPlugin &&
                  getOptionalDependencyDetails(currentSettingsPlugin).length > 0
                "
                class="detail-block"
              >
                <h4 class="detail-title">{{ i18n.t("plugins.optional_dependencies") }}</h4>
                <ul class="dependency-list">
                  <li
                    v-for="dep in getOptionalDependencyDetails(currentSettingsPlugin)"
                    :key="dep.id"
                    class="dependency-item"
                  >
                    <span class="dep-name">{{ dep.name }}</span>
                    <span v-if="dep.version" class="dep-version">{{ dep.version }}</span>
                    <span :class="['dep-status', `dep-status--${dep.status}`]">{{
                      dep.statusLabel
                    }}</span>
                  </li>
                </ul>
              </div>

              <div
                v-if="
                  currentSettingsPlugin && getDependentPlugins(currentSettingsPlugin).length > 0
                "
                class="detail-block"
              >
                <h4 class="detail-title">{{ i18n.t("plugins.dependents") }}</h4>
                <ul class="dependency-list">
                  <li
                    v-for="dep in getDependentPlugins(currentSettingsPlugin)"
                    :key="dep.id"
                    class="dependency-item"
                  >
                    <span class="dep-name">{{ dep.name }}</span>
                    <span
                      :class="[
                        'dep-type-tag',
                        dep.required ? 'dep-type-tag--required' : 'dep-type-tag--optional',
                      ]"
                    >
                      {{
                        dep.required
                          ? i18n.t("plugins.dep_required")
                          : i18n.t("plugins.dep_optional")
                      }}
                    </span>
                  </li>
                </ul>
              </div>
            </div>
          </div>
          <div class="modal-footer">
            <SLButton variant="secondary" size="sm" @click="closeSettings">{{
              i18n.t("plugins.cancel")
            }}</SLButton>
            <SLButton variant="primary" size="sm" :loading="savingSettings" @click="saveSettings">{{
              i18n.t("plugins.save")
            }}</SLButton>
          </div>
        </div>
      </div>
    </Teleport>

    <SLModal :visible="confirmDialog.show" :title="confirmDialog.title" @close="closeConfirmDialog">
      <p class="dialog-message">{{ confirmDialog.message }}</p>
      <template #footer>
        <SLButton variant="secondary" size="sm" @click="closeConfirmDialog">{{
          i18n.t("plugins.cancel")
        }}</SLButton>
        <SLButton variant="danger" size="sm" @click="executeConfirmDialog">{{
          i18n.t("plugins.delete")
        }}</SLButton>
      </template>
    </SLModal>

    <SLModal
      :visible="showSingleDeleteDialog"
      :title="i18n.t('plugins.confirm_delete')"
      @close="showSingleDeleteDialog = false"
    >
      <div class="batch-delete-dialog">
        <p class="dialog-message">
          {{ i18n.t("plugins.confirm_delete_message", { name: singleDeletePluginName }) }}
        </p>
        <div class="batch-delete-options">
          <SLButton
            variant="secondary"
            class="batch-delete-option"
            @click="executeSingleDelete(true)"
          >
            <Trash2 class="option-icon delete-with-data" :size="20" />
            <span class="option-label">{{ i18n.t("plugins.delete_with_data") }}</span>
          </SLButton>
          <SLButton
            variant="secondary"
            class="batch-delete-option"
            @click="executeSingleDelete(false)"
          >
            <Trash class="option-icon delete-without-data" :size="20" />
            <span class="option-label">{{ i18n.t("plugins.delete_without_data") }}</span>
          </SLButton>
        </div>
      </div>
      <template #footer>
        <SLButton variant="secondary" size="sm" @click="showSingleDeleteDialog = false">{{
          i18n.t("plugins.cancel")
        }}</SLButton>
      </template>
    </SLModal>

    <SLModal
      :visible="showBatchDeleteDialog"
      :title="i18n.t('plugins.confirm_batch_delete')"
      @close="showBatchDeleteDialog = false"
    >
      <div class="batch-delete-dialog">
        <p class="dialog-message">
          {{ i18n.t("plugins.confirm_batch_delete_message", { count: selectedPlugins.size }) }}
        </p>
        <div class="batch-delete-options">
          <SLButton
            variant="secondary"
            class="batch-delete-option"
            @click="executeBatchDelete(true)"
          >
            <Trash2 class="option-icon delete-with-data" :size="20" />
            <span class="option-label">{{ i18n.t("plugins.delete_with_data") }}</span>
          </SLButton>
          <SLButton
            variant="secondary"
            class="batch-delete-option"
            @click="executeBatchDelete(false)"
          >
            <Trash class="option-icon delete-without-data" :size="20" />
            <span class="option-label">{{ i18n.t("plugins.delete_without_data") }}</span>
          </SLButton>
        </div>
      </div>
      <template #footer>
        <SLButton variant="secondary" size="sm" @click="showBatchDeleteDialog = false">{{
          i18n.t("plugins.cancel")
        }}</SLButton>
      </template>
    </SLModal>

    <SLModal
      :visible="alertDialog.show"
      :title="alertDialog.title"
      :auto-close="3000"
      @close="closeAlertDialog"
    >
      <p class="dialog-message">{{ alertDialog.message }}</p>
      <template #footer>
        <SLButton variant="primary" size="sm" @click="closeAlertDialog">{{
          i18n.t("plugins.ok")
        }}</SLButton>
      </template>
    </SLModal>

    <SLPermissionDialog
      :show="permissionWarning.show"
      :plugin-name="permissionWarning.pluginName"
      :permissions="permissionWarning.permissions"
      @confirm="confirmPermissionWarning"
      @cancel="cancelPermissionWarning"
    />

    <SLModal
      :visible="showDependencyModal"
      :title="i18n.t('plugins.missing_deps_title')"
      @close="showDependencyModal = false"
    >
      <div class="dependency-dialog">
        <p class="dependency-intro">
          {{ i18n.t("plugins.missing_deps_intro", { name: installedPluginName }) }}
        </p>
        <ul class="dependency-list">
          <li v-for="dep in missingDependencies" :key="dep.id" class="dependency-item">
            <span class="dependency-name">{{ getDepDisplayName(dep.id) }}</span>
            <span v-if="dep.version_requirement" class="dependency-version">{{
              dep.version_requirement
            }}</span>
            <span :class="['dependency-badge', dep.required ? 'required' : 'optional']">
              {{ dep.required ? i18n.t("plugins.dep_required") : i18n.t("plugins.dep_optional") }}
            </span>
          </li>
        </ul>
        <p class="dependency-hint">
          {{ i18n.t("plugins.missing_deps_hint") }}
        </p>
      </div>
      <template #footer>
        <SLButton variant="secondary" size="sm" @click="showDependencyModal = false">{{
          i18n.t("plugins.later")
        }}</SLButton>
        <SLButton variant="primary" size="sm" @click="goToMarket">{{
          i18n.t("plugins.go_market")
        }}</SLButton>
      </template>
    </SLModal>

    <SLModal
      :visible="showBatchResultModal"
      :title="i18n.t('plugins.batch_result_title')"
      @close="showBatchResultModal = false"
    >
      <div class="batch-result-dialog" v-if="batchInstallResult">
        <div v-if="batchInstallResult.success.length > 0" class="batch-success-section">
          <p class="batch-section-title">
            {{ i18n.t("plugins.batch_success", { count: batchInstallResult.success.length }) }}
          </p>
          <ul class="batch-list">
            <li
              v-for="item in batchInstallResult.success"
              :key="item.plugin.manifest.id"
              class="batch-item success"
            >
              <span class="batch-item-name">{{ item.plugin.manifest.name }}</span>
              <span class="batch-item-version">v{{ item.plugin.manifest.version }}</span>
            </li>
          </ul>
        </div>
        <div v-if="batchInstallResult.failed.length > 0" class="batch-failed-section">
          <p class="batch-section-title">
            {{ i18n.t("plugins.batch_failed", { count: batchInstallResult.failed.length }) }}
          </p>
          <ul class="batch-list">
            <li
              v-for="item in batchInstallResult.failed"
              :key="item.path"
              class="batch-item failed"
            >
              <span class="batch-item-path">{{ item.path.split(/[/\\]/).pop() }}</span>
              <span class="batch-item-error">{{ item.error }}</span>
            </li>
          </ul>
        </div>
      </div>
      <template #footer>
        <SLButton variant="primary" size="sm" @click="showBatchResultModal = false">{{
          i18n.t("plugins.ok")
        }}</SLButton>
      </template>
    </SLModal>
  </div>
</template>

<style scoped>
.plugins-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
  min-height: 100%;
  flex: 1;
}

.plugins-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-md);
  padding: var(--sl-space-xs);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  margin-bottom: var(--sl-space-md);
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}

.plugin-search {
  padding: 6px 12px;
  border-radius: var(--sl-radius-sm);
  border: 1px solid var(--sl-border);
  background: var(--sl-bg-secondary);
  color: var(--sl-text-primary);
  font-size: 13px;
  width: 180px;
  transition: all var(--sl-transition-fast);
}

.plugin-search:focus {
  outline: none;
  border-color: var(--sl-primary);
}

.upload-zone {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  margin-bottom: 16px;
  border: 2px dashed var(--sl-border);
  border-radius: var(--sl-radius-lg);
  cursor: pointer;
  transition: all 0.2s ease;
  background: var(--sl-bg-primary);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.upload-zone:hover {
  border-color: var(--sl-primary);
  background: var(--sl-bg-tertiary);
}

.upload-zone.is-dragging {
  border-style: solid;
  border-color: var(--sl-primary);
  background: var(--sl-primary-bg);
}

.upload-zone.is-installing {
  pointer-events: none;
  opacity: 0.8;
}

.upload-content,
.upload-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.upload-icon {
  color: var(--sl-text-tertiary);
  transition: color 0.2s ease;
}

.upload-zone:hover .upload-icon,
.upload-zone.is-dragging .upload-icon {
  color: var(--sl-primary);
}

.upload-text {
  font-size: 14px;
  color: var(--sl-text-secondary);
  text-align: center;
}

.upload-buttons {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}

.batch-result-dialog {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.batch-section-title {
  font-size: 14px;
  color: var(--sl-text-primary);
  margin: 0 0 8px 0;
}

.batch-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.batch-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: var(--sl-radius-sm);
  font-size: 13px;
}

.batch-item.success {
  background: var(--sl-success-bg);
  border: 1px solid var(--sl-success);
}

.batch-item.failed {
  background: var(--sl-error-bg);
  border: 1px solid var(--sl-error);
  flex-direction: column;
  align-items: flex-start;
}

.batch-item-name {
  color: var(--sl-text-primary);
  font-weight: 500;
}

.batch-item-version {
  color: var(--sl-text-tertiary);
  font-size: 12px;
}

.batch-item-path {
  color: var(--sl-text-primary);
  font-weight: 500;
}

.batch-item-error {
  color: var(--sl-error);
  font-size: 12px;
}

.error-banner {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  margin-bottom: 16px;
  border-radius: var(--sl-radius-md);
  background: var(--sl-error-bg);
  border: 1px solid var(--sl-error);
}

.error-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--sl-error);
  color: var(--sl-text-inverse);
  font-size: 12px;
  font-weight: 700;
}

.error-text {
  color: var(--sl-error);
  font-size: 14px;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 24px;
  border-radius: var(--sl-radius-md);
  text-align: center;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--sl-border);
  border-top-color: var(--sl-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.loading-text {
  margin-top: 16px;
  color: var(--sl-text-secondary);
  font-size: 14px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 24px;
  border-radius: var(--sl-radius-md);
  text-align: center;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
}

.empty-icon {
  color: var(--sl-text-tertiary);
  margin-bottom: 16px;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 8px 0;
}

.empty-desc {
  font-size: 14px;
  color: var(--sl-text-secondary);
  margin: 0;
  max-width: 320px;
}

.plugin-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--sl-space-md);
}

@media (max-width: 1200px) {
  .plugin-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 700px) {
  .plugin-grid {
    grid-template-columns: 1fr;
  }
}

.plugin-card {
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease;
  height: 100%;
}

.plugin-card:hover {
  transform: translateY(-2px);
}

.plugin-card--selected {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 1px var(--sl-primary);
}

.batch-action-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  margin-bottom: 12px;
  border-radius: var(--sl-radius-md);
  border: 1px solid var(--sl-border);
  background: var(--sl-bg-primary);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.batch-action-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.batch-action-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.selected-count {
  font-size: 14px;
  color: var(--sl-text-primary);
  font-weight: 500;
}

.plugin-checkbox {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 5;
  display: flex;
  align-items: center;
}

.batch-delete-dialog {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.batch-delete-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.batch-delete-option {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border: none;
  border-radius: var(--sl-radius-md);
  background: var(--sl-bg-tertiary);
  cursor: pointer;
  transition: all var(--sl-transition-fast);
}

.batch-delete-option:hover {
  background: var(--sl-border);
}

.batch-delete-option:active {
  transform: scale(0.98);
}

.batch-delete-option .option-icon {
  flex-shrink: 0;
}

.batch-delete-option .option-icon.delete-with-data {
  color: var(--sl-error);
}

.batch-delete-option .option-icon.delete-without-data {
  color: var(--sl-warning);
}

.batch-delete-option .option-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--sl-text-primary);
}

.plugin-content {
  padding: 8px;
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100%;
}

.plugin-main {
  display: flex;
  gap: 12px;
  margin-bottom: 8px;
  flex: 1;
}

.plugin-icon {
  flex-shrink: 0;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.plugin-icon-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  border-radius: var(--sl-radius-md);
}

.plugin-icon-default {
  color: var(--sl-text-tertiary);
}

.plugin-info {
  flex: 1;
  min-width: 0;
}

.plugin-header {
  margin-bottom: 4px;
}

.plugin-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.plugin-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
}

.plugin-version {
  flex-shrink: 0;
  padding: 1px 5px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-xs);
  font-size: 11px;
  color: var(--sl-text-tertiary);
}

.plugin-author {
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.plugin-author-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 2px;
}

.repo-link-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  padding: 0;
  background: transparent;
  border: none;
  border-radius: var(--sl-radius-xs);
  color: var(--sl-text-tertiary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.repo-link-btn:hover {
  background: var(--sl-bg-tertiary);
  color: var(--sl-primary);
}

.plugin-description {
  margin: 6px 0;
  font-size: 13px;
  color: var(--sl-text-secondary);
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.plugin-error-message {
  margin: 4px 0 0;
  font-size: 12px;
  color: var(--sl-error);
  line-height: 1.4;
  word-break: break-word;
}

.plugin-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 8px;
  border-top: 1px solid var(--sl-border);
  margin-top: auto;
}

.plugin-status {
  font-size: 12px;
  font-weight: 500;
}

.plugin-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.settings-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-sm);
  color: var(--sl-text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.settings-btn:hover {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

.settings-modal {
  width: 90%;
  max-width: 480px;
  max-height: 80vh;
  background: var(--sl-surface);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-radius: var(--sl-radius-lg);
  border: 1px solid var(--sl-border);
  box-shadow: var(--sl-shadow-xl);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  font-family: var(--sl-font-sans);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--sl-border);
}

.modal-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0;
}

.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--sl-text-secondary);
  cursor: pointer;
  border-radius: var(--sl-radius-md);
  transition: all 0.2s ease;
}

.modal-close:hover {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-primary);
}

.modal-body {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
}

.setting-field {
  margin-bottom: 16px;
}

.setting-field:last-child {
  margin-bottom: 0;
}

.setting-label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--sl-text-primary);
  margin-bottom: 8px;
}

.setting-desc {
  display: block;
  font-size: 12px;
  font-weight: 400;
  color: var(--sl-text-tertiary);
  margin-top: 2px;
}

.setting-input {
  width: 100%;
  padding: 10px 14px;
  font-size: 14px;
  color: var(--sl-text-primary);
  background: var(--sl-bg-secondary);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  outline: none;
  transition: all 0.2s ease;
}

.setting-input:hover {
  border-color: var(--sl-border-light);
  background: var(--sl-bg-tertiary);
}

.setting-input:focus {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.15);
  background: var(--sl-bg);
}

.setting-color-field {
  display: flex;
  align-items: center;
  gap: 8px;
}

.setting-color-picker {
  width: 40px;
  height: 38px;
  padding: 2px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: var(--sl-radius-md);
  background: transparent;
  cursor: pointer;
  flex-shrink: 0;
}

.setting-color-field .setting-input {
  flex: 1;
  width: auto;
}

.setting-input::placeholder {
  color: var(--sl-text-tertiary);
}

.setting-select {
  width: 100%;
  padding: 10px 14px;
  padding-right: 36px;
  font-size: 14px;
  color: var(--sl-text-primary);
  background: var(--sl-bg-secondary);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  outline: none;
  cursor: pointer;
  transition: all 0.2s ease;
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 24 24' fill='none' stroke='%2394a3b8' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
  background-size: 16px;
}

.setting-select:hover {
  border-color: var(--sl-border-light);
  background-color: var(--sl-bg-tertiary);
}

.setting-select:focus {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.15);
  background-color: var(--sl-bg);
}

.setting-select option {
  background: var(--sl-bg-secondary);
  color: var(--sl-text-primary);
  padding: 8px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--sl-border);
}

.plugin-card-actions {
  position: absolute;
  top: 0;
  right: 0;
  display: flex;
  align-items: center;
  gap: 4px;
  z-index: 10;
}

.update-badge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  background: var(--sl-primary);
  border-radius: 50%;
  color: var(--sl-text-inverse);
}

.dependency-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  cursor: pointer;
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease;
  flex-shrink: 0;
}

.dependency-indicator:hover {
  transform: scale(1.3);
}

.dependency-indicator--required {
  background: #ef4444;
  box-shadow: 0 0 6px rgba(239, 68, 68, 0.5);
}

.dependency-indicator--required:hover {
  box-shadow: 0 0 10px rgba(239, 68, 68, 0.7);
}

.dependency-indicator--optional {
  background: #f59e0b;
  box-shadow: 0 0 6px rgba(245, 158, 11, 0.5);
}

.dependency-indicator--optional:hover {
  box-shadow: 0 0 10px rgba(245, 158, 11, 0.7);
}

.header-right {
  display: flex;
  gap: 8px;
}

.dialog-message {
  margin: 0;
  color: var(--sl-text-secondary, #6b7280);
  font-size: 14px;
  line-height: 1.6;
  white-space: pre-line;
}

.permission-warning-dialog {
  padding: 4px 0;
}

.permission-warning-message {
  margin: 0 0 16px 0;
  color: var(--sl-text-secondary, #6b7280);
  font-size: 14px;
  line-height: 1.6;
}

.permission-warning-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.permission-warning-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 12px;
  margin-bottom: 8px;
  background: var(--sl-bg-tertiary, rgba(255, 255, 255, 0.05));
  border-radius: var(--sl-radius-md);
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.permission-warning-item:last-child {
  margin-bottom: 0;
}

.permission-warning-icon {
  flex-shrink: 0;
  color: #ef4444;
  margin-top: 2px;
}

.permission-warning-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.permission-warning-name {
  font-weight: 500;
  color: var(--sl-text-primary, #e2e8f0);
  font-size: 14px;
}

.permission-warning-desc {
  color: var(--sl-text-secondary, #6b7280);
  font-size: 12px;
  line-height: 1.5;
}

.dependency-dialog {
  padding: 4px 0;
}

.dependency-intro {
  margin: 0 0 16px 0;
  color: var(--sl-text-secondary, #6b7280);
  font-size: 14px;
  line-height: 1.6;
}

.dependency-intro strong {
  color: var(--sl-text-primary, #e2e8f0);
}

.dependency-list {
  list-style: none;
  margin: 0 0 16px 0;
  padding: 0;
}

.dependency-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  margin-bottom: 8px;
  background: var(--sl-bg-tertiary, rgba(255, 255, 255, 0.05));
  border-radius: var(--sl-radius-md);
  border: 1px solid var(--sl-border, rgba(255, 255, 255, 0.08));
}

.dependency-item:last-child {
  margin-bottom: 0;
}

.dependency-name {
  font-weight: 500;
  color: var(--sl-text-primary, #e2e8f0);
  font-size: 14px;
}

.dependency-version {
  font-size: 12px;
  color: var(--sl-text-tertiary, #64748b);
  font-family: monospace;
}

.dependency-badge {
  margin-left: auto;
  padding: 2px 8px;
  border-radius: var(--sl-radius-xs);
  font-size: 11px;
  font-weight: 500;
}

.dependency-badge.not-installed {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.dependency-badge.not-enabled {
  background: rgba(234, 179, 8, 0.15);
  color: #eab308;
}

.dependency-badge.ok {
  background: rgba(74, 222, 128, 0.15);
  color: #4ade80;
}

.dependency-badge.required {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.dependency-badge.optional {
  background: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
}

.dependency-hint {
  margin: 0;
  color: var(--sl-text-tertiary, #64748b);
  font-size: 13px;
  line-height: 1.5;
}

.plugin-details-section {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--sl-border, rgba(255, 255, 255, 0.1));
}

.detail-block {
  margin-bottom: 16px;
}

.detail-block:last-child {
  margin-bottom: 0;
}

.detail-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--sl-text-secondary, #94a3b8);
  margin: 0 0 8px 0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.permission-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.permission-tag {
  display: inline-flex;
  align-items: center;
  padding: 4px 10px;
  font-size: 12px;
  font-weight: 500;
  color: var(--sl-text-primary, #e2e8f0);
  background: rgba(99, 102, 241, 0.15);
  border: 1px solid rgba(99, 102, 241, 0.3);
  border-radius: var(--sl-radius-sm);
}

.dependency-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.dependency-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  margin-bottom: 4px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: var(--sl-radius-md);
  font-size: 13px;
}

.dependency-item:last-child {
  margin-bottom: 0;
}

.dep-name {
  flex: 1;
  color: var(--sl-text-primary, #e2e8f0);
  font-weight: 500;
}

.dep-version {
  color: var(--sl-text-tertiary, #64748b);
  font-size: 12px;
  font-family: var(--sl-font-mono, monospace);
}

.dep-status {
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 500;
  border-radius: var(--sl-radius-xs);
}

.dep-status--enabled {
  background: rgba(74, 222, 128, 0.15);
  color: #4ade80;
}

.dep-status--disabled {
  background: rgba(250, 204, 21, 0.15);
  color: #facc15;
}

.dep-status--not-installed {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.dep-type-tag {
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 500;
  border-radius: var(--sl-radius-xs);
}

.dep-type-tag--required {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.dep-type-tag--optional {
  background: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
}
</style>
