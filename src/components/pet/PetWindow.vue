<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import {
  Activity,
  AppWindow,
  Check,
  ChevronRight,
  EyeOff,
  Music2,
  RefreshCw,
  Server,
  Sparkles,
  Volume2,
  X,
} from "lucide-vue-next";
import {
  calculateEyeOffset,
  createEyeSpringState,
  stepEyeSpring,
  type EyeMotion,
  type EyeOffset,
} from "@src/pet/eyeTracking";
import { PET_EXPRESSIONS, type PetExpression } from "@src/pet/expressions";
import {
  calculateJellyTransform,
  createPetSpringState,
  nudgePetSpring,
  resetPetSpringState,
  stepPetSpring,
  type PetSpringTransform,
} from "@src/pet/physics";
import { PetAudioEngine } from "@src/pet/audio";
import { petApi, type PetServerStatus } from "@src/pet/api";
import seaLanternTexture from "@assets/pet/sea-lantern.png";

type BubbleKind = "status" | "meme" | "notice";
type MenuSide = "left" | "right";

const isPreview = petApi.isPreview();
const previewKind = isPreview
  ? new URLSearchParams(window.location.search).get("preview") ||
    window.location.hash.slice(1) ||
    null
  : null;
const petBlockRef = ref<HTMLElement | null>(null);
const eyeTarget = ref<EyeOffset>({
  x: 0,
  y: 0,
  rotation: 0,
  intensity: 0,
  tracking: false,
  distance: 0,
});
const eyeSpringState = createEyeSpringState();
const eyeMotion = ref<EyeMotion>({ x: 0, y: 0, rotation: 0, intensity: 0 });
const currentExpression = ref<PetExpression>("neutral");
const transparentMode = ref(false);
const audioError = ref("");
const menuOpen = ref(false);
const menuSide = ref<MenuSide>("left");
const menuLeft = ref(20);
const menuTop = ref(20);
const actionsOpen = ref(false);
const expressionsOpen = ref(false);
const bubbleKind = ref<BubbleKind | null>(null);
const bubbleMessage = ref("");
const statuses = ref<PetServerStatus[]>([]);
const statusLoading = ref(false);
const statusError = ref("");
const audio = new PetAudioEngine();
const springState = createPetSpringState();
const springTransform = ref<PetSpringTransform>({
  x: 0,
  y: 0,
  scaleX: 1,
  scaleY: 1,
  rotation: 0,
});
const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

const seaLanternStyle = {
  "--sea-texture": `url("${seaLanternTexture}")`,
};
const eyeMotionStyle = computed(() => ({
  "--eye-x": `${eyeMotion.value.x.toFixed(2)}px`,
  "--eye-y": `${eyeMotion.value.y.toFixed(2)}px`,
  "--eye-rotation": `${eyeMotion.value.rotation.toFixed(2)}deg`,
}));
const currentExpressionLabel = computed(
  () => PET_EXPRESSIONS.find((item) => item.id === currentExpression.value)?.label ?? "平常心",
);
const runningCount = computed(
  () => statuses.value.filter((item) => item.status === "Running").length,
);
const statusSummary = computed(() => {
  if (!statuses.value.length) return "暂无服务器";
  return `${runningCount.value}/${statuses.value.length} 台运行中`;
});
// 子菜单是 primary menu 的 fixed 子元素；这里给的是相对 primary 的偏移。
// 表情菜单再相对动作菜单展开，避免三级菜单把窗口撑出屏幕。
const actionSubmenuStyle = computed(() => ({
  top: "6px",
  left: menuSide.value === "left" ? "163px" : "-165px",
}));
const expressionSubmenuStyle = { top: "6px" };
const physicsStyle = computed(() => ({
  transform: `translate3d(${springTransform.value.x.toFixed(2)}px, ${springTransform.value.y.toFixed(2)}px, 0) rotate(${springTransform.value.rotation.toFixed(2)}deg) scale(${springTransform.value.scaleX.toFixed(4)}, ${springTransform.value.scaleY.toFixed(4)})`,
}));

const particleClasses = ["particle-a", "particle-b", "particle-c", "particle-d", "particle-e"];

let previewExpression = currentExpression.value;
let cursorTimer: ReturnType<typeof setTimeout> | null = null;
let statusTimer: ReturnType<typeof setInterval> | null = null;
let disposed = false;
let cursorPolling = false;
let clickThroughActive: boolean | null = null;
let physicsFrame: number | null = null;
let lastPhysicsTime = 0;
let danceStartedAt = 0;
let windowGeometry = {
  originX: 0,
  originY: 0,
  scaleFactor: 1,
  updatedAt: 0,
  ready: false,
};

