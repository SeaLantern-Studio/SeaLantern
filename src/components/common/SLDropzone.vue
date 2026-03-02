<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { X, FileUp, Loader2 } from "lucide-vue-next";
import { i18n } from "@language";

export interface DropzoneProps {
  modelValue?: string;
  label?: string;
  subLabel?: string;
  badge?: string;
  disabled?: boolean;
  loading?: boolean;
  isDragging?: boolean;
  acceptFolders?: boolean;
  acceptFiles?: boolean;
  fileExtensions?: string[];
  placeholder?: string;
  clearable?: boolean;
  multiple?: boolean;
}

const props = withDefaults(defineProps<DropzoneProps>(), {
  modelValue: "",
  label: "",
  subLabel: "",
  badge: "",
  disabled: false,
  loading: false,
  isDragging: undefined,
  acceptFolders: true,
  acceptFiles: true,
  fileExtensions: () => [".zip", ".tar", ".tar.gz", ".tgz", ".jar"],
  placeholder: "",
  clearable: true,
  multiple: false,
});

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "drop", path: string): void;
  (e: "dropMultiple", paths: string[]): void;
  (e: "clear"): void;
  (e: "click"): void;
  (e: "error", message: string): void;
  (e: "update:isDragging", value: boolean): void;
}>();

const internalDragging = ref(false);

const isDraggingState = computed(() => {
  if (props.isDragging !== undefined) {
    return props.isDragging;
  }
  return internalDragging.value;
});

watch(internalDragging, (val) => {
  emit("update:isDragging", val);
});

const displayLabel = computed(() => {
  if (props.modelValue) {
    return props.label || getPathName(props.modelValue);
  }
  return props.placeholder || i18n.t("dropzone.placeholder");
});

const displaySubLabel = computed(() => {
  if (props.modelValue) {
    return props.subLabel || "";
  }
  if (props.acceptFiles && props.acceptFolders) {
    return i18n.t("dropzone.support_both");
  }
  if (props.acceptFiles) {
    return i18n.t("dropzone.support_files");
  }
  if (props.acceptFolders) {
    return i18n.t("dropzone.support_folders");
  }
  return "";
});

function getPathName(path: string): string {
  const segments = path.split(/[\\/]/).filter(Boolean);
  return segments.length > 0 ? segments[segments.length - 1] : path;
}

function hasAcceptedExtension(path: string): boolean {
  if (!props.fileExtensions || props.fileExtensions.length === 0) {
    return true;
  }
  const lowerPath = path.toLowerCase();
  return props.fileExtensions.some((ext) => lowerPath.endsWith(ext.toLowerCase()));
}

function extractPathsFromDrop(event: DragEvent): string[] {
  const files = event.dataTransfer?.files;
  if (!files || files.length === 0) {
    return [];
  }

  const paths: string[] = [];
  for (let i = 0; i < files.length; i++) {
    const fileWithPath = files[i] as File & { path?: string; webkitRelativePath?: string };
    if (fileWithPath.path && fileWithPath.path.length > 0) {
      paths.push(fileWithPath.path);
    } else if (fileWithPath.webkitRelativePath && fileWithPath.webkitRelativePath.length > 0) {
      paths.push(fileWithPath.webkitRelativePath);
    }
  }

  return paths;
}

function handleDragEnter(event: DragEvent) {
  event.preventDefault();
  if (props.disabled || props.loading) return;
  internalDragging.value = true;
}

function handleDragOver(event: DragEvent) {
  event.preventDefault();
  if (props.disabled || props.loading) return;
  internalDragging.value = true;
}

function handleDragLeave(event: DragEvent) {
  event.preventDefault();
  internalDragging.value = false;
}

function handleDrop(event: DragEvent) {
  event.preventDefault();
  internalDragging.value = false;

  if (props.disabled || props.loading) return;

  const droppedPaths = extractPathsFromDrop(event);
  if (droppedPaths.length === 0) {
    emit("error", i18n.t("dropzone.error_no_path"));
    return;
  }

  const validPaths = droppedPaths.filter((path) => {
    const isFile = hasAcceptedExtension(path);
    if (isFile && !props.acceptFiles) return false;
    if (!isFile && !props.acceptFolders) return false;
    return true;
  });

  if (validPaths.length === 0) {
    emit("error", i18n.t("dropzone.error_unsupported_type"));
    return;
  }

  if (props.multiple && validPaths.length > 1) {
    emit("dropMultiple", validPaths);
  } else {
    emit("update:modelValue", validPaths[0]);
    emit("drop", validPaths[0]);
  }
}

function handleClick() {
  if (props.disabled || props.loading) return;
  emit("click");
}

function handleClear(event: Event) {
  event.stopPropagation();
  emit("update:modelValue", "");
  emit("clear");
}
</script>

<template>
  <div class="sl-dropzone-wrapper">
    <button
      type="button"
      class="sl-dropzone"
      :class="{
        dragging: isDraggingState,
        selected: !!modelValue,
        disabled,
        loading,
      }"
      :disabled="disabled || loading"
      @click="handleClick"
      @dragenter="handleDragEnter"
      @dragover="handleDragOver"
      @dragleave="handleDragLeave"
      @drop="handleDrop"
    >
      <div class="sl-dropzone-icon">
        <slot name="icon">
          <Loader2 v-if="loading" :size="20" class="sl-dropzone-spinner" />
          <FileUp v-else :size="20" />
        </slot>
      </div>

      <div class="sl-dropzone-content">
        <p class="sl-dropzone-title" :class="{ selected: !!modelValue }">
          <slot name="title">{{ displayLabel }}</slot>
        </p>
        <p v-if="displaySubLabel && !$slots.subtitle" class="sl-dropzone-subtitle">
          {{ displaySubLabel }}
        </p>
        <slot name="subtitle" />
      </div>

      <div class="sl-dropzone-actions">
        <span v-if="badge" class="sl-dropzone-badge">{{ badge }}</span>
        <button
          v-if="modelValue && clearable && !loading"
          type="button"
          class="sl-dropzone-clear"
          @click="handleClear"
        >
          <X :size="14" />
        </button>
      </div>
    </button>

    <slot name="footer" />

    <div v-if="$slots.buttons" class="sl-dropzone-buttons">
      <slot name="buttons" />
    </div>
  </div>
</template>

<style src="@styles/components/common/SLDropzone.css" scoped></style>
