<script setup lang="ts">
/**
 * 世界地图：可平移缩放的俯视图 + 在线玩家实时位置。
 *
 * 1 像素 = 1 方块。视口由 `camera`（中心点 + 缩放）描述，屏幕上任何一点的
 * 位置都由 `worldToScreen` 算出，region 图片和玩家标记共用同一套换算，
 * 因此两者永远不会错位。
 *
 * region 图片按「当前视野内、且磁盘上确实存在」来拉取，并发上限 3：
 * 首次进入某片区域需要真渲染（每块约 1 秒），之后走后端磁盘缓存。
 */
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import {
  MAP_DIMENSIONS,
  mapApi,
  type MapDimension,
  type PlayerPosition,
  type WorldMapRegion,
} from "@api/map";
import { serverApi } from "@api/server";
import { i18n } from "@language";
import type { ServerInstance } from "@type/server";

/** 一块 region 的边长（像素 = 方块）。 */
const REGION_PX = 512;
/** 缩放下限：再小就只剩一片糊，且会一次性索要大量 region。 */
const MIN_SCALE = 0.2;
/** 缩放上限：一个方块铺 4 像素，够看清单体建筑。 */
const MAX_SCALE = 4;
/** 同时渲染的 region 数量上限，避免一次点开 20 块把 CPU 打满。 */
const MAX_PARALLEL_TILES = 3;
/** 玩家位置轮询间隔。RCON 单次往返 20~50ms，1 秒完全撑得住。 */
const PLAYER_POLL_MS = 1000;
/** region 清单刷新间隔，用于发现地形变化并让对应瓦片重新渲染。 */
const REGION_REFRESH_MS = 15_000;

interface RegionTile {
  url: string;
  /** 渲染时源 `.mca` 的修改时间，用于判断是否需要重渲染 */
  mtime: number;
}

const instances = ref<{ id: string; name: string }[]>([]);
const instanceId = ref("");
const dimension = ref<MapDimension>("overworld");

const regions = ref<WorldMapRegion[]>([]);
const levelName = ref("");
const tiles = ref<Record<string, RegionTile>>({});

/** 视口中心的世界坐标与缩放。 */
const camera = ref({ x: 0, z: 0, scale: 1 });
const viewport = ref({ width: 800, height: 600 });
const viewportEl = ref<HTMLElement | null>(null);
const pointerWorld = ref<{ x: number; z: number } | null>(null);

const players = ref<PlayerPosition[]>([]);
const playerErrorKind = ref("");
const playerErrorText = ref("");
const playerUpdatedAt = ref(0);

const errorText = ref("");
const regionLoading = ref(false);
const pendingTiles = ref(0);
const dragging = ref(false);
/** 跟随模式：每次轮询到玩家移动就把镜头压过去。默认关，手动拖动即退出。 */
const followPlayer = ref(false);

/** 每次切换实例/维度自增，用来丢弃过期的异步结果。 */
let generation = 0;

const regionKeys = computed(() => new Set(regions.value.map((r) => `${r.region_x}/${r.region_z}`)));

const dimensionOptions = computed(() =>
  MAP_DIMENSIONS.map((value) => ({ value, label: i18n.t(`world_map.${value}`) })),
);

const zoomPercent = computed(() => Math.round(camera.value.scale * 100));

/** region 图片与玩家标记共用的世界坐标 → 视口像素换算。 */
function worldToScreen(x: number, z: number): { left: number; top: number } {
  return {
    left: (x - camera.value.x) * camera.value.scale + viewport.value.width / 2,
    top: (z - camera.value.z) * camera.value.scale + viewport.value.height / 2,
  };
}

/** 视野内需要显示的 region；只包含磁盘上确实存在的块。 */
const visibleTiles = computed(() => {
  const { scale } = camera.value;
  const halfWidth = viewport.value.width / 2 / scale;
  const halfHeight = viewport.value.height / 2 / scale;
  const minX = Math.floor((camera.value.x - halfWidth) / REGION_PX);
  const maxX = Math.floor((camera.value.x + halfWidth) / REGION_PX);
  const minZ = Math.floor((camera.value.z - halfHeight) / REGION_PX);
  const maxZ = Math.floor((camera.value.z + halfHeight) / REGION_PX);

  const result: {
    key: string;
    regionX: number;
    regionZ: number;
    url: string;
    left: number;
    top: number;
    size: number;
  }[] = [];

  for (let regionX = minX; regionX <= maxX; regionX += 1) {
    for (let regionZ = minZ; regionZ <= maxZ; regionZ += 1) {
      const key = `${regionX}/${regionZ}`;
      if (!regionKeys.value.has(key)) continue;
      const { left, top } = worldToScreen(regionX * REGION_PX, regionZ * REGION_PX);
      result.push({
        key,
        regionX,
        regionZ,
        url: tiles.value[key]?.url ?? "",
        left,
        top,
        size: REGION_PX * scale,
      });
    }
  }
  return result;
});

