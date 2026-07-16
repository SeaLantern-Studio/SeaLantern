<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useServerStore } from "@stores/serverStore";
import {
  backupApi,
  type BackupItem,
  type BackupContentType,
  type BackupFormat,
  type CompressionLevel,
  type BackupSettings,
} from "@api/backup";
import { i18n } from "@language";
import { useLoading } from "@composables/useAsync";
import { useMessage } from "@composables/useMessage";
import SLButton from "@components/common/SLButton.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLSpinner from "@components/common/SLSpinner.vue";
import SLTooltip from "@components/common/SLTooltip.vue";
import SLCheckbox from "@components/common/SLCheckbox.vue";
import { Archive, RotateCcw, Trash2, Clock, Package, Gauge } from "lucide-vue-next";
import "@styles/views/BackupView.css";

const serverStore = useServerStore();
const { loading, withLoading } = useLoading();
const { error, success, showError, showSuccess } = useMessage();

const backups = ref<BackupItem[]>([]);
const settings = ref<BackupSettings>({
  maxBackups: 5,
  autoBackupEnabled: false,
  autoBackupInterval: 24,
  autoBackupContents: ["core", "config", "world"],
  defaultFormat: "zip",
  compressionLevel: "medium",
});

const selectedContents = ref<BackupContentType[]>(["core", "config", "world"]);
const selectedFormat = ref<BackupFormat>("zip");
const selectedCompression = ref<CompressionLevel>("medium");
const creatingBackup = ref(false);
const restoringId = ref<string | null>(null);
const deletingId = ref<string | null>(null);

const selectedServerId = computed(() => serverStore.currentServerId || "");
const selectedServerName = computed(() => {
  const server = serverStore.servers.find((s) => s.id === selectedServerId.value);
  return server?.name || "";
});

const contentOptions: { value: BackupContentType; labelKey: string }[] = [
  { value: "core", labelKey: "backup.content_core" },
  { value: "config", labelKey: "backup.content_config" },
  { value: "plugins", labelKey: "backup.content_plugins" },
  { value: "world", labelKey: "backup.content_world" },
  { value: "logs", labelKey: "backup.content_logs" },
];

const formatOptions = [
  { value: "zip", label: "ZIP" },
  { value: "tar.gz", label: "TAR.GZ" },
];

const compressionOptions = [
  { value: "low", label: i18n.t("backup.compression_low") },
  { value: "medium", label: i18n.t("backup.compression_medium") },
  { value: "high", label: i18n.t("backup.compression_high") },
];

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString();
}

async function loadBackups() {
  if (!selectedServerId.value) return;
  await withLoading(async () => {
    try {
      backups.value = await backupApi.list(selectedServerId.value);
    } catch {
      showError(i18n.t("backup.load_failed"));
    }
  });
}

async function loadSettings() {
  if (!selectedServerId.value) return;
  try {
    const s = await backupApi.getSettings(selectedServerId.value);
    settings.value = s;
    selectedFormat.value = s.defaultFormat;
    selectedCompression.value = s.compressionLevel;
    selectedContents.value = [...s.autoBackupContents];
  } catch {
    // 使用默认值
  }
}

async function createBackup() {
  if (!selectedServerId.value || selectedContents.value.length === 0) return;
  creatingBackup.value = true;
  try {
    const backup = await backupApi.create({
      serverId: selectedServerId.value,
      contents: selectedContents.value,
      format: selectedFormat.value,
      compressionLevel: selectedCompression.value,
    });
    backups.value.unshift(backup);
    // 超出最大数量删除旧备份
    if (backups.value.length > settings.value.maxBackups) {
      const toDelete = backups.value.slice(settings.value.maxBackups);
      for (const b of toDelete) {
        try {
          await backupApi.delete(b.id);
        } catch {}
      }
      backups.value = backups.value.slice(0, settings.value.maxBackups);
    }
    showSuccess(i18n.t("backup.create_success"));
  } catch {
    showError(i18n.t("backup.create_failed"));
  } finally {
    creatingBackup.value = false;
  }
}

async function restoreBackup(backup: BackupItem) {
  restoringId.value = backup.id;
  try {
    await backupApi.restore(backup.id);
    showSuccess(i18n.t("backup.restore_success"));
  } catch {
    showError(i18n.t("backup.restore_failed"));
  } finally {
    restoringId.value = null;
  }
}

async function deleteBackup(backup: BackupItem) {
  deletingId.value = backup.id;
  try {
    await backupApi.delete(backup.id);
    backups.value = backups.value.filter((b) => b.id !== backup.id);
    showSuccess(i18n.t("backup.delete_success"));
  } catch {
    showError(i18n.t("backup.delete_failed"));
  } finally {
    deletingId.value = null;
  }
}

async function updateSettings() {
  if (!selectedServerId.value) return;
  // 同步备份内容到自动备份设置
  settings.value.autoBackupContents = [...selectedContents.value];
  settings.value.defaultFormat = selectedFormat.value;
  settings.value.compressionLevel = selectedCompression.value;
  try {
    await backupApi.updateSettings(selectedServerId.value, settings.value);
    showSuccess(i18n.t("backup.settings_saved"));
  } catch {
    showError(i18n.t("backup.settings_save_failed"));
  }
}

function toggleContent(value: BackupContentType, enabled: boolean) {
  if (enabled && !selectedContents.value.includes(value)) {
    selectedContents.value.push(value);
  } else if (!enabled) {
    selectedContents.value = selectedContents.value.filter((c) => c !== value);
  }
  updateSettings();
}

onMounted(async () => {
  await serverStore.refreshList();
  if (!serverStore.currentServerId && serverStore.servers.length > 0) {
    serverStore.setCurrentServer(serverStore.servers[0].id);
  }
  if (serverStore.currentServerId) {
    await loadBackups();
    await loadSettings();
  }
});

