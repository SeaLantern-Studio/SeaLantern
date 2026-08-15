<script setup lang="ts">
/**
 * TaskPill —— 顶栏状态指示器 + 任务球，双态
 *
 * 复用顶栏 .header-status 位置：
 * - 没活儿的时候：绿点脉冲 + "SeaLantern"，跟原来状态指示器一个样
 * - 来活儿了：变形进度球，SVG 主题色环形进度条裹着胶囊，左小胶囊放图标和百分比，右边放文件名
 *
 * 点一下展开详情面板，从胶囊底下 absolute 浮出来，不受顶栏高度限制
 * 显隐由 downloadStore 操刀，完成 30s 自动撅掉，查看后关面板 8s 撅掉
 */
import { ref, reactive, computed, watch, nextTick, onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import { Download, Check, AlertCircle, FolderOpen, ExternalLink, Copy } from "lucide-vue-next";
import { useDownloadStore } from "@stores/downloadStore";
import { useToast } from "cmzya-modern-ui";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import { handleError } from "@utils/errorHandler";

/** 字节数转人话，内联写是因为 serverUtils 有模块加载副作用，import 一下能把整个应用撅白屏 */
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB", "PB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), sizes.length - 1);
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

const store = useDownloadStore();
const toast = useToast();
const router = useRouter();

// 胶囊根元素，含面板，量尺寸给 SVG 进度环用，也用来判断点击是不是在外面
const rootRef = ref<HTMLElement>();
// 文件名容器，用来检测要不要 marquee 滚
const nameWrapRef = ref<HTMLElement>();
const needsMarquee = ref(false);

// 有任务且该显示就是 true
const hasTask = computed(() => store.shouldShowPill);

// 闲着的时候显示的品牌名
const appName = computed(() => i18n.t("common.app_name"));

// 胶囊的 class 绑定
const capsuleClass = computed(() => ({
  "has-task": hasTask.value,
  completed: store.isCompleted,
  error: store.isError,
  expanded: store.panelOpen,
}));

// SVG 进度环描边宽度
const STROKE_W = 2;
const ringSize = reactive({ w: 0, h: 0 });

// 显示用进度，rAF 缓动插值，避免 800ms 轮询跳变
const displayProgress = ref(0);
let progressRaf: number | null = null;

/** 用 rAF 把 displayProgress 从当前值缓动到目标值，duration 跟轮询间隔一致 */
function animateProgress(target: number) {
  if (progressRaf !== null) cancelAnimationFrame(progressRaf);
  const start = displayProgress.value;
  const startTime = performance.now();
  const duration = 800;
  const step = (now: number) => {
    const t = Math.min(1, (now - startTime) / duration);
    // easeOutQuad，开头快后面慢
    const eased = 1 - (1 - t) * (1 - t);
    displayProgress.value = start + (target - start) * eased;
    if (t < 1) {
      progressRaf = requestAnimationFrame(step);
    } else {
      progressRaf = null;
    }
  };
  progressRaf = requestAnimationFrame(step);
}

// 胶囊高 32px，full 圆角就是 (32-2)/2 = 15
const ringRadius = computed(() =>
  ringSize.h === 0 ? 0 : Math.max(0, (ringSize.h - STROKE_W) / 2),
);

// 圆角矩形周长 = 2(w-2r) + 2(h-2r) + 2πr
const perimeter = computed(() => {
  const w = Math.max(0, ringSize.w - STROKE_W);
  const h = Math.max(0, ringSize.h - STROKE_W);
  const r = ringRadius.value;
  if (w === 0 || h === 0 || r === 0) return 0;
  return 2 * (w - 2 * r) + 2 * (h - 2 * r) + 2 * Math.PI * r;
});

// 进度映射到 stroke 偏移量，用 displayProgress 做平滑
const dashOffset = computed(() => {
  const p = perimeter.value;
  if (p === 0) return 0;
  const prog = Math.min(100, Math.max(0, displayProgress.value));
  return p * (1 - prog / 100);
});

// 完成/出错时改成实心描边，不然 dash 浮点缝隙看着膈应
const isSolidRing = computed(() => store.isCompleted || store.isError);

const ringColor = computed(() => {
  if (store.isError) return "var(--sl-error)";
  if (store.isCompleted) return "var(--sl-success)";
  return "var(--sl-primary)";
});

