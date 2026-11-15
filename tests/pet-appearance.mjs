import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const css = await readFile(new URL("../src/styles/pet.css", import.meta.url), "utf8");
const petEntry = await readFile(new URL("../src/pet/main.ts", import.meta.url), "utf8");

test("桌宠使用静态海晶灯贴图，不再播放序列帧", () => {
  assert.match(
    css,
    /\.sea-texture\s*\{[\s\S]*?background-size:\s*100% 100%;[\s\S]*?background-position:\s*center;/,
  );
  assert.doesNotMatch(css, /lantern-frames|background-position-y/);
});

test("眼睛颜色取自海晶灯贴图最高饱和色", () => {
  assert.match(css, /--pet-eye:\s*rgb\(115, 172, 160\)/);
  assert.match(css, /\.eye-shape\s*\{[\s\S]*?background:\s*var\(--pet-eye\)/);
  assert.doesNotMatch(css, /#43ddff|#52e4ff|#56e5ff/);
});

test("桌宠气泡和菜单复用客户端主题 token", () => {
  assert.match(petEntry, /@styles\/variables\.css/);
  assert.match(css, /\.pet-bubble\s*\{[\s\S]*?background:\s*var\(--pet-panel-bg\)/);
  assert.match(css, /\.pet-menu,\s*\n\.pet-submenu\s*\{[\s\S]*?color:\s*var\(--sl-text-primary\)/);
  assert.match(css, /\.menu-item:[\s\S]*?background:\s*var\(--sl-surface-hover\)/);
});

test("眼睛本身不绘制外发光", () => {
  const eyeStyles = css.slice(css.indexOf(".eye-motion"), css.indexOf(".music-note"));
  assert.ok(eyeStyles.length > 100, "眼睛样式段应存在");
  assert.doesNotMatch(eyeStyles, /box-shadow/);
  assert.doesNotMatch(eyeStyles, /drop-shadow/);
  assert.doesNotMatch(eyeStyles, /filter:/);
});

test("通透模式播放本地朋友酒 DJ 版", async () => {
  const audio = await readFile(new URL("../src/pet/audio.ts", import.meta.url), "utf8");
  assert.match(audio, /friends-wine-dj\.mp3/);
  assert.match(audio, /new Audio\(friendsWineDjUrl\)/);
  assert.match(audio, /audio\.loop = true/);
  assert.doesNotMatch(audio, /setInterval|onBeat/);
});

test("通透模式由 Rust 接管桌面图标层并遵循原版参数", async () => {
  const physics = await readFile(new URL("../src/pet/physics.ts", import.meta.url), "utf8");
  const api = await readFile(new URL("../src/pet/api.ts", import.meta.url), "utf8");
  const desktopJelly = await readFile(
    new URL("../src-tauri/src/desktop/desktop_jelly.rs", import.meta.url),
    "utf8",
  );
  assert.match(physics, /JELLY_SPEED_CYCLES_PER_SECOND = 1\.1/);
  assert.match(physics, /JELLY_STRENGTH = 0\.38/);
  assert.match(api, /desktop_jelly_start/);
  assert.match(api, /desktop_jelly_stop/);
  assert.match(desktopJelly, /SPEED_CYCLES_PER_SECOND: f64 = 1\.1/);
  assert.match(desktopJelly, /STRENGTH: f64 = 0\.38/);
  assert.match(desktopJelly, /SetParent/);
  assert.match(desktopJelly, /ShowWindow\(list, SW_HIDE\)/);
  assert.match(desktopJelly, /ShowWindow\(list, SW_SHOW\)/);
});