async function setClickThrough(ignore: boolean) {
  if (clickThroughActive === ignore) return;
  clickThroughActive = ignore;
  try {
    await petApi.setClickThrough(ignore);
  } catch (error) {
    console.debug("桌宠切换鼠标穿透失败:", error);
  }
}

/** 点击、换表情、鼓点等交互统一走这套弹簧冲量。 */
function pulsePet(strength = 1, direction = 1) {
  if (reduceMotion) return;
  // 通透模式只允许纵向 Q 弹，任何交互都不能引入左右晃动。
  nudgePetSpring(springState, strength, transparentMode.value ? 0 : direction);
}

function handlePetPointerEnter() {
  if (!transparentMode.value) pulsePet(0.24, 1);
}

function startPetAnimationLoop() {
  lastPhysicsTime = performance.now();
  const tick = (now: number) => {
    const delta = Math.min((now - lastPhysicsTime) / 1000, 0.033);
    lastPhysicsTime = now;
    if (transparentMode.value) {
      // 原版 DesktopJelly 的 Motion.Scale：底边固定，只改变宽高比例。
      const jelly = calculateJellyTransform((now - danceStartedAt) / 1000);
      springTransform.value = {
        x: 0,
        y: 0,
        scaleX: jelly.scaleX,
        scaleY: jelly.scaleY,
        rotation: 0,
      };
    } else if (!reduceMotion) {
      springTransform.value = stepPetSpring(springState, delta, now / 1000, false);
    }
    if (!reduceMotion) {
      eyeMotion.value = stepEyeSpring(eyeSpringState, eyeTarget.value, delta);
    } else {
      eyeMotion.value = {
        x: eyeTarget.value.x,
        y: eyeTarget.value.y,
        rotation: eyeTarget.value.rotation,
        intensity: eyeTarget.value.intensity,
      };
    }
    physicsFrame = requestAnimationFrame(tick);
  };
  physicsFrame = requestAnimationFrame(tick);
}

function closeBubble() {
  bubbleKind.value = null;
  stopStatusPolling();
}

function stopStatusPolling() {
  if (statusTimer) {
    clearInterval(statusTimer);
    statusTimer = null;
  }
}