// 状态图标：下载中是下载图标，完成是 ✓，出错是 ⚠
const statusIcon = computed(() => {
  if (store.isError) return AlertCircle;
  if (store.isCompleted) return Check;
  return Download;
});

// 胶囊右边显示的文本
const displayName = computed(() => {
  if (store.isCompleted) return i18n.t("taskPill.completed");
  if (store.isError) return i18n.t("taskPill.failed");
  return store.filename || i18n.t("download-file.downloading");
});

/** 速度文本，带 /s 后缀，没速度就显示 -- */
const speedText = computed(() => (store.speed > 0 ? `${formatBytes(store.speed)}/s` : "--"));

/** 大小文本，已下载 / 总大小 */
const sizeText = computed(() => {
  const t = store.currentTask;
  if (!t) return "--";
  return `${formatBytes(t.downloaded)} / ${formatBytes(t.totalSize)}`;
});

/** 点一下开/关面板，完成了就标记已查看，把自动消失计时撅掉 */
function togglePanel() {
  if (store.panelOpen) {
    store.setPanelOpen(false);
    return;
  }
  // 得先开面板再标记已查看，反着来的话中间状态 viewed=true 但 panelOpen=false，
  // shouldShowPill 会瞬间变 false，球和面板一闪就撅了
  store.setPanelOpen(true);
  if (store.isFinished) store.markViewed();
}

/** 打开保存路径所在的文件夹，取 savePath 的目录部分 */
async function openFolder() {
  try {
    const dir = store.savePath.replace(/[\\/][^\\/]+$/, "");
    if (dir) await systemApi.openFolder(dir);
  } catch (e) {
    toast.error(handleError(e));
  }
}

/** 复制保存路径到剪贴板 */
async function copySavePath() {
  try {
    await navigator.clipboard.writeText(store.savePath);
    toast.success(i18n.t("taskPill.copied"));
  } catch {
    /* 剪贴板不可用就不管了 */
  }
}

/** 关面板，跳转到下载页 */
function gotoDownloadPage() {
  store.setPanelOpen(false);
  router.push("/download");
}

/** 撅掉当前下载任务 */
async function cancelDownload() {
  await store.cancelTask();
}

/** 点面板外面时收起，document 级 mousedown 监听 */
function onDocMouseDown(e: MouseEvent) {
  if (!store.panelOpen) return;
  if (rootRef.value && !rootRef.value.contains(e.target as Node)) {
    store.setPanelOpen(false);
  }
}

/** 检测文件名溢没溢出，溢出了就开 marquee 滚动，顺便设滚动距离 */
function checkMarquee() {
  const wrap = nameWrapRef.value;
  if (!wrap) return;
  const span = wrap.querySelector(".task-name") as HTMLElement | null;
  if (!span) return;
  const overflow = span.scrollWidth - wrap.clientWidth;
  if (overflow > 2) {
    needsMarquee.value = true;
    span.style.setProperty("--marquee-distance", `-${overflow}px`);
  } else {
    needsMarquee.value = false;
  }
}

/** ResizeObserver 实例，盯着胶囊尺寸变没变，变了就同步 SVG 进度环 */
let roCapsule: ResizeObserver | null = null;

/** 量一下胶囊实际尺寸，更新 ringSize，驱动 SVG 进度环重绘 */
function measureCapsule() {
  if (rootRef.value) {
    ringSize.w = rootRef.value.offsetWidth;
    ringSize.h = rootRef.value.offsetHeight;
  }
}

/** 绑定 ResizeObserver 到 rootRef */
function bindObserver() {
  roCapsule?.disconnect();
  roCapsule = null;
  if (rootRef.value) {
    roCapsule = new ResizeObserver(measureCapsule);
    roCapsule.observe(rootRef.value);
    measureCapsule();
  }
}

onMounted(() => {
  document.addEventListener("mousedown", onDocMouseDown);
  // 胶囊始终渲染，直接绑
  nextTick(() => {
    bindObserver();
    checkMarquee();
  });
});

onUnmounted(() => {
  roCapsule?.disconnect();
  if (progressRaf !== null) cancelAnimationFrame(progressRaf);
  document.removeEventListener("mousedown", onDocMouseDown);
});

// 进度变了，缓动到目标值
watch(
  () => store.progress,
  (v) => animateProgress(v),
);

