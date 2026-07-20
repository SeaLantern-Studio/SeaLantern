<script setup lang="ts">
import { ref, computed } from "vue";
import { uploadFile, isUploadSupported } from "@api/upload";

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
const isUploading = ref(false);
const uploadProgress = ref(0);

// 计算属性：是否支持上传
const canUpload = computed(() => isUploadSupported());

// 将 accept 字符串转为 fileExtensions 数组供 cmz-dropzone 使用
const fileExtensions = computed(() => {
  if (!props.accept) return [];
  return props.accept
    .split(",")
    .map((t) => t.trim())
    .filter((t) => t.startsWith("."));
});

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

// 上传选中的文件
const uploadSelectedFiles = async (files: File[]) => {
  if (!canUpload.value) {
    emit("error", "当前环境不支持文件上传");
    return;
  }

  isUploading.value = true;
  uploadProgress.value = 0;

  const acceptedTypes = props.accept?.split(",").map((type) => type.trim()) ?? [];
  const validFiles = files.filter((file) => {
    if (props.maxSize && file.size > props.maxSize) {
      emit("error", `文件 ${file.name} 超过大小限制 (${props.maxSize} 字节)`);
      return false;
    }

    if (acceptedTypes.length > 0) {
      const fileExtension = `.${file.name.split(".").pop()}`;
      const isAccepted = acceptedTypes.some(
        (type) => file.name.includes(type) || fileExtension === type,
      );
      if (!isAccepted) {
        emit("error", `文件类型不支持: ${file.name}`);
        return false;
      }
    }

    return true;
  });

  try {
    const uploadedFiles = await Promise.all(validFiles.map((file) => uploadFile(file)));
    uploadedFiles.forEach((uploadedFile) => {
      emit("uploaded", uploadedFile);
    });
    uploadProgress.value = validFiles.length > 0 ? 100 : 0;
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
    <cmz-dropzone
      v-if="showDropZone && canUpload"
      :placeholder="'拖拽文件到此处，或点击选择文件'"
      :multiple="multiple ?? false"
      :file-extensions="fileExtensions"
      :loading="isUploading"
      :accept-folders="false"
      :clearable="false"
      @click="handleClick"
      @error="(msg: string) => emit('error', msg)"
    />

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
