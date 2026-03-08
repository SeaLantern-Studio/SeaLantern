/**
 * Docker环境下的文件上传API
 * 用于在浏览器/Docker模式下替代原生文件选择器
 */

import { HTTP_API_BASE } from "./tauri";

export interface UploadedFile {
  original_name: string;
  saved_path: string;
  size: number;
}

export interface UploadResult {
  files: UploadedFile[];
  count: number;
}

/**
 * 上传单个文件
 */
export async function uploadFile(file: File): Promise<UploadedFile> {
  const formData = new FormData();
  formData.append("file", file);

  const response = await fetch(`${HTTP_API_BASE}/upload`, {
    method: "POST",
    body: formData,
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Upload failed: ${errorText}`);
  }

  const result = await response.json();

  if (!result.success) {
    throw new Error(result.error || "Upload failed");
  }

  return result.data.files[0];
}

/**
 * 上传多个文件
 */
export async function uploadFiles(files: File[]): Promise<UploadResult> {
  const formData = new FormData();
  files.forEach((file) => formData.append("files", file));

  const response = await fetch(`${HTTP_API_BASE}/upload`, {
    method: "POST",
    body: formData,
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Upload failed: ${errorText}`);
  }

  const result = await response.json();

  if (!result.success) {
    throw new Error(result.error || "Upload failed");
  }

  return result.data;
}

/**
 * 从文件输入元素上传文件
 */
export async function uploadFromInput(
  inputElement: HTMLInputElement,
): Promise<UploadedFile | null> {
  const files = inputElement.files;
  if (!files || files.length === 0) {
    return null;
  }

  return uploadFile(files[0]);
}

/**
 * 从拖拽事件上传文件
 */
export async function uploadFromDropEvent(event: DragEvent): Promise<UploadedFile[]> {
  event.preventDefault();
  event.stopPropagation();

  const files: File[] = [];

  if (event.dataTransfer?.items) {
    // 使用 DataTransferItemList 接口
    for (let i = 0; i < event.dataTransfer.items.length; i++) {
      const item = event.dataTransfer.items[i];
      if (item.kind === "file") {
        const file = item.getAsFile();
        if (file) {
          files.push(file);
        }
      }
    }
  } else if (event.dataTransfer?.files) {
    // 使用 DataTransfer.files 接口
    for (let i = 0; i < event.dataTransfer.files.length; i++) {
      files.push(event.dataTransfer.files[i]);
    }
  }

  if (files.length === 0) {
    return [];
  }

  const result = await uploadFiles(files);
  return result.files;
}

/**
 * 检测当前环境是否支持上传（Docker/浏览器模式）
 */
export function isUploadSupported(): boolean {
  return typeof window !== "undefined" && !window.__TAURI__;
}
