<script setup lang="ts">
import { ref, computed } from "vue";
import { uploadFile, uploadFromDropEvent, isUploadSupported } from "@api/upload";

const props = defineProps<{
  /** 是否显示拖拽区域 */
  showDropZone?: boolean;
  /** 接受的文件类型 */
  accept?: string;
  /** 是否多选 */
  multiple?: boolean;
  /** 最大文件大小（字节） */
  maxSize?: number;
}>();

const emit = defineEmits<{
  (e: "uploaded", file: { original_name: string; saved_path: string; size: number }): void;
  (e: "error", message: string): void;
}>();

const fileInput = ref<HTMLInputElement | null>(null);
const isDragging = ref(false);
const isUploading = ref(false);
const uploadProgress = ref(0);

// 计算属性：是否支持上传
const canUpload = computed(() => isUploadSupported());

// 计算属性：拖拽区域样式
const dropZoneClass = computed(() => ({
  "border-dashed": true,
  "border-2": true,
  "rounded-lg": true,
  "p-8": true,
  "text-center": true,
  "transition-colors": true,
  "border-gray-300": !isDragging.value,
  "border-blue-500": isDragging.value,
  "bg-blue-50": isDragging.value,
}));

// 点击上传按钮
const handleClick = () => {
  if (!canUpload.value) {
    emit("error", "当前环境不支持文件上传");
    return;
  }
  fileInput.value?.click();
};

// 文件选择变化
const handleFileChange = async (event: Event) => {
  const input = event.target as HTMLInputElement;
  const files = input.files;
  if (!files || files.length === 0) return;

  await uploadSelectedFiles(Array.from(files));
  input.value = ""; // 重置输入
};

// 处理拖拽开始
const handleDragStart = () => {
  if (!canUpload.value) return;
  isDragging.value = true;
};

// 处理拖拽结束
const handleDragEnd = () => {
  isDragging.value = false;
};

// 处理拖拽放置
const handleDrop = async (event: DragEvent) => {
  if (!canUpload.value) {
    event.preventDefault();
    event.stopPropagation();
    emit("error", "当前环境不支持文件上传");
    return;
  }

  isDragging.value = false;

  try {
    const uploadedFiles = await uploadFromDropEvent(event);
    uploadedFiles.forEach((file) => {
      emit("uploaded", file);
    });
  } catch (error) {
    const message = error instanceof Error ? error.message : "上传失败";
    emit("error", message);
  }
};

// 上传选中的文件
const uploadSelectedFiles = async (files: File[]) => {
  if (!canUpload.value) {
    emit("error", "当前环境不支持文件上传");
    return;
  }

  isUploading.value = true;
  uploadProgress.value = 0;

  try {
    for (let i = 0; i < files.length; i++) {
      const file = files[i];

      // 检查文件大小
      if (props.maxSize && file.size > props.maxSize) {
        emit("error", `文件 ${file.name} 超过大小限制 (${props.maxSize} 字节)`);
        continue;
      }

      // 检查文件类型
      if (props.accept) {
        const acceptedTypes = props.accept.split(",").map((t) => t.trim());
        const fileExtension = `.${file.name.split(".").pop()}`;
        if (!acceptedTypes.some((type) => file.name.includes(type) || fileExtension === type)) {
          emit("error", `文件类型不支持: ${file.name}`);
          continue;
        }
      }

      const uploadedFile = await uploadFile(file);
      emit("uploaded", uploadedFile);
      uploadProgress.value = Math.round(((i + 1) / files.length) * 100);
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : "上传失败";
    emit("error", message);
  } finally {
    isUploading.value = false;
    uploadProgress.value = 0;
  }
};

// 触发文件选择
const triggerFileSelect = () => {
  handleClick();
};

// 暴露方法给父组件
defineExpose({
  triggerFileSelect,
});
</script>

<template>
  <div>
    <!-- 文件输入（隐藏） -->
    <input
      ref="fileInput"
      type="file"
      :accept="accept"
      :multiple="multiple"
      class="hidden"
      @change="handleFileChange"
    />

    <!-- 拖拽上传区域 -->
    <div
      v-if="showDropZone && canUpload"
      :class="dropZoneClass"
      @click="handleClick"
      @dragenter.prevent="handleDragStart"
      @dragover.prevent="handleDragStart"
      @dragleave.prevent="handleDragEnd"
      @drop="handleDrop"
    >
      <div class="flex flex-col items-center gap-2">
        <svg class="w-12 h-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
          />
        </svg>
        <p class="text-gray-600">
          {{ isDragging ? "释放文件以上传" : "拖拽文件到此处，或点击选择文件" }}
        </p>
        <p v-if="accept" class="text-sm text-gray-400">支持的格式: {{ accept }}</p>
        <p v-if="maxSize" class="text-sm text-gray-400">
          最大大小: {{ (maxSize / 1024 / 1024).toFixed(2) }} MB
        </p>
      </div>
    </div>

    <!-- 上传进度 -->
    <div v-if="isUploading" class="mt-4">
      <div class="flex items-center justify-between mb-2">
        <span class="text-sm text-gray-600">上传中...</span>
        <span class="text-sm text-gray-600">{{ uploadProgress }}%</span>
      </div>
      <div class="w-full bg-gray-200 rounded-full h-2">
        <div
          class="bg-blue-500 h-2 rounded-full transition-all"
          :style="{ width: uploadProgress + '%' }"
        ></div>
      </div>
    </div>

    <!-- 不支持上传的提示 -->
    <div v-if="!canUpload" class="p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
      <p class="text-yellow-700 text-sm">
        当前环境不支持文件上传。请使用桌面版应用或确保在浏览器/Docker模式下运行。
      </p>
    </div>
  </div>
</template>
