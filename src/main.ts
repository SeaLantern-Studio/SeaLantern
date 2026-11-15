/**
 * 桌面端有两个独立入口：
 * - 主窗口走完整的 App / Router / Pinia 初始化；
 * - 桌宠窗口使用轻量的 PetWindow，只加载桌宠需要的 Vue 模块与样式。
 *
 * 两个窗口共用同一个 Vite 页面，通过 query 参数区分，避免为桌宠再维护
 * 一套 HTML 入口，同时保证主窗口不会把桌宠的拖拽、轮询逻辑一起加载。
 */
const isPetWindow = new URLSearchParams(window.location.search).get("window") === "pet";

if (isPetWindow) {
  void import("./pet/main");
} else {
  void import("./app");
}
