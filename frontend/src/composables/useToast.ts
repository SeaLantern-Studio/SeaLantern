import { useGlobalMessage, type MessageItem, type MessageType } from "./useMessage";

export type ToastType = MessageType;

export type ToastItem = MessageItem;

const DEFAULT_TOAST_DURATION = 3000;

export function useToast() {
  const globalMessage = useGlobalMessage();

  return {
    toasts: globalMessage.messages,
    removeToast: globalMessage.remove,
    success: (message: string, duration?: number) =>
      globalMessage.success(message, duration ?? DEFAULT_TOAST_DURATION),
    error: (message: string, duration?: number) =>
      globalMessage.error(message, duration ?? DEFAULT_TOAST_DURATION),
    warning: (message: string, duration?: number) =>
      globalMessage.warning(message, duration ?? DEFAULT_TOAST_DURATION),
    info: (message: string, duration?: number) =>
      globalMessage.info(message, duration ?? DEFAULT_TOAST_DURATION),
  };
}
