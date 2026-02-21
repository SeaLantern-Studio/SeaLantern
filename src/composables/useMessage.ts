import { ref, onUnmounted } from "vue";
import { TIME } from "../utils/constants";

/**
 * 消息类型
 */
export type MessageType = "error" | "success" | "warning" | "info";

/**
 * 消息项接口
 */
export interface MessageItem {
  id: number;
  type: MessageType;
  message: string;
}

let nextId = 0;

/**
 * 统一消息提示 composable
 * 用于在组件中显示错误、成功等提示消息
 */
export function useMessage(defaultDuration: number = TIME.ERROR_MESSAGE_DURATION) {
  const error = ref<string | null>(null);
  const success = ref<string | null>(null);
  const warning = ref<string | null>(null);
  const info = ref<string | null>(null);

  let errorTimer: ReturnType<typeof setTimeout> | null = null;
  let successTimer: ReturnType<typeof setTimeout> | null = null;
  let warningTimer: ReturnType<typeof setTimeout> | null = null;
  let infoTimer: ReturnType<typeof setTimeout> | null = null;

  /**
   * 清除指定类型的定时器
   */
  function clearTimer(type: MessageType) {
    switch (type) {
      case "error":
        if (errorTimer) {
          clearTimeout(errorTimer);
          errorTimer = null;
        }
        break;
      case "success":
        if (successTimer) {
          clearTimeout(successTimer);
          successTimer = null;
        }
        break;
      case "warning":
        if (warningTimer) {
          clearTimeout(warningTimer);
          warningTimer = null;
        }
        break;
      case "info":
        if (infoTimer) {
          clearTimeout(infoTimer);
          infoTimer = null;
        }
        break;
    }
  }

  /**
   * 显示错误消息
   */
  function showError(message: string, duration?: number): void {
    clearTimer("error");
    error.value = message;
    const actualDuration = duration ?? defaultDuration;
    if (actualDuration > 0) {
      errorTimer = setTimeout(() => {
        error.value = null;
      }, actualDuration);
    }
  }

  /**
   * 显示成功消息
   */
  function showSuccess(message: string, duration?: number): void {
    clearTimer("success");
    success.value = message;
    const actualDuration = duration ?? TIME.SUCCESS_MESSAGE_DURATION;
    if (actualDuration > 0) {
      successTimer = setTimeout(() => {
        success.value = null;
      }, actualDuration);
    }
  }

  /**
   * 显示警告消息
   */
  function showWarning(message: string, duration?: number): void {
    clearTimer("warning");
    warning.value = message;
    const actualDuration = duration ?? defaultDuration;
    if (actualDuration > 0) {
      warningTimer = setTimeout(() => {
        warning.value = null;
      }, actualDuration);
    }
  }

  /**
   * 显示信息消息
   */
  function showInfo(message: string, duration?: number): void {
    clearTimer("info");
    info.value = message;
    const actualDuration = duration ?? defaultDuration;
    if (actualDuration > 0) {
      infoTimer = setTimeout(() => {
        info.value = null;
      }, actualDuration);
    }
  }

  /**
   * 清除所有消息
   */
  function clearAll(): void {
    clearTimer("error");
    clearTimer("success");
    clearTimer("warning");
    clearTimer("info");
    error.value = null;
    success.value = null;
    warning.value = null;
    info.value = null;
  }

  /**
   * 清除指定类型的消息
   */
  function clear(type: MessageType): void {
    clearTimer(type);
    switch (type) {
      case "error":
        error.value = null;
        break;
      case "success":
        success.value = null;
        break;
      case "warning":
        warning.value = null;
        break;
      case "info":
        info.value = null;
        break;
    }
  }

  /**
   * 清除错误消息（便捷方法）
   */
  function clearError(): void {
    clear("error");
  }

  onUnmounted(() => {
    clearTimer("error");
    clearTimer("success");
    clearTimer("warning");
    clearTimer("info");
  });

  return {
    error,
    success,
    warning,
    info,
    showError,
    showSuccess,
    showWarning,
    showInfo,
    clear,
    clearError,
    clearAll,
  };
}

/**
 * 全局消息列表（用于 Toast 样式的消息）
 */
const messages = ref<MessageItem[]>([]);
const messageTimers = new Map<number, ReturnType<typeof setTimeout>>();

/**
 * 清理指定消息的定时器
 */
function clearMessageTimer(id: number) {
  const timer = messageTimers.get(id);
  if (timer) {
    clearTimeout(timer);
    messageTimers.delete(id);
  }
}

/**
 * 添加消息到全局列表
 */
function addMessage(type: MessageType, message: string, duration: number = 3000): number {
  const id = nextId++;
  messages.value.push({ id, type, message });

  if (duration > 0) {
    const timer = setTimeout(() => {
      removeMessage(id);
      messageTimers.delete(id);
    }, duration);
    messageTimers.set(id, timer);
  }

  return id;
}

/**
 * 从全局列表移除消息
 */
function removeMessage(id: number): void {
  clearMessageTimer(id);
  const index = messages.value.findIndex((m) => m.id === id);
  if (index > -1) {
    messages.value.splice(index, 1);
  }
}

/**
 * 全局消息管理（Toast 样式）
 */
export function useGlobalMessage() {
  return {
    messages,
    remove: removeMessage,
    error: (message: string, duration?: number) => addMessage("error", message, duration),
    success: (message: string, duration?: number) => addMessage("success", message, duration),
    warning: (message: string, duration?: number) => addMessage("warning", message, duration),
    info: (message: string, duration?: number) => addMessage("info", message, duration),
  };
}