// 完成时直接跳到 100，不等缓动
watch(
  () => store.isCompleted,
  (v) => {
    if (v) {
      if (progressRaf !== null) cancelAnimationFrame(progressRaf);
      displayProgress.value = 100;
    }
  },
);

// 任务态切了，重新绑 observer + 检测 marquee
watch(
  () => store.shouldShowPill,
  () => {
    nextTick(() => {
      bindObserver();
      checkMarquee();
    });
  },
);

// 百分比位数变了主动重测 SVG 尺寸
watch(
  () => Math.round(store.progress),
  () => nextTick(measureCapsule),
);

// 文件名变了，重测 + 检测 marquee
watch(
  () => store.filename,
  () =>
    nextTick(() => {
      measureCapsule();
      checkMarquee();
    }),
);

// 状态切换，重测
watch(
  () => `${store.isDownloading}:${store.isCompleted}:${store.isError}`,
  () => nextTick(measureCapsule),
);

// 面板开/关，只重测 marquee
watch(
  () => store.panelOpen,
  () => nextTick(checkMarquee),
);
</script>

<template>
  <!-- 胶囊根元素，始终渲染，没活儿是状态指示器，来活儿了变进度球 -->
  <div ref="rootRef" class="task-capsule" :class="capsuleClass">
    <!-- SVG 进度环，只有任务态才画，裹着整个胶囊外轮廓 -->
    <svg
      v-if="hasTask"
      class="progress-ring"
      :width="ringSize.w"
      :height="ringSize.h"
      aria-hidden="true"
    >
      <rect
        :x="STROKE_W / 2"
        :y="STROKE_W / 2"
        :width="Math.max(0, ringSize.w - STROKE_W)"
        :height="Math.max(0, ringSize.h - STROKE_W)"
        :rx="ringRadius"
        :ry="ringRadius"
        fill="none"
        :stroke="ringColor"
        :stroke-width="STROKE_W"
        :stroke-dasharray="isSolidRing ? undefined : perimeter"
        :stroke-dashoffset="isSolidRing ? 0 : dashOffset"
        stroke-linecap="round"
      />
    </svg>

    <!-- 没活儿：绿点脉冲 + SeaLantern -->
    <div v-if="!hasTask" class="capsule-idle">
      <span class="status-dot"></span>
      <span class="status-text">{{ appName }}</span>
    </div>

    <!-- 来活儿了：左小胶囊放图标和百分比，右边放文件名 -->
    <button v-else class="capsule-task" :aria-expanded="store.panelOpen" @click="togglePanel">
      <div class="task-badge">
        <component
          v-if="statusIcon"
          :is="statusIcon"
          :size="12"
          class="badge-icon"
          :stroke-width="2.2"
        />
        <span class="badge-percent">{{ Math.round(displayProgress) }}%</span>
      </div>
      <div ref="nameWrapRef" class="task-name-wrap">
        <span class="task-name" :class="{ marquee: needsMarquee }">{{ displayName }}</span>
      </div>
    </button>

    <!-- 详情面板，从胶囊底下 absolute 浮出来 -->
    <Transition name="pill-expand">
      <div v-if="hasTask && store.panelOpen" class="pill-panel">
        <div class="panel-filename" :title="store.filename">
          {{ store.filename || displayName }}
        </div>

        <div class="panel-stats">
          <div class="stat-row">
            <span class="stat-label">{{ i18n.t("taskPill.progress") }}</span>
            <span class="stat-value stat-big">{{ displayProgress.toFixed(1) }}%</span>
          </div>
          <div v-if="!store.isError" class="stat-row">
            <span class="stat-label">{{ i18n.t("taskPill.size") }}</span>
            <span class="stat-value">{{ sizeText }}</span>
          </div>
          <div v-if="store.isDownloading" class="stat-row">
            <span class="stat-label">{{ i18n.t("taskPill.speed") }}</span>
            <span class="stat-value">{{ speedText }}</span>
          </div>
          <div v-if="store.taskError" class="stat-row error-row">
            <span class="stat-label">{{ i18n.t("taskPill.error") }}</span>
            <span class="stat-value">{{ store.taskError }}</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">{{ i18n.t("taskPill.savePath") }}</span>
            <span class="stat-value path-value" :title="store.savePath" @click="copySavePath">
              <span class="path-text">{{ store.savePath || "-" }}</span>
              <Copy :size="12" class="copy-icon" />
            </span>
          </div>
        </div>

        <div class="panel-actions">
          <button class="pill-btn" @click="openFolder">
            <FolderOpen v-if="FolderOpen" :size="14" class="btn-icon" />
            <span>{{ i18n.t("taskPill.openFolder") }}</span>
          </button>
          <button class="pill-btn" @click="gotoDownloadPage">
            <ExternalLink v-if="ExternalLink" :size="14" class="btn-icon" />
            <span>{{ i18n.t("taskPill.gotoDownload") }}</span>
          </button>
          <button v-if="store.isDownloading" class="pill-btn pill-btn-warn" @click="cancelDownload">
            <span>{{ i18n.t("taskPill.cancel") }}</span>
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* ===================== 胶囊本体，复用 .header-status 样式 =====================
   始终渲染，没活儿是状态指示器，来活儿了变进度球
   32px 高、full 圆角、白底、细边框、轻阴影，跟顶栏其他控件视觉一致 */
