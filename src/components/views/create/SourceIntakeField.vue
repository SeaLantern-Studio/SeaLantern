<script setup lang="ts">
import { computed, ref, watch, onMounted } from "vue";
import {
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";
import { File, Folder, Plus, X, Upload } from "lucide-vue-next";
import SLButton from "@components/common/SLButton.vue";
import SLDropzone from "@components/common/SLDropzone.vue";
import SLSelect from "@components/common/SLSelect.vue";
import { systemApi } from "@api/system";
import { downloadServerApi } from "@api/downloader";
import { i18n } from "@language";

export type SourceType = "archive" | "folder" | "download" | "";

const props = withDefaults(
  defineProps<{
    sourcePath: string;
    sourceType: SourceType;
    disabled?: boolean;
  }>(),
  {
    disabled: false,
  },
);

const emit = defineEmits<{
  (e: "update:sourcePath", value: string): void;
  (e: "update:sourceType", value: SourceType): void;
  (e: "update:serverDownloadType", value: string): void;
  (e: "update:serverDownloadVersion", value: string): void;
  (e: "error", value: string): void;
}>();

/* ── 本地文件选择 ── */
const chooserOpen = ref(false);

const archiveExtensions = [".zip", ".tar", ".tar.gz", ".tgz", ".jar"];

const selectedName = computed(() => getPathName(props.sourcePath));
const sourceTypeText = computed(() => {
  if (props.sourceType === "archive") return i18n.t("create.source_kind_file");
  if (props.sourceType === "folder") return i18n.t("create.source_kind_folder");
  if (props.sourceType === "download") return i18n.t("create.source_kind_download");
  return i18n.t("create.source_not_selected");
});

// 是否已选择本地文件（archive 或 folder）
const hasLocalSource = computed(
  () => props.sourceType === "archive" || props.sourceType === "folder",
);

function getPathName(path: string): string {
  const segments = path.split(/[\\/]/).filter(Boolean);
  return segments.length > 0 ? segments[segments.length - 1] : path;
}

function hasArchiveExtension(path: string): boolean {
  const lowerPath = path.toLowerCase();
  return archiveExtensions.some((ext) => lowerPath.endsWith(ext));
}

function setSource(path: string, type: SourceType) {
  emit("update:sourcePath", path);
  emit("update:sourceType", type);
}

function handleDrop(path: string) {
  if (hasArchiveExtension(path)) {
    setSource(path, "archive");
  } else {
    setSource(path, "folder");
  }
}

function handleError(message: string) {
  emit("error", message);
}

function handleClearLocal() {
  setSource("", "");
}

function openChooser() {
  if (props.disabled) return;
  chooserOpen.value = true;
}

async function pickFile() {
  chooserOpen.value = false;
  const selected = await systemApi.pickArchiveFile();
  if (selected) {
    setSource(selected, "archive");
  }
}

async function pickFolder() {
  chooserOpen.value = false;
  const selected = await systemApi.pickFolder();
  if (selected) {
    setSource(selected, "folder");
  }
}

/* ── 服务端选择 ── */
const serverTypes = ref<string[]>([]);
const versions = ref<string[]>([]);
const selectedType = ref("");
const selectedVersion = ref("");

const loadingTypes = ref(false);
const loadingVersions = ref(false);

const serverTypeOptions = computed(() =>
  serverTypes.value.map((type) => ({ label: type, value: type })),
);

const versionOptions = computed(() => {
  return [...versions.value]
    .toSorted((a, b) => {
      const aParts = a.split(".").map(Number);
      const bParts = b.split(".").map(Number);
      for (let i = 0; i < Math.max(aParts.length, bParts.length); i++) {
        const aNum = aParts[i] || 0;
        const bNum = bParts[i] || 0;
        if (bNum - aNum !== 0) return bNum - aNum;
      }
      return 0;
    })
    .map((v) => ({ label: v, value: v }));
});

// 选完类型+版本后，进入下载模式
const downloadReady = computed(() => selectedType.value && selectedVersion.value);

watch(downloadReady, (ready) => {
  // 如果已选择了本地文件，不要覆盖
  if (hasLocalSource.value) return;
  if (ready) {
    emit("update:sourceType", "download");
    emit("update:serverDownloadType", selectedType.value);
    emit("update:serverDownloadVersion", selectedVersion.value);
  } else if (props.sourceType === "download") {
    emit("update:sourceType", "");
  }
});

/* ── 加载服务端类型 ── */
async function loadServerTypes() {
  loadingTypes.value = true;
  try {
    const types = await downloadServerApi.getServerTypes();
    serverTypes.value = types;
    if (types.length > 0) selectedType.value = types[0];
  } catch (e) {
    emit("error", String(e));
  } finally {
    loadingTypes.value = false;
  }
}

/* ── 加载版本列表 ── */
async function loadVersions(serverType: string) {
  if (!serverType) return;
  loadingVersions.value = true;
  versions.value = [];
  selectedVersion.value = "";

  try {
    const list = await downloadServerApi.getVersionsByType(serverType);
    versions.value = list;
    if (list.length > 0) selectedVersion.value = list[list.length - 1];
  } catch (e) {
    emit("error", String(e));
  } finally {
    loadingVersions.value = false;
  }
}

/* ── 类型联动 ── */
watch(selectedType, (type) => {
  if (type) loadVersions(type);
});

/* ── 初始化 ── */
onMounted(async () => {
  await loadServerTypes();
});
</script>

<template>
  <div class="source-intake-step">
    <!-- 已选择本地文件时：显示文件名+清除按钮 -->
    <template v-if="hasLocalSource">
      <SLDropzone
        :model-value="sourcePath"
        :label="selectedName"
        :badge="sourceTypeText"
        :disabled="disabled"
        :file-extensions="archiveExtensions"
        :placeholder="i18n.t('create.source_drop_or_click')"
        @click="openChooser"
        @drop="handleDrop"
        @clear="handleClearLocal"
        @error="handleError"
      >
        <template #icon>
          <Plus :size="20" stroke-width="2.5" />
        </template>
      </SLDropzone>
    </template>

    <!-- 未选择本地文件时：显示服务端选择器 -->
    <template v-else>
      <div class="server-download-panel">
        <div class="server-download-row">
          <div class="server-download-field">
            <label>{{ i18n.t("downloadServerView.form.type") }}</label>
            <SLSelect
              :model-value="selectedType"
              :options="serverTypeOptions"
              :placeholder="i18n.t('downloadServerView.form.typePlaceholder')"
              :disabled="loadingTypes"
              :loading="loadingTypes"
              searchable
              maxHeight="240px"
              @update:modelValue="selectedType = $event"
            />
          </div>
          <div class="server-download-field">
            <label>{{ i18n.t("downloadServerView.form.version") }}</label>
            <SLSelect
              :model-value="selectedVersion"
              :options="versionOptions"
              :placeholder="i18n.t('downloadServerView.form.versionPlaceholder')"
              :disabled="loadingVersions || !selectedType"
              :loading="loadingVersions"
              searchable
              maxHeight="240px"
              @update:modelValue="selectedVersion = $event"
            />
          </div>
        </div>
      </div>

      <!-- 添加自定义服务端按钮 -->
      <button class="custom-source-btn" @click="chooserOpen = true">
        <Upload :size="14" />
        <span>{{ i18n.t("create.add_custom_source") }}</span>
      </button>
    </template>

    <!-- 选择文件对话框 -->
    <DialogRoot v-model:open="chooserOpen">
      <DialogPortal>
        <DialogOverlay class="source-chooser-overlay" />
        <DialogContent class="source-chooser-content">
          <div class="source-chooser-header">
            <DialogTitle class="source-chooser-title">{{
              i18n.t("create.source_choose_title")
            }}</DialogTitle>
            <button
              class="source-chooser-close"
              @click="chooserOpen = false"
              :aria-label="i18n.t('common.close_modal')"
            >
              <X :size="18" />
            </button>
          </div>
          <DialogDescription class="source-chooser-description">
            {{ i18n.t("create.source_choose_description_file") }}
          </DialogDescription>
          <div class="source-chooser-actions">
            <SLButton variant="primary" size="lg" class="source-chooser-option" @click="pickFile">
              <File :size="22" />
              <span>{{ i18n.t("create.source_pick_file") }}</span>
            </SLButton>
            <SLButton
              variant="secondary"
              size="lg"
              class="source-chooser-option"
              @click="pickFolder"
            >
              <Folder :size="22" />
              <span>{{ i18n.t("create.source_pick_folder") }}</span>
            </SLButton>
          </div>
        </DialogContent>
      </DialogPortal>
    </DialogRoot>
  </div>
</template>

<style src="@styles/components/views/create/SourceIntakeField.css" scoped></style>