watch(
  () => serverStore.currentServerId,
  async () => {
    if (serverStore.currentServerId) {
      await loadBackups();
      await loadSettings();
    }
  },
);
</script>

<template>
  <div class="backup-view animate-stagger-in">
    <!-- 标题区 -->
    <div class="backup-header">
      <h1 class="backup-title">
        {{ i18n.t("backup.title") }}
        <span v-if="selectedServerName" class="backup-server-name">{{ selectedServerName }}</span>
      </h1>
    </div>

    <!-- 操作区：立即备份 + 自动备份开关 -->
    <div class="backup-actions glass-strong">
      <div class="backup-actions-row">
        <SLButton
          variant="primary"
          :loading="creatingBackup"
          :disabled="selectedContents.length === 0 || !selectedServerId"
          class="backup-create-btn"
          @click="createBackup"
        >
          <Archive :size="16" />
          {{ i18n.t("backup.create_now") }}
        </SLButton>
        <div class="backup-auto-toggle">
          <span>{{ i18n.t("backup.auto_backup") }}</span>
          <SLSwitch
            :modelValue="settings.autoBackupEnabled"
            @update:modelValue="
              (v: boolean) => {
                settings.autoBackupEnabled = v;
                updateSettings();
              }
            "
          />
        </div>
      </div>

      <!-- 自动备份间隔（开关打开时显示） -->
      <div class="sl-collapse" :class="{ 'sl-collapse--expanded': settings.autoBackupEnabled }">
        <div class="sl-collapse__content">
          <div class="backup-interval-row">
            <label>{{ i18n.t("backup.auto_interval") }}</label>
            <div class="backup-interval-input">
              <input
                v-model.number="settings.autoBackupInterval"
                type="number"
                min="1"
                max="720"
                class="backup-number-input"
                @change="updateSettings"
              />
              <span>{{ i18n.t("backup.hours") }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 备份设置区 -->
    <div class="backup-settings glass-strong">
      <div class="backup-settings-header">{{ i18n.t("backup.settings_title") }}</div>
      <div class="backup-settings-grid">
        <div class="backup-setting-item">
          <label>{{ i18n.t("backup.max_backups") }}</label>
          <input
            v-model.number="settings.maxBackups"
            type="number"
            min="1"
            max="50"
            class="backup-number-input"
            @change="updateSettings"
          />
        </div>
        <div class="backup-setting-item">
          <label>
            <Package :size="14" />
            {{ i18n.t("backup.format") }}
          </label>
          <SLSelect
            :modelValue="selectedFormat"
            :options="formatOptions"
            size="sm"
            class="backup-select"
            @update:modelValue="
              (v) => {
                selectedFormat = v as BackupFormat;
                updateSettings();
              }
            "
          />
        </div>
        <div class="backup-setting-item">
          <label>
            <Gauge :size="14" />
            {{ i18n.t("backup.compression") }}
          </label>
          <SLSelect
            :modelValue="selectedCompression"
            :options="compressionOptions"
            size="sm"
            class="backup-select"
            @update:modelValue="
              (v) => {
                selectedCompression = v as CompressionLevel;
                updateSettings();
              }
            "
          />
        </div>
      </div>

      <!-- 备份内容选择 -->
      <div class="backup-contents-section">
        <div class="backup-contents-label">{{ i18n.t("backup.select_contents") }}</div>
        <div class="backup-checkbox-row">
          <label v-for="opt in contentOptions" :key="opt.value" class="backup-checkbox-item">
            <SLCheckbox
              :modelValue="selectedContents.includes(opt.value)"
              @update:modelValue="(v: boolean) => toggleContent(opt.value, v)"
            />
            <span>{{ i18n.t(opt.labelKey) }}</span>
          </label>
        </div>
      </div>
    </div>

    <!-- 备份列表 -->
    <div class="backup-list-section glass-strong">
      <div class="backup-list-header">{{ i18n.t("backup.list_title") }}</div>
      <div v-if="loading" class="backup-loading">
        <SLSpinner size="md" />
      </div>
      <div v-else-if="backups.length === 0" class="backup-empty">
        {{ i18n.t("backup.empty") }}
      </div>
      <div v-else class="backup-list">
        <div v-for="backup in backups" :key="backup.id" class="backup-item">
          <div class="backup-item-main">
            <div class="backup-item-name">{{ backup.name }}</div>
            <div class="backup-item-meta">
              <span class="backup-tag">{{ backup.format.toUpperCase() }}</span>
              <span>{{ formatSize(backup.size) }}</span>
              <span class="backup-time">
                <Clock :size="12" />
                {{ formatDate(backup.createdAt) }}
              </span>
            </div>
            <div class="backup-item-contents">
              <span v-for="c in backup.contents" :key="c" class="backup-content-tag">
                {{ i18n.t(`backup.content_${c}`) }}
              </span>
            </div>
          </div>
          <div class="backup-item-actions">
            <SLTooltip :content="i18n.t('backup.restore')">
              <SLButton
                variant="secondary"
                size="sm"
                iconOnly
                :loading="restoringId === backup.id"
                :disabled="restoringId !== null || deletingId !== null"
                @click="restoreBackup(backup)"
              >
                <RotateCcw :size="16" />
              </SLButton>
            </SLTooltip>
            <SLTooltip :content="i18n.t('backup.delete')">
              <SLButton
                variant="secondary"
                size="sm"
                iconOnly
                :loading="deletingId === backup.id"
                :disabled="restoringId !== null || deletingId !== null"
                class="backup-delete-btn"
                @click="deleteBackup(backup)"
              >
                <Trash2 :size="16" />
              </SLButton>
            </SLTooltip>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
