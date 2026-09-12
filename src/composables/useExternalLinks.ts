import { openUrl } from "@tauri-apps/plugin-opener";
import { isBrowserEnv } from "@api/tauri";

/** 判断是否为需要交给系统浏览器处理的绝对外链（http/https） */
function isExternalHref(href: string | null): href is string {
  return !!href && /^https?:\/\//i.test(href);
}

/**
 * 外部链接统一处理
 *
 * Tauri 的 WebView 中，点击 Markdown 渲染出的 `<a href="https://...">` 会在
 * WebView 内部直接导航，整个 SPA 页面被替换掉（必须 history.back 才能回来）。
 * 这里提供容器级点击委托，拦截这类外链并改用系统浏览器打开。
 *
 * 说明：带 `target="_blank"` 的链接虽走新窗口逻辑，但统一拦截同样安全，
 * 可保证桌面端与浏览器（Docker）行为一致。
 */
export function useExternalLinks() {
  /** 在系统浏览器（桌面端）或新标签页（浏览器端）打开链接 */
  async function openLink(href: string) {
    if (!href) return;

    if (isBrowserEnv()) {
      window.open(href, "_blank", "noopener,noreferrer");
      return;
    }

    try {
      await openUrl(href);
    } catch (error) {
      console.error("[useExternalLinks] 打开外部链接失败:", error);
    }
  }

  /**
   * 点击事件委托：只拦截绝对外链，锚点（#xxx）、相对路径与 Vue Router
   * 生成的内部链接保持默认行为。
   *
   * 用法：`<div @click="handleLinkClick"> ...渲染后的 Markdown... </div>`
   */
  function handleLinkClick(event: MouseEvent) {
    if (event.defaultPrevented || event.button !== 0) return;
    // 带修饰键（Ctrl/Cmd 等）时保留默认的新窗口/新标签页行为
    if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;

    const target = event.target;
    if (!(target instanceof Element)) return;

    const anchor = target.closest("a[href]");
    if (!anchor) return;

    const href = anchor.getAttribute("href");
    if (!isExternalHref(href)) return;

    event.preventDefault();
    void openLink(href);
  }

  return { openLink, handleLinkClick };
}
