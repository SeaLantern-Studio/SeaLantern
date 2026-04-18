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
  // Tauri v2 默认不注入 window.__TAURI__，使用 __TAURI_INTERNALS__ 可靠判断
  return typeof window !== "undefined" && !window.__TAURI_INTERNALS__;
}

/**
 * 浏览器文件选择器选项
 */
export interface BrowserFilePickerOptions {
  /** 接受的文件类型，如 '.jar,.zip' */
  accept?: string;
  /** 是否允许多选 */
  multiple?: boolean;
}

/**
 * 使用浏览器原生文件选择器选择文件
 * @param options 选择器选项
 * @returns 选择的文件或文件数组，取消则返回 null
 */
export function pickFileFromBrowser(
  options?: BrowserFilePickerOptions,
): Promise<File | File[] | null> {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.style.display = "none";
    if (options?.accept) {
      input.accept = options.accept;
    }
    if (options?.multiple) {
      input.multiple = true;
    }
    input.onchange = () => {
      const files = input.files;
      if (!files || files.length === 0) {
        resolve(null);
        return;
      }
      // 清理 DOM
      document.body.removeChild(input);
      if (options?.multiple) {
        resolve(Array.from(files));
      } else {
        resolve(files[0]);
      }
    };
    // 处理用户取消的情况（某些浏览器支持）
    input.oncancel = () => {
      document.body.removeChild(input);
      resolve(null);
    };
    document.body.appendChild(input);
    input.click();
  });
}
