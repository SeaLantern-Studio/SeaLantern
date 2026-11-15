import { createApp } from "vue";
import PetWindow from "@components/pet/PetWindow.vue";
import "@styles/variables.css";
import "@styles/pet.css";

// 桌宠是独立 WebView，不会经过 AppLayout。这里复用主窗口缓存的主题与字号，
// 让气泡、右键菜单和客户端保持同一套设计 token。
try {
  const cached = JSON.parse(localStorage.getItem("sl_theme_cache") || "{}") as {
    theme?: string;
    fontSize?: number;
  };
  const effectiveTheme =
    cached.theme === "light" || cached.theme === "dark"
      ? cached.theme
      : window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light";
  document.documentElement.setAttribute("data-theme", effectiveTheme);
  document.documentElement.style.fontSize = `${cached.fontSize || 14}px`;
} catch {
  document.documentElement.setAttribute(
    "data-theme",
    window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light",
  );
}

createApp(PetWindow).mount("#app");
