import { ref, type Ref } from "vue";
import { handleError } from "@utils/errorHandler";

/**
 * 异步操作状态
 */
export interface AsyncState<T> {
  loading: Ref<boolean>;
  error: Ref<string | null>;
  data: Ref<T | null>;
}

/**
 * 异步操作选项
 */
export interface AsyncOptions {
  /** 错误时是否抛出异常 */
  throwError?: boolean;
  /** 错误上下文（用于日志） */
  context?: string;
  /** 成功回调 */
  onSuccess?: (data: any) => void;
  /** 错误回调 */
  onError?: (error: string) => void;
  /** 完成回调（无论成功或失败） */
  onFinally?: () => void;
}

/**
 * 基础异步操作 composable
 * 用于管理单个异步操作的状态
 */
export function useAsync<T>(
  asyncFn: () => Promise<T>,
  options: AsyncOptions = {},
): AsyncState<T> & { execute: () => Promise<T | null> } {
  const loading = ref(false);
  const error = ref<string | null>(null);
  const data = ref<T | null>(null) as Ref<T | null>;

  const execute = async (): Promise<T | null> => {
    loading.value = true;
    error.value = null;

    try {
      const result = await asyncFn();
      data.value = result;
      options.onSuccess?.(result);
      return result;
    } catch (e) {
      const errorMessage = handleError(e, options.context);
      error.value = errorMessage;
      options.onError?.(errorMessage);
      if (options.throwError) {
        throw e;
      }
      return null;
    } finally {
      loading.value = false;
      options.onFinally?.();
    }
  };

  return {
    loading,
    error,
    data,
    execute,
  };
}

/**
 * 多个异步操作的状态管理
 * 用于管理多个不同操作的加载状态（如 startLoading, stopLoading 等）
 */
export function useAsyncActions<T extends Record<string, () => Promise<any>>>(
  actions: T,
): {
  loading: Record<keyof T, Ref<boolean>>;
  error: Ref<string | null>;
  execute: <K extends keyof T>(key: K) => Promise<ReturnType<T[K]> | null>;
} {
  const loading = {} as Record<keyof T, Ref<boolean>>;
  const error = ref<string | null>(null);

  for (const key of Object.keys(actions) as (keyof T)[]) {
    loading[key] = ref(false);
  }

  const execute = async <K extends keyof T>(key: K): Promise<ReturnType<T[K]> | null> => {
    const action = actions[key];
    if (!action) {
      console.error(`Action "${String(key)}" not found`);
      return null;
    }

    loading[key].value = true;
    error.value = null;

    try {
      return await action();
    } catch (e) {
      error.value = handleError(e, String(key));
      return null;
    } finally {
      loading[key].value = false;
    }
  };

  return {
    loading,
    error,
    execute,
  };
}

/**
 * 带键的异步操作状态管理
 * 用于管理多个相同类型操作的加载状态（如多个服务器的启动/停止）
 */
export function useAsyncByKey<K extends string = string>() {
  const loading = ref<Record<K, boolean>>({} as Record<K, boolean>);
  const error = ref<string | null>(null);

  /**
   * 执行异步操作
   */
  async function execute<T>(
    key: K,
    asyncFn: () => Promise<T>,
    options: Omit<AsyncOptions, "context"> = {},
  ): Promise<T | null> {
    loading.value[key] = true;
    error.value = null;

    try {
      const result = await asyncFn();
      options.onSuccess?.(result);
      return result;
    } catch (e) {
      const errorMessage = handleError(e, String(key));
      error.value = errorMessage;
      options.onError?.(errorMessage);
      if (options.throwError) {
        throw e;
      }
      return null;
    } finally {
      loading.value[key] = false;
      options.onFinally?.();
    }
  }

  /**
   * 检查指定键是否正在加载
   */
  function isLoading(key: K): boolean {
    return loading.value[key] ?? false;
  }

  /**
   * 设置指定键的加载状态
   */
  function setLoading(key: K, value: boolean): void {
    loading.value[key] = value;
  }

  /**
   * 清除错误
   */
  function clearError(): void {
    error.value = null;
  }

  return {
    loading,
    error,
    execute,
    isLoading,
    setLoading,
    clearError,
  };
}

/**
 * 简单的加载状态管理
 * 用于只需要管理加载状态的场景
 */
export function useLoading(initialState = false) {
  const loading = ref(initialState);

  function start(): void {
    loading.value = true;
  }

  function stop(): void {
    loading.value = false;
  }

  function toggle(): void {
    loading.value = !loading.value;
  }

  async function withLoading<T>(asyncFn: () => Promise<T>): Promise<T> {
    loading.value = true;
    try {
      return await asyncFn();
    } finally {
      loading.value = false;
    }
  }

  return {
    loading,
    start,
    stop,
    toggle,
    withLoading,
  };
}
