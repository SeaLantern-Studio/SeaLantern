import { ref } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";

export function useAboutLinks() {
  const copiedQQ = ref<string | null>(null);

  async function openLink(url: string) {
    if (!url) return;
    try {
      await openUrl(url);
    } catch (e) {
      console.error("[useAboutLinks] 打开URL失败:", e);
    }
  }

  async function copyQQ(qq: string) {
    try {
      await navigator.clipboard.writeText(qq);
      copiedQQ.value = qq;
      setTimeout(() => {
        copiedQQ.value = null;
      }, 2000);
    } catch (e) {
      console.error("[useAboutLinks] 复制QQ失败:", e);
    }
  }

  async function openSocialLink(platform: string, value: string) {
    if (platform === "qq") {
      await copyQQ(value);
    } else {
      await openLink(value);
    }
  }

  return {
    copiedQQ,
    openLink,
    copyQQ,
    openSocialLink,
  };
}
