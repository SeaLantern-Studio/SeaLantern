<script setup lang="ts">
import { computed, ref } from "vue";
import {
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";
import { File, Folder, Plus, X } from "lucide-vue-next";
import SLButton from "@components/common/SLButton.vue";
import SLDropzone from "@components/common/SLDropzone.vue";
import { systemApi } from "@api/system";
import { i18n } from "@language";

export type SourceType = "archive" | "folder" | "";

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
  (e: "error", value: string): void;
}>();

const chooserOpen = ref(false);

const archiveExtensions = [".zip", ".tar", ".tar.gz", ".tgz", ".jar"];

const selectedName = computed(() => getPathName(props.sourcePath));
const sourceTypeText = computed(() => {
  if (props.sourceType === "archive") {
    return i18n.t("create.source_kind_file");
  }
  if (props.sourceType === "folder") {
    return i18n.t("create.source_kind_folder");
  }
  return i18n.t("create.source_not_selected");
});

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

function handleClear() {
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
</script>

<template>
  <div class="source-intake-step">
    <SLDropzone
      :model-value="sourcePath"
      :label="selectedName"
      :badge="sourceTypeText"
      :disabled="disabled"
      :file-extensions="archiveExtensions"
      :placeholder="i18n.t('create.source_drop_or_click')"
      @click="openChooser"
      @drop="handleDrop"
      @clear="handleClear"
      @error="handleError"
    >
      <template #icon>
        <Plus :size="20" stroke-width="2.5" />
      </template>
    </SLDropzone>

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
