import { ref } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { i18n } from "@language";
import { useGlobalMessage } from "@composables/useMessage";

async function openExternalLink(url: string) {
  if (!url) return;
  try {
    await openUrl(url);
  } catch (error) {
    console.error("[useAboutLinks] 打开URL失败:", error);
  }
}

export function useAboutLinks() {
  const copiedQQ = ref<string | null>(null);
  const globalMessage = useGlobalMessage();

  async function copyQQ(qq: string) {
    try {
      await navigator.clipboard.writeText(qq);
      copiedQQ.value = qq;
      globalMessage.success(i18n.t("about.copied"));
      setTimeout(() => {
        copiedQQ.value = null;
      }, 2000);
    } catch (e) {
      console.error("[useAboutLinks] 复制QQ失败:", e);
      globalMessage.error(String(e));
    }
  }

  async function openSocialLink(platform: string, value: string) {
    if (platform === "qq") {
      await copyQQ(value);
    } else {
      await openExternalLink(value);
    }
  }

  return {
    copiedQQ,
    openLink: openExternalLink,
    copyQQ,
    openSocialLink,
  };
}
