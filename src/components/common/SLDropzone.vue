<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, computed, watch } from "vue";
import { X, FileUp, Loader2 } from "lucide-vue-next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { i18n } from "@language";
import { isUploadSupported, uploadFromDropEvent } from "@api/upload";

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
const nativeDroppedPaths = ref<string[]>([]);
let unlistenNativeDragDrop: UnlistenFn | null = null;
const DROPZONE_DEBUG = import.meta.env.DEV;

function logDropzone(message: string, payload?: unknown) {
  if (!DROPZONE_DEBUG) return;
  if (payload === undefined) {
    console.debug(message);
    return;
  }
  console.debug(message, payload);
}

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
  const dataTransfer = event.dataTransfer;
  if (!dataTransfer) {
    logDropzone("[SLDropzone] Drop event missing dataTransfer");
    return [];
  }

  const paths: string[] = [];

  if (dataTransfer.files && dataTransfer.files.length > 0) {
    logDropzone("[SLDropzone] Inspecting dataTransfer.files", {
      count: dataTransfer.files.length,
    });
    for (let i = 0; i < dataTransfer.files.length; i++) {
      const fileWithPath = dataTransfer.files[i] as File & {
        path?: string;
        webkitRelativePath?: string;
      };
      logDropzone("[SLDropzone] dataTransfer.files item", {
        index: i,
        name: fileWithPath.name,
        path: fileWithPath.path,
        webkitRelativePath: fileWithPath.webkitRelativePath,
      });
      if (fileWithPath.path && fileWithPath.path.length > 0) {
        paths.push(fileWithPath.path);
      } else if (fileWithPath.webkitRelativePath && fileWithPath.webkitRelativePath.length > 0) {
        paths.push(fileWithPath.webkitRelativePath);
      }
    }
  }

  if (paths.length > 0) {
    logDropzone("[SLDropzone] Resolved drop paths from dataTransfer.files", paths);
    return paths;
  }

  if (dataTransfer.items && dataTransfer.items.length > 0) {
    logDropzone("[SLDropzone] Inspecting dataTransfer.items", {
      count: dataTransfer.items.length,
    });
    for (let i = 0; i < dataTransfer.items.length; i++) {
      const item = dataTransfer.items[i];
      if (item.kind !== "file") {
        logDropzone("[SLDropzone] Skipping non-file drop item", {
          index: i,
          kind: item.kind,
          type: item.type,
        });
        continue;
      }

      const file = item.getAsFile() as
        | (File & { path?: string; webkitRelativePath?: string })
        | null;
      logDropzone("[SLDropzone] dataTransfer.items file", {
        index: i,
        name: file?.name,
        path: file?.path,
        webkitRelativePath: file?.webkitRelativePath,
      });
      if (!file) {
        continue;
      }

      if (file.path && file.path.length > 0) {
        paths.push(file.path);
      } else if (file.webkitRelativePath && file.webkitRelativePath.length > 0) {
        paths.push(file.webkitRelativePath);
      }
    }
  }

  if (paths.length > 0) {
    logDropzone("[SLDropzone] Resolved drop paths from dataTransfer.items", paths);
  } else {
    logDropzone("[SLDropzone] No usable paths resolved from DOM drop event");
  }

  return paths;
}

function getDroppedPaths(event: DragEvent): string[] {
  const domPaths = extractPathsFromDrop(event);
  if (domPaths.length > 0) {
    return domPaths;
  }

  if (nativeDroppedPaths.value.length > 0) {
    logDropzone(
      "[SLDropzone] Falling back to native Tauri drag-drop paths",
      nativeDroppedPaths.value,
    );
    return [...nativeDroppedPaths.value];
  }

  return [];
}