.task-capsule {
  position: relative;
  display: flex;
  align-items: center;
  height: 32px;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-full);
  box-shadow: var(--sl-shadow-sm);
  -webkit-app-region: no-drag;
  cursor: default;
  user-select: none;
  -webkit-user-select: none;
  flex-shrink: 0;
  z-index: 200;
  transition:
    border-color var(--sl-transition-normal),
    box-shadow var(--sl-transition-normal);
}

/* 任务态：border 撅掉用 SVG 进度环代替，阴影加强 */
.task-capsule.has-task {
  border-color: transparent;
  box-shadow: var(--sl-shadow-md);
  cursor: pointer;
}

/* 胶囊始终保持 full 圆角，面板跟胶囊之间留 6px 间距 */

/* SVG 进度环，绝对贴合整个胶囊外轮廓 */
.progress-ring {
  position: absolute;
  top: -1px; /* 补偿原 border 的 1px，让描边贴在原 border 位置 */
  left: -1px;
  pointer-events: none;
  z-index: 2;
  transition: stroke var(--sl-transition-fast);
}

/* ===================== 没活儿：绿点 + SeaLantern ===================== */
.capsule-idle {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 0 12px;
  height: 100%;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--sl-primary);
  flex-shrink: 0;
  animation: dot-pulse 2s ease-in-out infinite;
}

@keyframes dot-pulse {
  0%,
  100% {
    opacity: 1;
    box-shadow: 0 0 0 0 color-mix(in srgb, var(--sl-primary) 40%, transparent);
  }
  50% {
    opacity: 0.85;
    box-shadow: 0 0 0 4px color-mix(in srgb, var(--sl-primary) 0%, transparent);
  }
}

.status-text {
  font-size: var(--sl-font-size-sm);
  color: var(--sl-text-primary);
  font-weight: 500;
  white-space: nowrap;
}

/* ===================== 来活儿了：图标+% + 文件名 ===================== */
.capsule-task {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 2px 12px 2px 4px;
  height: 100%;
  background: transparent;
  border: none;
  cursor: pointer;
  min-width: 0;
  max-width: 320px;
  box-sizing: border-box;
  color: inherit;
  font-family: inherit;
  border-radius: var(--sl-radius-full);
}

.capsule-task:focus-visible {
  outline: 2px solid var(--sl-primary);
  outline-offset: 2px;
}

/* 左侧小胶囊，主题色浅调底，深色调图标和百分比 */
.task-badge {
  display: flex;
  align-items: center;
  gap: 3px;
  height: 24px;
  padding: 0 8px;
  background: var(--sl-primary-bg);
  border-radius: var(--sl-radius-full);
  color: var(--sl-primary-dark);
  font-size: var(--sl-font-size-xs);
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}

.badge-icon {
  flex-shrink: 0;
}

.task-capsule.completed .task-badge {
  background: var(--sl-success-bg);
  color: var(--sl-success);
}

.task-capsule.error .task-badge {
  background: var(--sl-error-bg);
  color: var(--sl-error);
}

/* 右边文件名，溢出了就 marquee 滚 */
.task-name-wrap {
  overflow: hidden;
  flex: 1;
  min-width: 0;
  max-width: 200px;
}