/** 玩家标记的屏幕坐标。 */
const playerMarkers = computed(() =>
  players.value.map((player) => {
    const { left, top } = worldToScreen(player.x, player.z);
    return {
      name: player.name,
      left,
      top,
      // 游戏内 yaw：0 = +Z（南），90 = -X（西）。屏幕 +Z 朝下，
      // 所以朝上为 0 度的 CSS 旋转角是 atan2(-sin, -cos)。
      rotation:
        (Math.atan2(
          -Math.sin((player.yaw * Math.PI) / 180),
          -Math.cos((player.yaw * Math.PI) / 180),
        ) *
          180) /
        Math.PI,
      y: Math.round(player.y),
    };
  }),
);

function revokeTile(key: string) {
  const tile = tiles.value[key];
  if (tile?.url) URL.revokeObjectURL(tile.url);
  delete tiles.value[key];
}

function revokeAllTiles() {
  for (const key of Object.keys(tiles.value)) revokeTile(key);
}

/** 瓦片请求队列：限制并发，避免一次点开一大片直接卡住。 */
const queue: { key: string; regionX: number; regionZ: number }[] = [];
const queued = new Set<string>();
let activeTiles = 0;

function enqueueTile(regionX: number, regionZ: number) {
  const key = `${regionX}/${regionZ}`;
  if (tiles.value[key]?.url || queued.has(key)) return;
  queued.add(key);
  queue.push({ key, regionX, regionZ });
  pendingTiles.value = queue.length + activeTiles;
  pumpQueue();
}

function pumpQueue() {
  while (activeTiles < MAX_PARALLEL_TILES && queue.length > 0) {
    const job = queue.shift();
    if (!job) break;
    activeTiles += 1;
    void runTileJob(job);
  }
}

async function runTileJob(job: { key: string; regionX: number; regionZ: number }) {
  const current = generation;
  try {
    const blob = await mapApi.loadImage(instanceId.value, {
      region_x: job.regionX,
      region_z: job.regionZ,
      dimension: dimension.value,
    });
    // 切换实例/维度后回来的结果直接丢掉，别污染新地图
    if (current !== generation) return;
    const url = URL.createObjectURL(blob);
    revokeTile(job.key);
    tiles.value[job.key] = { url, mtime: Date.now() };
  } catch (error) {
    if (current === generation) errorText.value = describeError(error);
  } finally {
    queued.delete(job.key);
    activeTiles -= 1;
    pendingTiles.value = queue.length + activeTiles;
    pumpQueue();
  }
}

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/** 视野变化后把缺失的 region 排进队列。 */
function ensureVisibleTiles() {
  for (const tile of visibleTiles.value) {
    if (!tile.url) enqueueTile(tile.regionX, tile.regionZ);
  }
}

watch(visibleTiles, ensureVisibleTiles, { immediate: true });

async function loadInstances() {
  try {
    const list: ServerInstance[] = await serverApi.getList();
    instances.value = list.map((item) => ({ id: item.id, name: item.name }));
    if (!instanceId.value && instances.value.length > 0) {
      instanceId.value = instances.value[0].id;
    }
  } catch (error) {
    errorText.value = describeError(error);
  }
}