function formatUptime(seconds: number | null): string {
  if (seconds === null || !Number.isFinite(seconds)) return "";
  const total = Math.max(0, Math.floor(seconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  if (hours > 0) return `${hours} 小时 ${minutes} 分`;
  return `${minutes} 分钟`;
}

function statusLabel(status: PetServerStatus["status"]): string {
  const labels: Record<PetServerStatus["status"], string> = {
    Running: "运行中",
    Starting: "启动中",
    Stopping: "停止中",
    Stopped: "已停止",
    Error: "异常",
    Unknown: "未知",
  };
  return labels[status];
}

function statusClass(status: PetServerStatus["status"]): string {
  return `status-${status.toLowerCase()}`;
}

async function refreshStatuses() {
  if (statusLoading.value) return;
  statusLoading.value = true;
  statusError.value = "";
  try {
    statuses.value = await petApi.loadServerStatuses();
  } catch (error) {
    statusError.value = error instanceof Error ? error.message : String(error);
  } finally {
    statusLoading.value = false;
  }
}

function startStatusPolling() {
  stopStatusPolling();
  statusTimer = setInterval(() => {
    if (bubbleKind.value === "status" && !disposed) void refreshStatuses();
  }, 5000);
}

async function showServerStatus() {
  closeMenus();
  bubbleKind.value = "status";
  bubbleMessage.value = "";
  pulsePet(0.42, -1);
  await refreshStatuses();
  startStatusPolling();
}

function openMenu(event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();
  const width = window.innerWidth;
  const height = window.innerHeight;
  menuSide.value = event.clientX < width / 2 ? "left" : "right";
  menuLeft.value = menuSide.value === "left" ? 22 : width - 22 - 156;
  // 三级菜单都从同一高度展开，给表情列表留出完整的可点击区域。
  menuTop.value = Math.max(16, Math.min(event.clientY - 24, height - 312));
  menuOpen.value = true;
  actionsOpen.value = false;
  expressionsOpen.value = false;
  void setClickThrough(false);
}

function closeMenus() {
  menuOpen.value = false;
  actionsOpen.value = false;
  expressionsOpen.value = false;
}

function chooseExpression(expression: PetExpression) {
  const direction = expression === currentExpression.value ? 1 : -1;
  currentExpression.value = expression;
  pulsePet(0.82, direction);
  closeMenus();
}

function showNotice(message: string) {
  bubbleKind.value = "notice";
  bubbleMessage.value = message;
  stopStatusPolling();
}

async function hidePet() {
  closeMenus();
  if (transparentMode.value) exitTransparentMode(false);
  try {
    await petApi.hide();
  } catch (error) {
    showNotice(`隐藏桌宠失败：${error instanceof Error ? error.message : String(error)}`);
  }
}

async function openMainWindow() {
  closeMenus();
  try {
    await petApi.openMainWindow();
    if (isPreview) showNotice("桌面端会在这里唤起 Sea Lantern 主窗口。");
  } catch (error) {
    showNotice(`打开主窗口失败：${error instanceof Error ? error.message : String(error)}`);
  }
}

function handleStatusEntryClick() {
  void openMainWindow();
}

async function enterTransparentMode() {
  closeMenus();
  try {
    await petApi.startDesktopJelly();
  } catch (error) {
    void petApi.stopDesktopJelly().catch(() => undefined);
    showNotice(`桌面图标弹弹弹启动失败：${error instanceof Error ? error.message : String(error)}`);
    return;
  }
  previewExpression = currentExpression.value;
  currentExpression.value = "happy";
  transparentMode.value = true;
  danceStartedAt = performance.now();
  resetPetSpringState(springState);
  springTransform.value = {
    x: 0,
    y: 0,
    scaleX: 1,
    scaleY: 1,
    rotation: 0,
  };
  audioError.value = "";
  bubbleKind.value = "meme";
  bubbleMessage.value = "朋友，这杯先干为敬。";
  stopStatusPolling();
  try {
    await audio.start();
  } catch (error) {
    audioError.value = error instanceof Error ? error.message : String(error);
  }
}

function exitTransparentMode(showMessage = true) {
  if (!transparentMode.value) return;
  transparentMode.value = false;
  void petApi.stopDesktopJelly().catch((error) => {
    console.warn("恢复桌面图标失败:", error);
  });
  audio.stop();
  resetPetSpringState(springState);
  springTransform.value = {
    x: 0,
    y: 0,
    scaleX: 1,
    scaleY: 1,
    rotation: 0,
  };
  currentExpression.value = previewExpression;
  if (showMessage) {
    bubbleKind.value = "notice";
    bubbleMessage.value = "通透模式已退出，海晶灯重新亮起来了。";
  } else if (bubbleKind.value === "meme") {
    bubbleKind.value = null;
  }
}

function toggleTransparentMode() {
  if (transparentMode.value) {
    exitTransparentMode();
  } else {
    void enterTransparentMode();
  }
}

async function handlePetPointerDown(event: PointerEvent) {
  if (event.button !== 0) return;
  if (!transparentMode.value) {
    pulsePet(1.25, event.clientX < window.innerWidth / 2 ? 1 : -1);
  }
  if (isPreview) return;
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().startDragging();
  } catch (error) {
    console.warn("桌宠窗口拖拽失败:", error);
  }
}

function handleStagePointerDown(event: PointerEvent) {
  if (menuOpen.value) {
    const target = event.target as HTMLElement | null;
    if (!target?.closest(".pet-menu")) closeMenus();
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key !== "Escape") return;
  if (menuOpen.value) {
    closeMenus();
    return;
  }
  if (transparentMode.value) exitTransparentMode();
}

function updateEyeFromLocalPointer(event: PointerEvent) {
  const block = petBlockRef.value;
  if (!block) return;
  const rect = block.getBoundingClientRect();
  eyeTarget.value = calculateEyeOffset(
    { x: event.clientX, y: event.clientY },
    { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 },
  );
}

function resetEyeTarget() {
  eyeTarget.value = {
    x: 0,
    y: 0,
    rotation: 0,
    intensity: 0,
    tracking: false,
    distance: 0,
  };
}

async function pollGlobalCursor() {
  if (disposed || cursorPolling || isPreview) return;
  cursorPolling = true;
  try {
    const { cursorPosition, getCurrentWindow } = await import("@tauri-apps/api/window");
    const window = getCurrentWindow();
    const now = performance.now();
    const refreshGeometry = !windowGeometry.ready || now - windowGeometry.updatedAt > 250;
    const cursorPromise = cursorPosition();
    if (refreshGeometry) {
      const [windowPosition, scaleFactor] = await Promise.all([
        window.outerPosition(),
        window.scaleFactor(),
      ]);
      windowGeometry = {
        originX: windowPosition.x / scaleFactor,
        originY: windowPosition.y / scaleFactor,
        scaleFactor,
        updatedAt: now,
        ready: true,
      };
    }
    const cursor = await cursorPromise;
    const block = petBlockRef.value;
    if (!block) return;
    const rect = block.getBoundingClientRect();
    const cursorX = cursor.x / windowGeometry.scaleFactor - windowGeometry.originX;
    const cursorY = cursor.y / windowGeometry.scaleFactor - windowGeometry.originY;
    eyeTarget.value = calculateEyeOffset(
      { x: cursorX + windowGeometry.originX, y: cursorY + windowGeometry.originY },
      {
        x: windowGeometry.originX + rect.left + rect.width / 2,
        y: windowGeometry.originY + rect.top + rect.height / 2,
      },
    );

    if (menuOpen.value || bubbleKind.value) {
      void setClickThrough(false);
    } else {
      const padding = 14;
      const interactive =
        cursorX >= rect.left - padding &&
        cursorX <= rect.right + padding &&
        cursorY >= rect.top - padding &&
        cursorY <= rect.bottom + padding;
      void setClickThrough(!interactive);
    }
  } catch (error) {
    // 窗口刚创建或权限尚未生效时忽略这一帧，下一帧继续。
    console.debug("桌宠读取全局鼠标位置失败:", error);
  } finally {
    cursorPolling = false;
    // 30Hz 更新目标点，60fps 的弹簧负责补间，比原来 18Hz 直接改 transform 更顺滑。
    if (!disposed && !isPreview) cursorTimer = setTimeout(pollGlobalCursor, 34);
  }
}

function handleExpressionPreview(expression: PetExpression) {
  currentExpression.value = expression;
}

onMounted(async () => {
  startPetAnimationLoop();
  document.addEventListener("pointerdown", handleStagePointerDown);
  document.addEventListener("keydown", handleKeydown);
  document.addEventListener("pointermove", updateEyeFromLocalPointer);
  document.addEventListener("pointerleave", resetEyeTarget);

  await nextTick();
  try {
    await petApi.frontendReady();
  } catch (error) {
    console.warn("桌宠通知前端就绪失败:", error);
  }

  if (isPreview) {
    if (previewKind === "menu") {
      menuSide.value = "left";
      menuLeft.value = 22;
      menuTop.value = 62;
      menuOpen.value = true;
      actionsOpen.value = true;
      expressionsOpen.value = true;
    } else if (previewKind === "status") {
      void showServerStatus();
    } else if (previewKind === "transparent") {
      transparentMode.value = true;
      danceStartedAt = performance.now();
      currentExpression.value = "happy";
      bubbleKind.value = "meme";
      bubbleMessage.value = "朋友，这杯先干为敬。";
    }
  } else {
    void pollGlobalCursor();
  }
});

onBeforeUnmount(() => {
  disposed = true;
  if (cursorTimer) clearTimeout(cursorTimer);
  if (physicsFrame !== null) cancelAnimationFrame(physicsFrame);
  stopStatusPolling();
  audio.dispose();
  void petApi.stopDesktopJelly().catch(() => undefined);
  document.removeEventListener("pointerdown", handleStagePointerDown);
  document.removeEventListener("keydown", handleKeydown);
  document.removeEventListener("pointermove", updateEyeFromLocalPointer);
  document.removeEventListener("pointerleave", resetEyeTarget);
});
</script>

<template>
  <div
    class="pet-stage"
    :class="{ 'is-preview': isPreview, 'transparent-mode': transparentMode }"
    :style="seaLanternStyle"
    @contextmenu.prevent
  >
    <div v-if="isPreview" class="preview-wallpaper" aria-hidden="true">
      <span class="preview-island island-one"></span>
      <span class="preview-island island-two"></span>
      <span class="preview-glow glow-one"></span>
      <span class="preview-glow glow-two"></span>
    </div>

    <div v-if="transparentMode" class="party-lights" aria-hidden="true">
      <span class="party-beam beam-one"></span>
      <span class="party-beam beam-two"></span>
      <span class="party-beam beam-three"></span>
    </div>

    <div v-if="isPreview" class="preview-badge">
      <span class="preview-dot"></span>
      浏览器效果预览
    </div>

    <Transition name="bubble-pop">
      <section
        v-if="bubbleKind"
        class="pet-bubble"
        :class="`bubble-${bubbleKind}`"
        aria-live="polite"
      >
        <div class="bubble-heading">
          <span class="bubble-icon">
            <Activity v-if="bubbleKind === 'status'" :size="15" />
            <Volume2 v-else-if="bubbleKind === 'meme'" :size="15" />
            <Sparkles v-else :size="15" />
          </span>
          <strong>
            {{
              bubbleKind === "status" ? "服务器状态" : bubbleKind === "meme" ? "通透模式" : "小晶说"
            }}
          </strong>
          <button type="button" class="bubble-close" aria-label="关闭气泡" @click="closeBubble">
            <X :size="14" />
          </button>
        </div>

        <template v-if="bubbleKind === 'status'">
          <p v-if="statusLoading && !statuses.length" class="bubble-placeholder">
            正在读取服务器状态…
          </p>
          <p v-else-if="statusError" class="bubble-placeholder bubble-error">
            {{ statusError }}
          </p>
          <p v-else-if="!statuses.length" class="bubble-placeholder">
            还没有服务器，先去主窗口创建一个吧。
          </p>
          <ul v-else class="status-list">
            <li
              v-for="server in statuses"
              :key="server.id"
              class="status-row"
              @click="handleStatusEntryClick"
            >
              <span class="status-dot" :class="statusClass(server.status)"></span>
              <span class="status-name">{{ server.name }}</span>
              <span class="status-state" :class="statusClass(server.status)">
                {{ statusLabel(server.status) }}
              </span>
              <span v-if="server.uptime !== null" class="status-uptime">
                {{ formatUptime(server.uptime) }}
              </span>
            </li>
          </ul>
          <div class="bubble-footer">
            <span class="status-summary">{{ statusSummary }}</span>
            <span v-if="statuses.some((server) => server.demo)" class="preview-data-tag"
              >预览数据</span
            >
            <button
              type="button"
              class="bubble-refresh"
              :class="{ spinning: statusLoading }"
              aria-label="刷新服务器状态"
              @click="refreshStatuses"
            >
              <RefreshCw :size="13" />
            </button>
          </div>
        </template>

        <template v-else-if="bubbleKind === 'meme'">
          <div class="meme-copy">
            <p class="meme-title">🎵 朋友的酒 DJ 版 · 洛天依</p>
            <p>{{ bubbleMessage }}</p>
            <p v-if="audioError" class="audio-warning">音频启动失败：{{ audioError }}</p>
          </div>
          <div class="meme-equalizer" aria-hidden="true">
            <span v-for="index in 9" :key="index" :style="{ '--bar-index': index }"></span>
          </div>
          <button type="button" class="meme-exit" @click="exitTransparentMode()">
            退出通透模式
          </button>
        </template>

        <template v-else>
          <p class="notice-copy">{{ bubbleMessage }}</p>
        </template>

        <span class="bubble-tail" aria-hidden="true"></span>
      </section>
    </Transition>

    <div class="pet-anchor">
      <div v-if="transparentMode" class="transparent-aura" aria-hidden="true">
        <span class="aura-ring ring-one"></span>
        <span class="aura-ring ring-two"></span>
        <span class="aura-ring ring-three"></span>
      </div>

      <div class="pet-visual">
        <span
          v-for="particleClass in particleClasses"
          :key="particleClass"
          class="pet-particle"
          :class="particleClass"
          aria-hidden="true"
        ></span>

        <div
          ref="petBlockRef"
          class="pet-block"
          :class="{ dancing: transparentMode, tracking: eyeTarget.tracking }"
          :style="physicsStyle"
          role="img"
          :aria-label="`Sea Lantern 桌宠，当前表情：${currentExpressionLabel}`"
          @pointerenter="handlePetPointerEnter"
          @contextmenu="openMenu"
          @pointerdown="handlePetPointerDown"
        >
          <div class="pet-glow" aria-hidden="true"></div>
          <div class="sea-face">
            <div class="sea-texture" aria-hidden="true"></div>
            <div class="sea-bevel" aria-hidden="true"></div>
            <div class="sea-shine" aria-hidden="true"></div>

            <div class="pet-eyes" :class="`expression-${currentExpression}`">
              <div class="eye-slot left">
                <span class="eye-motion" :style="eyeMotionStyle">
                  <span class="eye-shape" :class="`eye-${currentExpression}`"></span>
                </span>
              </div>
              <div class="eye-slot right">
                <span class="eye-motion" :style="eyeMotionStyle">
                  <span class="eye-shape" :class="`eye-${currentExpression}`"></span>
                </span>
              </div>
            </div>

            <div class="face-sparkle sparkle-one" aria-hidden="true"></div>
            <div class="face-sparkle sparkle-two" aria-hidden="true"></div>
            <div v-if="transparentMode" class="music-note note-one" aria-hidden="true">♪</div>
            <div v-if="transparentMode" class="music-note note-two" aria-hidden="true">♫</div>
          </div>
          <span class="pet-shadow" aria-hidden="true"></span>
        </div>

        <div class="pet-nameplate">
          <Music2 v-if="transparentMode" :size="11" class="nameplate-music" />
          <span v-else class="nameplate-mark" aria-hidden="true"></span>
          <span class="nameplate-name">小晶</span>
          <span class="nameplate-divider" aria-hidden="true"></span>
          <span class="nameplate-sub">{{ transparentMode ? "通透模式" : "海晶灯桌宠" }}</span>
        </div>
      </div>
    </div>

    <Transition name="menu-pop">
      <nav
        v-if="menuOpen"
        class="pet-menu"
        :class="menuSide === 'left' ? 'opens-right' : 'opens-left'"
        :style="{ left: `${menuLeft}px`, top: `${menuTop}px` }"
        aria-label="桌宠菜单"
        @contextmenu.stop.prevent
      >
        <div class="menu-caption">小晶 · {{ currentExpressionLabel }}</div>

        <button type="button" class="menu-item" @click="hidePet">
          <EyeOff :size="16" />
          <span>隐藏桌宠</span>
        </button>
        <button type="button" class="menu-item" @click="openMainWindow">
          <AppWindow :size="16" />
          <span>打开 Sea Lantern</span>
        </button>
        <button type="button" class="menu-item" @click="showServerStatus">
          <Server :size="16" />
          <span>显示服务器状态</span>
        </button>

        <div class="menu-divider"></div>

        <div
          class="menu-item submenu-host"
          :class="{ active: actionsOpen }"
          role="button"
          tabindex="0"
          aria-haspopup="menu"
          :aria-expanded="actionsOpen"
          @mouseenter="actionsOpen = true"
          @click.stop="actionsOpen = true"
        >
          <Sparkles :size="16" />
          <span>动作菜单</span>
          <ChevronRight :size="14" class="menu-arrow" />

          <div
            v-if="actionsOpen"
            class="pet-submenu action-submenu"
            :style="actionSubmenuStyle"
            role="menu"
          >
            <div class="submenu-caption">动作</div>

            <div
              class="menu-item submenu-host"
              :class="{ active: expressionsOpen }"
              role="button"
              tabindex="0"
              aria-haspopup="menu"
              :aria-expanded="expressionsOpen"
              @mouseenter="expressionsOpen = true"
              @click.stop="expressionsOpen = true"
            >
              <span class="mini-eye-icon" aria-hidden="true">◉</span>
              <span>切换表情</span>
              <ChevronRight :size="14" class="menu-arrow" />

              <div
                v-if="expressionsOpen"
                class="pet-submenu expression-submenu"
                :style="expressionSubmenuStyle"
                role="menu"
              >
                <div class="submenu-caption">用眼睛表达情绪</div>
                <button
                  v-for="expression in PET_EXPRESSIONS"
                  :key="expression.id"
                  type="button"
                  class="expression-option"
                  :class="{ selected: currentExpression === expression.id }"
                  @mouseenter="handleExpressionPreview(expression.id)"
                  @click.stop="chooseExpression(expression.id)"
                >
                  <span class="expression-icon" :class="`icon-${expression.id}`">
                    <span></span><span></span>
                  </span>
                  <span class="expression-label">
                    {{ expression.label }}
                    <small>{{ expression.hint }}</small>
                  </span>
                  <Check v-if="currentExpression === expression.id" :size="13" class="check-icon" />
                </button>
              </div>
            </div>

            <button
              type="button"
              class="menu-item transparent-mode-item"
              :class="{ active: transparentMode }"
              @click.stop="toggleTransparentMode"
            >
              <Music2 :size="16" />
              <span>{{ transparentMode ? "退出通透模式" : "通透模式" }}</span>
              <span class="mode-switch" :class="{ on: transparentMode }">
                <span></span>
              </span>
            </button>
            <div class="submenu-tip">朋友的酒 DJ 版 · 洛天依</div>
          </div>
        </div>
      </nav>
    </Transition>
  </div>
</template>