.task-name {
  display: inline-block;
  white-space: nowrap;
  font-size: var(--sl-font-size-sm);
  color: var(--sl-text-primary);
  font-weight: 500;
}

.task-name.marquee {
  animation: marquee-scroll 7s ease-in-out infinite;
}

@keyframes marquee-scroll {
  0%,
  18% {
    transform: translateX(0);
  }
  82%,
  100% {
    transform: translateX(var(--marquee-distance, 0));
  }
}

/* ===================== 详情面板，从胶囊底下 absolute 浮出来 ===================== */
.pill-panel {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 6px;
  min-width: 320px;
  max-width: 400px;
  padding: var(--sl-space-md) var(--sl-space-lg) var(--sl-space-lg);
  box-sizing: border-box;
  background: var(--sl-surface);
  border-radius: var(--sl-radius-lg);
  box-shadow: 0 8px 28px rgba(0, 0, 0, 0.12);
  z-index: 1;
}

[data-acrylic="on"] .pill-panel {
  backdrop-filter: blur(var(--sl-acrylic-blur)) saturate(var(--sl-saturate-normal));
  -webkit-backdrop-filter: blur(var(--sl-acrylic-blur)) saturate(var(--sl-saturate-normal));
}

[data-acrylic="off"] .pill-panel {
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}

[data-theme="dark"] .pill-panel {
  box-shadow: 0 8px 28px rgba(0, 0, 0, 0.5);
}

.panel-filename {
  font-size: var(--sl-font-size-base);
  font-weight: 600;
  color: var(--sl-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: var(--sl-space-md);
  padding-bottom: var(--sl-space-sm);
  border-bottom: 1px solid var(--sl-border-light);
  line-height: 1.5;
}

.panel-stats {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: var(--sl-space-lg);
}

.stat-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-lg);
  font-size: var(--sl-font-size-sm);
  line-height: 1.6;
  min-height: 22px;
}

.stat-label {
  color: var(--sl-text-tertiary);
  flex-shrink: 0;
  font-weight: 400;
}

.stat-value {
  color: var(--sl-text-primary);
  text-align: right;
  font-variant-numeric: tabular-nums;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
}

.stat-big {
  font-size: var(--sl-font-size-xl);
  font-weight: 700;
  color: var(--sl-primary);
  line-height: 1.2;
}

.error-row .stat-value {
  color: var(--sl-error);
}

.path-value {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  max-width: 100%;
}

.path-value:hover {
  color: var(--sl-primary);
}

.path-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  direction: rtl;
  text-align: right;
}

.copy-icon {
  flex-shrink: 0;
  color: var(--sl-text-tertiary);
  opacity: 0.6;
  transition:
    opacity var(--sl-transition-fast),
    color var(--sl-transition-fast);
}

.path-value:hover .copy-icon {
  opacity: 1;
  color: var(--sl-primary);
}

.panel-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sl-space-sm);
  padding-top: var(--sl-space-md);
  border-top: 1px solid var(--sl-border-light);
}

.pill-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  font-size: var(--sl-font-size-sm);
  color: var(--sl-text-secondary);
  background: var(--sl-bg-secondary);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  cursor: pointer;
  transition:
    background-color var(--sl-transition-fast),
    color var(--sl-transition-fast),
    border-color var(--sl-transition-fast);
  font-family: inherit;
  line-height: 1.4;
}

.btn-icon {
  flex-shrink: 0;
  width: 15px;
  height: 15px;
  color: inherit;
}

.pill-btn:hover {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
  border-color: var(--sl-primary-border);
}

.pill-btn-warn:hover {
  background: var(--sl-error-bg);
  color: var(--sl-error);
  border-color: var(--sl-error-border);
}

/* 面板开/关过渡 */
.pill-expand-enter-active,
.pill-expand-leave-active {
  transition:
    transform 0.25s cubic-bezier(0.4, 0, 0.2, 1),
    opacity 0.2s ease;
  transform-origin: top right;
  overflow: hidden;
}

.pill-expand-enter-from,
.pill-expand-leave-to {
  transform: scaleY(0.7) translateY(-8px);
  opacity: 0;
}

.pill-expand-enter-to,
.pill-expand-leave-from {
  transform: scaleY(1) translateY(0);
  opacity: 1;
}
</style>