/** 拉取 region 清单；地形有变化时让对应瓦片重渲染。 */
async function refreshRegions(resetCamera: boolean) {
  if (!instanceId.value) return;
  const current = generation;
  regionLoading.value = true;
  try {
    const list = await mapApi.listRegions(instanceId.value, dimension.value);
    if (current !== generation) return;
    levelName.value = list.level_name;

    // 源文件时间变了 → 丢掉旧图，让视口重新排队渲染
    const changed: string[] = [];
    for (const region of list.regions) {
      const key = `${region.region_x}/${region.region_z}`;
      const tile = tiles.value[key];
      if (tile && region.modified_at_unix_secs > 0 && tile.mtime > 0) {
        // tile.mtime 是客户端取图时间；源文件比它新就说明地形改过
        if (region.modified_at_unix_secs * 1000 > tile.mtime) changed.push(key);
      }
    }
    for (const key of changed) revokeTile(key);

    regions.value = list.regions;
    if (resetCamera) frameOnFocus(list.regions);
    if (changed.length > 0) ensureVisibleTiles();
  } catch (error) {
    if (current === generation) errorText.value = describeError(error);
  } finally {
    if (current === generation) regionLoading.value = false;
  }
}

/** 把镜头对准已有 region 的中心，缩放到能看见大半片。 */
function centerOnRegions(list: WorldMapRegion[]) {
  const xs = list.map((region) => region.region_x);
  const zs = list.map((region) => region.region_z);
  const centerX = ((Math.min(...xs) + Math.max(...xs)) / 2 + 0.5) * REGION_PX;
  const centerZ = ((Math.min(...zs) + Math.max(...zs)) / 2 + 0.5) * REGION_PX;
  camera.value = { x: centerX, z: centerZ, scale: 0.5 };
}

/**
 * 初始取景：有人在就把镜头压到玩家身上，没人才退回 region 中心。
 *
 * 只按 region 外接框取中心有个致命问题：世界通常正好是
 * r.-1.-1 / r.-1.0 / r.0.-1 / r.0.0 四块，算出来的中心恒等于 (0, 0)，
 * 也就是出生点。玩家明明在别处，一进来看到的却永远是出生点视角。
 */
function frameOnFocus(list: WorldMapRegion[]) {
  const first = players.value[0];
  if (first) {
    camera.value = { x: first.x, z: first.z, scale: 1 };
    return;
  }
  if (list.length > 0) centerOnRegions(list);
}

/** 重置视图：退出跟随，重新取一次景。 */
function resetView() {
  followPlayer.value = false;
  if (players.value.length === 0 && regions.value.length === 0) {
    camera.value = { x: 0, z: 0, scale: 1 };
    return;
  }
  frameOnFocus(regions.value);
}

function zoomBy(factor: number, anchor?: { x: number; y: number }) {
  const next = Math.min(MAX_SCALE, Math.max(MIN_SCALE, camera.value.scale * factor));
  if (next === camera.value.scale) return;
  const anchorX = anchor?.x ?? viewport.value.width / 2;
  const anchorY = anchor?.y ?? viewport.value.height / 2;
  // 缩放围绕锚点：先算出锚点对应的世界坐标，缩放后再让中心平移回去
  const worldX = camera.value.x + (anchorX - viewport.value.width / 2) / camera.value.scale;
  const worldZ = camera.value.z + (anchorY - viewport.value.height / 2) / camera.value.scale;
  camera.value = {
    x: worldX - (anchorX - viewport.value.width / 2) / next,
    z: worldZ - (anchorY - viewport.value.height / 2) / next,
    scale: next,
  };
}

function onWheel(event: WheelEvent) {
  event.preventDefault();
  const rect = viewportEl.value?.getBoundingClientRect();
  const anchor = rect ? { x: event.clientX - rect.left, y: event.clientY - rect.top } : undefined;
  // 指数缩放：滚轮每格约 1.15 倍，快慢手感一致
  zoomBy(Math.exp(-event.deltaY * 0.0015), anchor);
}

let dragStart = { x: 0, y: 0, camX: 0, camZ: 0 };

function onPointerDown(event: PointerEvent) {
  if (event.button !== 0) return;
  // 手动拖动即视为「我要自己看」，立刻退出跟随，免得镜头一秒后被拽回去
  followPlayer.value = false;
  dragging.value = true;
  dragStart = { x: event.clientX, y: event.clientY, camX: camera.value.x, camZ: camera.value.z };
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
}

function onPointerMove(event: PointerEvent) {
  const rect = viewportEl.value?.getBoundingClientRect();
  if (rect) {
    pointerWorld.value = {
      x: Math.round(
        camera.value.x +
          (event.clientX - rect.left - viewport.value.width / 2) / camera.value.scale,
      ),
      z: Math.round(
        camera.value.z +
          (event.clientY - rect.top - viewport.value.height / 2) / camera.value.scale,
      ),
    };
  }
  if (!dragging.value) return;
  camera.value = {
    ...camera.value,
    x: dragStart.camX - (event.clientX - dragStart.x) / camera.value.scale,
    z: dragStart.camZ - (event.clientY - dragStart.y) / camera.value.scale,
  };
}

