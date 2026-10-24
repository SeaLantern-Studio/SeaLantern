import { ref, type Ref } from "vue";

// 需要 keep-alive 的页面组件 name（Vue SFC 文件名 PascalCase）。
// 只缓存有交互状态的页面；静态页/一次性页不缓存，避免白白占用内存。
const KEEP_ALIVE_VIEWS = [
  "HomeView",
  "ConsoleView",
  "ConfigView",
  "PlayerView",
  "PluginsView",
  "ResourceMarketView",
  "SettingsView",
  "TunnelView",
] as const;

// 全局单例：当前 keep-alive 实际缓存的组件 name 列表。
// 用模块级 ref 保证所有调用方共享同一份缓存状态。
const cachedViews: Ref<string[]> = ref([...KEEP_ALIVE_VIEWS]);

export function useKeepAliveCache() {
  /** 从缓存中移除某个页面，keep-alive 会销毁其实例（如删除服务器后清控制台缓存） */
  function removeCache(name: string): void {
    cachedViews.value = cachedViews.value.filter((n) => n !== name);
  }

  /** 恢复缓存某个页面 */
  function addCache(name: string): void {
    if (!cachedViews.value.includes(name)) {
      cachedViews.value.push(name);
    }
  }

  /** 重置为初始缓存列表 */
  function resetCache(): void {
    cachedViews.value = [...KEEP_ALIVE_VIEWS];
  }

  return { cachedViews, removeCache, addCache, resetCache };
}

export { KEEP_ALIVE_VIEWS };