onMounted(async () => {
  logDropzone("[SLDropzone] mounted", {
    hasTauriInternals: !!window.__TAURI_INTERNALS__,
    uploadSupported: isUploadSupported(),
  });

  if (isUploadSupported()) {
    logDropzone("[SLDropzone] Upload mode detected, skip native drag-drop listener");
    return;
  }

  try {
    const currentWindow = getCurrentWindow();
    logDropzone("[SLDropzone] Preparing native drag-drop listener");
    unlistenNativeDragDrop = await currentWindow.onDragDropEvent((event) => {
      logDropzone("[SLDropzone] Native Tauri drag-drop event", event.payload);
      if (event.payload.type === "enter" || event.payload.type === "over") {
        internalDragging.value = true;
      } else if (event.payload.type === "drop") {
        internalDragging.value = false;
        nativeDroppedPaths.value = [...event.payload.paths];
      } else {
        internalDragging.value = false;
        nativeDroppedPaths.value = [];
      }
    });
    logDropzone("[SLDropzone] Native drag-drop listener registered");
  } catch (error) {
    logDropzone("[SLDropzone] Failed to register native drag-drop listener", error);
  }
});

onBeforeUnmount(() => {
  if (unlistenNativeDragDrop) {
    unlistenNativeDragDrop();
    unlistenNativeDragDrop = null;
  }
});

function handleDragEnter(event: DragEvent) {
  event.preventDefault();
  if (props.disabled || props.loading) return;
  logDropzone("[SLDropzone] DOM dragenter", {
    fileCount: event.dataTransfer?.files?.length ?? 0,
    itemCount: event.dataTransfer?.items?.length ?? 0,
  });
  internalDragging.value = true;
}

function handleDragOver(event: DragEvent) {
  event.preventDefault();
  if (props.disabled || props.loading) return;
  internalDragging.value = true;
}

function handleDragLeave(event: DragEvent) {
  event.preventDefault();
  logDropzone("[SLDropzone] DOM dragleave");
  internalDragging.value = false;
}

async function handleDrop(event: DragEvent) {
  event.preventDefault();
  event.stopPropagation();
  internalDragging.value = false;

  if (props.disabled || props.loading) return;

  logDropzone("[SLDropzone] handleDrop triggered", {
    hasDataTransfer: !!event.dataTransfer,
    fileCount: event.dataTransfer?.files?.length ?? 0,
    itemCount: event.dataTransfer?.items?.length ?? 0,
    nativeDroppedPaths: nativeDroppedPaths.value,
  });

  // Docker/浏览器环境：通过HTTP上传文件
  if (isUploadSupported()) {
    try {
      const uploadedFiles = await uploadFromDropEvent(event);
      if (uploadedFiles.length === 0) {
        logDropzone("[SLDropzone] Upload mode drop produced no uploaded files");
        emit("error", i18n.t("dropzone.error_no_path"));
        return;
      }

      const validPaths = uploadedFiles
        .filter((f) => {
          const isFile = hasAcceptedExtension(f.saved_path);
          if (isFile && !props.acceptFiles) return false;
          if (!isFile && !props.acceptFolders) return false;
          return true;
        })
        .map((f) => f.saved_path);

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
    } catch (err) {
      emit("error", err instanceof Error ? err.message : "Upload failed");
    }
    return;
  }

  const droppedPaths = getDroppedPaths(event);
  logDropzone("[SLDropzone] Final dropped paths for Tauri", {
    nativeDroppedPaths: nativeDroppedPaths.value,
    chosenPaths: droppedPaths,
  });
  nativeDroppedPaths.value = [];
  if (droppedPaths.length === 0) {
    logDropzone("[SLDropzone] Tauri drop produced no usable paths");
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
    logDropzone("[SLDropzone] All dropped paths were filtered out", {
      droppedPaths,
      acceptFiles: props.acceptFiles,
      acceptFolders: props.acceptFolders,
      fileExtensions: props.fileExtensions,
    });
    emit("error", i18n.t("dropzone.error_unsupported_type"));
    return;
  }

  logDropzone("[SLDropzone] Accepted dropped paths", validPaths);
  if (props.multiple && validPaths.length > 1) {
    emit("dropMultiple", validPaths);
  } else {
    emit("update:modelValue", validPaths[0]);
    emit("drop", validPaths[0]);
  }
}

function handleClick() {
  if (props.disabled || props.loading) return;
  logDropzone("[SLDropzone] Click triggered chooser open");
  emit("click");
}

function handleClear(event: Event) {
  event.stopPropagation();
  nativeDroppedPaths.value = [];
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