function onPointerUp(event: PointerEvent) {
  dragging.value = false;
  const target = event.currentTarget as HTMLElement;
  if (target.hasPointerCapture(event.pointerId)) target.releasePointerCapture(event.pointerId);
}

/** 把错误归类，前端才能给出「该去改哪」的提示。 */
function classifyPlayerError(raw: string): string {
  const text = raw.toLowerCase();
  if (text.includes("rcon_disabled") || text.includes("not enabled")) return "disabled";
  if (text.includes("unauthorized") || text.includes("authentication")) return "unauthorized";
  if (text.includes("unreachable") || text.includes("connect") || text.includes("timed out")) {
    return "unreachable";
  }
  return "unknown";
}

async function pollPlayers() {
  if (!instanceId.value || document.hidden) return;
  const current = generation;
  try {
    const snapshot = await mapApi.loadPlayerPositions(instanceId.value, dimension.value);
    if (current !== generation) return;
    players.value = snapshot.players;
    playerUpdatedAt.value = snapshot.captured_at_unix_secs;
    playerErrorKind.value = "";
    playerErrorText.value = "";
    applyFollow();
  } catch (error) {
    if (current !== generation) return;
    const raw = describeError(error);
    playerErrorKind.value = classifyPlayerError(raw);
    playerErrorText.value = raw;
    players.value = [];
  }
}

/** 跟随模式：把镜头钉在第一个在线玩家身上，缩放保持不变。 */
function applyFollow() {
  if (!followPlayer.value) return;
  const first = players.value[0];
  if (!first) return;
  camera.value = { x: first.x, z: first.z, scale: camera.value.scale };
}

/** 镜头对准第一个在线玩家。 */
function focusFirstPlayer() {
  const first = players.value[0];
  if (!first) return;
  camera.value = { x: first.x, z: first.z, scale: Math.max(camera.value.scale, 1) };
}

let playerTimer: number | undefined;
let regionTimer: number | undefined;
let resizeObserver: ResizeObserver | undefined;

function restartTimers() {
  window.clearInterval(playerTimer);
  window.clearInterval(regionTimer);
  playerTimer = window.setInterval(() => void pollPlayers(), PLAYER_POLL_MS);
  regionTimer = window.setInterval(() => void refreshRegions(false), REGION_REFRESH_MS);
}

/** 实例或维度变化：清空旧图、重置缓存代次、重新加载。 */
async function resetForTarget() {
  generation += 1;
  queue.length = 0;
  queued.clear();
  activeTiles = 0;
  pendingTiles.value = 0;
  revokeAllTiles();
  regions.value = [];
  players.value = [];
  playerErrorKind.value = "";
  playerErrorText.value = "";
  errorText.value = "";
  // 先问玩家在哪，再取景——顺序反了就只能退回出生点中心
  await pollPlayers();
  await refreshRegions(true);
}

watch([instanceId, dimension], () => void resetForTarget());

onMounted(async () => {
  if (viewportEl.value) {
    resizeObserver = new ResizeObserver((entries) => {
      const box = entries[0]?.contentRect;
      if (box) viewport.value = { width: box.width, height: box.height };
    });
    resizeObserver.observe(viewportEl.value);
    const rect = viewportEl.value.getBoundingClientRect();
    viewport.value = { width: rect.width, height: rect.height };
  }
  await loadInstances();
  restartTimers();
});

onBeforeUnmount(() => {
  window.clearInterval(playerTimer);
  window.clearInterval(regionTimer);
  resizeObserver?.disconnect();
  generation += 1;
  revokeAllTiles();
});
</script>

