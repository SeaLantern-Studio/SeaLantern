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
import { Archive, Folder } from "lucide-vue-next";
import SLButton from "@components/common/SLButton.vue";
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
const dragging = ref(false);

const archiveExtensions = [".zip", ".tar", ".tar.gz", ".tgz"];

const selectedName = computed(() => getPathName(props.sourcePath));
const sourceTypeText = computed(() => {
  if (props.sourceType === "archive") {
    return i18n.t("create.source_kind_archive");
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

function clearSource(event?: Event) {
  event?.stopPropagation();
  setSource("", "");
}

function extractPathFromDrop(event: DragEvent): string | null {
  const file = event.dataTransfer?.files?.[0];
  if (!file) {
    return null;
  }

  const fileWithPath = file as File & { path?: string; webkitRelativePath?: string };
  if (fileWithPath.path && fileWithPath.path.length > 0) {
    return fileWithPath.path;
  }

  if (fileWithPath.webkitRelativePath && fileWithPath.webkitRelativePath.length > 0) {
    return fileWithPath.webkitRelativePath;
  }

  return null;
}

function handleDrop(event: DragEvent) {
  event.preventDefault();
  if (props.disabled) {
    dragging.value = false;
    return;
  }

  dragging.value = false;
  const droppedPath = extractPathFromDrop(event);
  if (!droppedPath) {
    emit("error", i18n.t("create.source_drop_unavailable"));
    return;
  }

  if (hasArchiveExtension(droppedPath)) {
    setSource(droppedPath, "archive");
    return;
  }

  setSource(droppedPath, "folder");
}

function openChooser() {
  if (props.disabled) {
    return;
  }
  chooserOpen.value = true;
}

async function pickArchive() {
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
    <button
      type="button"
      class="source-intake-dropzone"
      :class="{ dragging, selected: !!sourcePath, disabled }"
      :disabled="disabled"
      @click="openChooser"
      @dragenter.prevent="dragging = true"
      @dragover.prevent="dragging = true"
      @dragleave.prevent="dragging = false"
      @drop="handleDrop"
    >
      <div class="source-intake-icon">+</div>
      <div class="source-intake-main">
        <p class="source-intake-title" :class="{ selected: !!sourcePath }">
          {{ sourcePath ? selectedName : i18n.t("create.source_drop_or_click") }}
        </p>
        <p v-if="!sourcePath" class="source-intake-subtitle">
          {{ i18n.t("create.source_accept_hint") }}
        </p>
      </div>
      <div class="source-intake-meta">
        <span class="source-intake-badge">{{ sourceTypeText }}</span>
        <SLButton v-if="sourcePath" variant="ghost" size="sm" @click="clearSource($event)">
          {{ i18n.t("create.source_clear") }}
        </SLButton>
      </div>
    </button>

    <DialogRoot v-model:open="chooserOpen">
      <DialogPortal>
        <DialogOverlay class="source-chooser-overlay" />
        <DialogContent class="source-chooser-content">
          <DialogTitle class="source-chooser-title">{{ i18n.t("create.source_choose_title") }}</DialogTitle>
          <DialogDescription class="source-chooser-description">
            {{ i18n.t("create.source_choose_description") }}
          </DialogDescription>
          <div class="source-chooser-actions">
            <SLButton variant="primary" size="lg" class="source-chooser-option" @click="pickArchive">
              <Archive :size="22" />
              <span>{{ i18n.t("create.source_pick_archive") }}</span>
            </SLButton>
            <SLButton variant="secondary" size="lg" class="source-chooser-option" @click="pickFolder">
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
