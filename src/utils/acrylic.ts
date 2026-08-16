// 窗口透明靠系统原生亚克力合成模糊,强度不可调,CSS 侧固定写死 medium 档
// 统一通过 --sl-acrylic-blur 下发,供窗口内浮层/toast 等磨玻璃元素使用
const ACRYLIC_BLUR = "16px";

export function applyAcrylicEffect(enabled: boolean): void {
  const root = document.documentElement;

  root.setAttribute("data-acrylic", enabled ? "on" : "off");

  // 关闭时清理模糊属性,避免残留值误导其他依赖方
  if (!enabled) {
    root.removeAttribute("data-acrylic-blur");
    root.style.removeProperty("--sl-acrylic-blur");
    return;
  }

  root.setAttribute("data-acrylic-blur", "medium");
  root.style.setProperty("--sl-acrylic-blur", ACRYLIC_BLUR);
}