<template>
  <div class="world-map-view animate-stagger-in">
    <header class="page-header">
      <div>
        <h1 class="page-title">{{ i18n.t("world_map.title") }}</h1>
        <p class="page-desc">
          {{ i18n.t("world_map.subtitle") }}
          <span v-if="levelName" class="level-name">
            {{ i18n.t("world_map.level", { name: levelName }) }}
          </span>
        </p>
      </div>
      <div class="header-stats">
        <span class="badge">{{ i18n.t("world_map.players", { count: players.length }) }}</span>
        <span v-if="pendingTiles > 0" class="badge badge-busy">
          {{ i18n.t("world_map.rendering", { count: pendingTiles }) }}
        </span>
        <span class="badge badge-muted">{{ zoomPercent }}%</span>
      </div>
    </header>

    <section class="toolbar">
      <label class="field">
        <span class="field-label">{{ i18n.t("world_map.instance") }}</span>
        <select v-model="instanceId" class="control">
          <option v-if="instances.length === 0" value="">—</option>
          <option v-for="item in instances" :key="item.id" :value="item.id">{{ item.name }}</option>
        </select>
      </label>

      <div class="segmented">
        <button
          v-for="option in dimensionOptions"
          :key="option.value"
          class="segment"
          :class="{ active: dimension === option.value }"
          @click="dimension = option.value"
        >
          {{ option.label }}
        </button>
      </div>

      <div class="toolbar-actions">
        <button class="ghost" :title="i18n.t('world_map.zoom_out')" @click="zoomBy(1 / 1.3)">
          −
        </button>
        <button class="ghost" :title="i18n.t('world_map.zoom_in')" @click="zoomBy(1.3)">＋</button>
        <button class="ghost" @click="resetView">{{ i18n.t("world_map.reset_view") }}</button>
        <button class="ghost" :disabled="players.length === 0" @click="focusFirstPlayer">
          {{ i18n.t("world_map.locate_player") }}
        </button>
        <button
          class="ghost"
          :class="{ active: followPlayer }"
          :disabled="players.length === 0"
          :title="i18n.t('world_map.follow_player')"
          @click="followPlayer = !followPlayer"
        >
          {{ i18n.t("world_map.follow_player") }}
        </button>
      </div>
    </section>

    <section
      ref="viewportEl"
      class="viewport"
      :class="{ dragging }"
      @wheel="onWheel"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
      @pointerleave="pointerWorld = null"
    >
      <!-- region 瓦片层 -->
      <img
        v-for="tile in visibleTiles"
        :key="tile.key"
        v-show="tile.url"
        class="tile"
        :src="tile.url"
        :style="{
          left: `${tile.left}px`,
          top: `${tile.top}px`,
          width: `${tile.size}px`,
          height: `${tile.size}px`,
        }"
        :alt="`r.${tile.regionX}.${tile.regionZ}`"
        draggable="false"
      />
      <!-- 还没渲染好的 region 用虚线框占位，让人知道正在干活 -->
      <div
        v-for="tile in visibleTiles.filter((item) => !item.url)"
        :key="`placeholder-${tile.key}`"
        class="tile-placeholder"
        :style="{
          left: `${tile.left}px`,
          top: `${tile.top}px`,
          width: `${tile.size}px`,
          height: `${tile.size}px`,
        }"
      >
        <span>r.{{ tile.regionX }}.{{ tile.regionZ }}</span>
      </div>

      <!-- 玩家层 -->
      <div
        v-for="marker in playerMarkers"
        :key="marker.name"
        class="marker"
        :style="{ left: `${marker.left}px`, top: `${marker.top}px` }"
        :title="`${marker.name} · Y=${marker.y}`"
      >
        <svg
          class="marker-arrow"
          viewBox="0 0 24 24"
          :style="{ transform: `rotate(${marker.rotation}deg)` }"
        >
          <path d="M12 2 L19 22 L12 17 L5 22 Z" />
        </svg>
        <span class="marker-name">{{ marker.name }}</span>
      </div>

      <p v-if="regions.length === 0 && !regionLoading" class="viewport-empty">
        {{ i18n.t("world_map.no_regions") }}
      </p>

      <div class="viewport-footer">
        <span v-if="pointerWorld" class="coord">
          X {{ pointerWorld.x }} · Z {{ pointerWorld.z }}
        </span>
      </div>
    </section>

    <p v-if="errorText" class="error">{{ errorText }}</p>

    <p v-if="playerErrorKind" class="notice">
      <template v-if="playerErrorKind === 'disabled'">{{
        i18n.t("world_map.rcon_disabled")
      }}</template>
      <template v-else-if="playerErrorKind === 'unreachable'">
        {{ i18n.t("world_map.rcon_unreachable") }}
      </template>
      <template v-else-if="playerErrorKind === 'unauthorized'">
        {{ i18n.t("world_map.rcon_unauthorized") }}
      </template>
      <template v-else>{{ playerErrorText }}</template>
    </p>
    <p v-else-if="players.length === 0" class="hint">{{ i18n.t("world_map.no_players") }}</p>
  </div>
</template>

<style scoped>
.world-map-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-lg);
  height: 100%;
  min-height: 0;
}

.page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--sl-space-md);
}

.page-title {
  margin: 0 0 var(--sl-space-xs);
  font-size: var(--sl-font-size-2xl);
  color: var(--sl-text-primary);
}

.page-desc {
  margin: 0;
  font-size: var(--sl-font-size-sm);
  color: var(--sl-text-secondary);
}

.level-name {
  margin-left: var(--sl-space-xs);
  color: var(--sl-text-tertiary);
}

.header-stats {
  display: flex;
  gap: var(--sl-space-xs);
  flex-shrink: 0;
}

.badge {
  padding: 2px var(--sl-space-sm);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-secondary);
  font-variant-numeric: tabular-nums;
}

.badge-busy {
  color: var(--sl-primary);
  border-color: var(--sl-primary);
}

.badge-muted {
  color: var(--sl-text-tertiary);
}

.toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-end;
  gap: var(--sl-space-sm);
}

.field {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
  min-width: 180px;
}

.field-label {
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-tertiary);
}

.control {
  height: 32px;
  padding: 0 var(--sl-space-sm);
  background: var(--sl-bg-secondary);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  color: var(--sl-text-primary);
  font-size: var(--sl-font-size-sm);
}

.segmented {
  display: flex;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  overflow: hidden;
}

.segment {
  height: 32px;
  padding: 0 var(--sl-space-md);
  background: var(--sl-bg-secondary);
  border: none;
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-sm);
  cursor: pointer;
}

.segment + .segment {
  border-left: 1px solid var(--sl-border);
}

.segment.active {
  background: var(--sl-primary);
  color: var(--sl-text-inverse);
}

.toolbar-actions {
  display: flex;
  gap: var(--sl-space-xs);
  margin-left: auto;
}

.ghost {
  height: 32px;
  min-width: 32px;
  padding: 0 var(--sl-space-sm);
  background: var(--sl-bg-secondary);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-sm);
  cursor: pointer;
}

.ghost:hover:not(:disabled) {
  color: var(--sl-text-primary);
  border-color: var(--sl-primary);
}

.ghost.active {
  color: var(--sl-text-inverse);
  border-color: var(--sl-primary);
  background: var(--sl-primary);
}

.ghost:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.viewport {
  position: relative;
  flex: 1;
  min-height: 320px;
  overflow: hidden;
  background: var(--sl-bg-secondary);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  cursor: grab;
  touch-action: none;
  user-select: none;
}

.viewport.dragging {
  cursor: grabbing;
}

.tile {
  position: absolute;
  image-rendering: pixelated;
  pointer-events: none;
}

.tile-placeholder {
  position: absolute;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px dashed var(--sl-border);
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-xs);
  pointer-events: none;
}

.marker {
  position: absolute;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  pointer-events: none;
}

.marker-arrow {
  width: 22px;
  height: 22px;
  fill: var(--sl-primary);
  stroke: var(--sl-bg);
  stroke-width: 1.2;
  filter: drop-shadow(0 0 2px rgba(0, 0, 0, 0.6));
}

.marker-name {
  margin-top: 2px;
  padding: 0 4px;
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-primary);
  background: color-mix(in srgb, var(--sl-bg) 70%, transparent);
  border-radius: var(--sl-radius-sm);
  white-space: nowrap;
}

.viewport-empty {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0;
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-sm);
}

.viewport-footer {
  position: absolute;
  left: var(--sl-space-sm);
  bottom: var(--sl-space-sm);
  pointer-events: none;
}

.coord {
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-tertiary);
  font-variant-numeric: tabular-nums;
}

.error {
  margin: 0;
  font-size: var(--sl-font-size-sm);
  color: var(--sl-error);
}

.notice {
  margin: 0;
  font-size: var(--sl-font-size-sm);
  color: var(--sl-notice);
}

.hint {
  margin: 0;
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-tertiary);
}
</style>
