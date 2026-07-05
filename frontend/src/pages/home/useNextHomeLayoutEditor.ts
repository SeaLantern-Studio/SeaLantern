import { computed, onMounted, onUnmounted, shallowRef, watch, toValue } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";
import { useSettingsStore } from "@src/stores/settingsStore";
import {
  HOME_GRID_COLUMNS,
  HOME_GRID_GAP,
  HOME_GRID_MAX_ROWS,
  HOME_GRID_ROW_HEIGHT,
  NEXT_HOME_CARD_LAYOUTS,
  NEXT_HOME_DEFAULT_LAYOUT,
} from "./layoutContract";
import type {
  NextHomeCardInstance,
  NextHomeCardKind,
  NextHomeCardLayoutMeta,
  NextHomeCardRegistry,
} from "./layoutContract";
import { resolveNextHomeCardTitle } from "./cardMeta";

const STORAGE_KEY = "sealantern.next.home.layout.v1";
const DEFAULT_MAX_INSTANCES = 5;
const SETTINGS_SAVE_DEBOUNCE_MS = 180;
const SECTION_ORDER = ["summary", "operations", "workspace", "attention"] as const;

interface UseNextHomeLayoutEditorOptions {
  editMode: Ref<boolean>;
  additionalRegistry?: MaybeRefOrGetter<NextHomeCardLayoutMeta[]>;
}

interface NextHomeCardPaletteEntry {
  kind: NextHomeCardKind;
  meta: NextHomeCardLayoutMeta;
  count: number;
  limit: number;
  canAdd: boolean;
}

interface NextHomePlacementRequest {
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  colStart?: number;
  rowStart?: number;
  colSpan?: number;
  rowSpan?: number;
}

type PersistedNextHomeCardInstance = Partial<Omit<NextHomeCardInstance, "kind">> & {
  kind?: string;
};

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function syncGridMetrics(instance: NextHomeCardInstance): NextHomeCardInstance {
  return {
    ...instance,
    colStart: Math.floor(instance.x) + 1,
    rowStart: Math.floor(instance.y) + 1,
    colSpan: Math.max(1, Math.ceil(instance.width)),
    rowSpan: Math.max(1, Math.ceil(instance.height)),
  };
}

function intersects(a: NextHomeCardInstance, b: NextHomeCardInstance): boolean {
  const aColEnd = a.colStart + a.colSpan;
  const bColEnd = b.colStart + b.colSpan;
  const aRowEnd = a.rowStart + a.rowSpan;
  const bRowEnd = b.rowStart + b.rowSpan;
  return (
    a.colStart < bColEnd && aColEnd > b.colStart && a.rowStart < bRowEnd && aRowEnd > b.rowStart
  );
}

function normalizeSpan(
  value: number | undefined,
  min: number,
  fallback: number,
  max: number,
): number {
  const source = Number.isFinite(value) ? Math.floor(value as number) : fallback;
  return clamp(source, min, max);
}

function normalizeStart(value: number | undefined, fallback: number, maxStart: number): number {
  const source = Number.isFinite(value) ? Math.floor(value as number) : fallback;
  return clamp(source, 1, Math.max(1, maxStart));
}

function buildInstanceId(kind: NextHomeCardKind): string {
  return `${kind}-${Math.random().toString(36).slice(2, 10)}`;
}

function resolveMeta(
  registry: NextHomeCardRegistry,
  kind: NextHomeCardKind,
): NextHomeCardLayoutMeta | null {
  return registry[kind] ?? null;
}

function sanitizeInstance(
  raw: PersistedNextHomeCardInstance,
  registry: NextHomeCardRegistry,
): NextHomeCardInstance | null {
  // Persisted layout data may come from older schemas or manual edits in localStorage.
  // Sanitize every instance through the current registry so bad coordinates never enter
  // the editor state and future registry changes can still hydrate old layouts safely.
  if (typeof raw.kind !== "string" || raw.kind.length === 0) return null;
  const kind = raw.kind as NextHomeCardKind;
  const meta = resolveMeta(registry, kind);
  if (!meta) return null;

  const maxCols = meta.maxCols ?? HOME_GRID_COLUMNS;
  const maxRows = meta.maxRows ?? HOME_GRID_MAX_ROWS;
  const width = clamp(
    Number.isFinite(raw.width)
      ? Number(raw.width)
      : normalizeSpan(raw.colSpan, meta.minCols, meta.defaultCols, maxCols),
    meta.minCols,
    maxCols,
  );
  const height = clamp(
    Number.isFinite(raw.height)
      ? Number(raw.height)
      : normalizeSpan(raw.rowSpan, meta.minRows, meta.defaultRows, maxRows),
    meta.minRows,
    maxRows,
  );
  const x = clamp(
    Number.isFinite(raw.x)
      ? Number(raw.x)
      : normalizeStart(raw.colStart, 1, HOME_GRID_COLUMNS - Math.ceil(width) + 1) - 1,
    0,
    HOME_GRID_COLUMNS - width,
  );
  const y = clamp(
    Number.isFinite(raw.y)
      ? Number(raw.y)
      : normalizeStart(raw.rowStart, 1, HOME_GRID_MAX_ROWS - Math.ceil(height) + 1) - 1,
    0,
    HOME_GRID_MAX_ROWS - height,
  );

  return syncGridMetrics({
    instanceId:
      typeof raw.instanceId === "string" && raw.instanceId.length > 0
        ? raw.instanceId
        : buildInstanceId(kind),
    kind,
    x,
    y,
    width,
    height,
    colStart: 1,
    rowStart: 1,
    colSpan: 1,
    rowSpan: 1,
    zIndex: Number.isFinite(raw.zIndex) ? Math.floor(raw.zIndex as number) : 1,
  });
}

function findPlacement(
  kind: NextHomeCardKind,
  registry: NextHomeCardRegistry,
  existing: NextHomeCardInstance[],
  preferred?: NextHomePlacementRequest,
): NextHomePlacementRequest | null {
  // Placement is first-fit on the normalized grid, not geometric packing. That keeps
  // behavior deterministic across drag, paste, reset, and future registry migrations.
  const meta = resolveMeta(registry, kind);
  if (!meta) return null;

  const maxCols = meta.maxCols ?? HOME_GRID_COLUMNS;
  const maxRows = meta.maxRows ?? HOME_GRID_MAX_ROWS;
  const colSpan = normalizeSpan(preferred?.colSpan, meta.minCols, meta.defaultCols, maxCols);
  const rowSpan = normalizeSpan(preferred?.rowSpan, meta.minRows, meta.defaultRows, maxRows);

  const tryPlacement = (colStart: number, rowStart: number): NextHomePlacementRequest | null => {
    const candidate = sanitizeInstance(
      { instanceId: "candidate", kind, colStart, rowStart, colSpan, rowSpan },
      registry,
    );
    if (!candidate) return null;
    const blocked = existing.some((instance) => intersects(candidate, instance));
    if (blocked) return null;
    return {
      x: candidate.x,
      y: candidate.y,
      width: candidate.width,
      height: candidate.height,
      colStart: candidate.colStart,
      rowStart: candidate.rowStart,
      colSpan: candidate.colSpan,
      rowSpan: candidate.rowSpan,
    };
  };

  if (preferred?.colStart && preferred?.rowStart) {
    const preferredPlacement = tryPlacement(preferred.colStart, preferred.rowStart);
    if (preferredPlacement) return preferredPlacement;
  }

  for (let rowStart = 1; rowStart <= HOME_GRID_MAX_ROWS - rowSpan + 1; rowStart += 1) {
    for (let colStart = 1; colStart <= HOME_GRID_COLUMNS - colSpan + 1; colStart += 1) {
      const placement = tryPlacement(colStart, rowStart);
      if (placement) return placement;
    }
  }

  return null;
}

function resolveSnapPlacementForInstance(
  source: NextHomeCardInstance,
  registry: NextHomeCardRegistry,
  existing: NextHomeCardInstance[],
  preferred: NextHomePlacementRequest,
): NextHomePlacementRequest | null {
  return findPlacement(
    source.kind,
    registry,
    existing.filter((instance) => instance.instanceId !== source.instanceId),
    {
      colStart: preferred.colStart ?? source.colStart,
      rowStart: preferred.rowStart ?? source.rowStart,
      colSpan: preferred.colSpan ?? source.colSpan,
      rowSpan: preferred.rowSpan ?? source.rowSpan,
    },
  );
}

function buildSnapPlacement(
  source: NextHomeCardInstance,
  patch: NextHomePlacementRequest,
): NextHomePlacementRequest {
  const colStart = patch.colStart ?? source.colStart;
  const rowStart = patch.rowStart ?? source.rowStart;
  const colSpan = patch.colSpan ?? source.colSpan;
  const rowSpan = patch.rowSpan ?? source.rowSpan;

  return {
    x: colStart - 1,
    y: rowStart - 1,
    width: colSpan,
    height: rowSpan,
    colStart,
    rowStart,
    colSpan,
    rowSpan,
  };
}

function resolveAutoHeightLayout(
  registry: NextHomeCardRegistry,
  current: NextHomeCardInstance[],
  candidate: NextHomeCardInstance,
): NextHomeCardInstance[] {
  const pinned = current
    .filter((instance) => instance.instanceId !== candidate.instanceId)
    .map((instance) => ({ ...instance }));

  const result = [...pinned, candidate].toSorted((left, right) => {
    if (left.instanceId === candidate.instanceId) return -1;
    if (right.instanceId === candidate.instanceId) return 1;
    if (left.rowStart !== right.rowStart) return left.rowStart - right.rowStart;
    if (left.colStart !== right.colStart) return left.colStart - right.colStart;
    return left.instanceId.localeCompare(right.instanceId);
  });

  for (let index = 0; index < result.length; index += 1) {
    let currentInstance = result[index];
    let guard = 0;
    while (
      result.some(
        (other, otherIndex) => otherIndex < index && intersects(currentInstance, other),
      ) &&
      guard < HOME_GRID_MAX_ROWS
    ) {
      const blockers = result.filter(
        (other, otherIndex) => otherIndex < index && intersects(currentInstance, other),
      );
      const nextRowStart = blockers.reduce(
        (maxValue, blocker) => Math.max(maxValue, blocker.rowStart + blocker.rowSpan),
        currentInstance.rowStart + 1,
      );
      const shifted = sanitizeInstance(
        {
          ...currentInstance,
          rowStart: nextRowStart,
          y: nextRowStart - 1,
        },
        registry,
      );
      if (!shifted) break;
      currentInstance = shifted;
      result[index] = currentInstance;
      guard += 1;
    }
  }

  return result;
}

export function useNextHomeLayoutEditor(options: UseNextHomeLayoutEditorOptions) {
  const settingsStore = useSettingsStore();
  const instances = shallowRef<NextHomeCardInstance[]>([]);
  const selectedInstanceId = shallowRef<string | null>(null);
  const clipboardInstance = shallowRef<NextHomeCardInstance | null>(null);
  const snapMode = shallowRef(false);
  const isHydratingFromSettings = shallowRef(false);
  const hasHydrated = shallowRef(false);
  const lastPersistedSnapshot = shallowRef("[]");

  let pendingSettingsSaveTimer: ReturnType<typeof window.setTimeout> | null = null;

  const registry = computed<NextHomeCardRegistry>(() => {
    const dynamicEntries = toValue(options.additionalRegistry) ?? [];
    const dynamicRegistry = Object.fromEntries(dynamicEntries.map((entry) => [entry.id, entry]));
    return { ...NEXT_HOME_CARD_LAYOUTS, ...dynamicRegistry } as NextHomeCardRegistry;
  });

  const countsByKind = computed<Record<string, number>>(() =>
    instances.value.reduce<Record<string, number>>((result, instance) => {
      result[instance.kind] = (result[instance.kind] ?? 0) + 1;
      return result;
    }, {}),
  );

  const sortedInstances = computed(() =>
    [...instances.value].toSorted((left, right) => {
      const isLeftSelected = left.instanceId === selectedInstanceId.value;
      const isRightSelected = right.instanceId === selectedInstanceId.value;
      if (isLeftSelected !== isRightSelected) return isLeftSelected ? 1 : -1;
      if (left.rowStart !== right.rowStart) return left.rowStart - right.rowStart;
      if (left.colStart !== right.colStart) return left.colStart - right.colStart;
      return left.instanceId.localeCompare(right.instanceId);
    }),
  );

  const paletteEntries = computed<NextHomeCardPaletteEntry[]>(() =>
    Object.values(registry.value)
      .toSorted((left, right) => {
        const leftSection = SECTION_ORDER.indexOf(left.section);
        const rightSection = SECTION_ORDER.indexOf(right.section);
        if (leftSection !== rightSection) return leftSection - rightSection;
        return resolveNextHomeCardTitle(left).localeCompare(
          resolveNextHomeCardTitle(right),
          "zh-CN",
        );
      })
      .map((meta) => {
        const count = countsByKind.value[meta.id] ?? 0;
        const limit = meta.maxInstances ?? DEFAULT_MAX_INSTANCES;
        return { kind: meta.id, meta, count, limit, canAdd: count < limit };
      }),
  );

  const selectedInstance = computed(() => {
    if (!selectedInstanceId.value) return null;
    return (
      instances.value.find((instance) => instance.instanceId === selectedInstanceId.value) ?? null
    );
  });

  function canAddKind(kind: NextHomeCardKind): boolean {
    const meta = resolveMeta(registry.value, kind);
    if (!meta) return false;
    const limit = meta.maxInstances ?? DEFAULT_MAX_INSTANCES;
    return (countsByKind.value[kind] ?? 0) < limit;
  }

  function replaceInstances(nextInstances: NextHomeCardInstance[]): void {
    instances.value = nextInstances;
  }

  function getNextZIndex(): number {
    return (
      instances.value.reduce((maxValue, instance) => Math.max(maxValue, instance.zIndex), 0) + 1
    );
  }

  function bringToFront(instanceId: string): void {
    const nextZIndex = getNextZIndex();
    replaceInstances(
      instances.value.map((instance) =>
        instance.instanceId === instanceId ? { ...instance, zIndex: nextZIndex } : instance,
      ),
    );
  }

  function selectInstance(instanceId: string | null): void {
    selectedInstanceId.value = instanceId;
    if (instanceId) bringToFront(instanceId);
  }

  function deployCard(
    kind: NextHomeCardKind,
    preferred?: NextHomePlacementRequest,
  ): NextHomeCardInstance | null {
    if (!canAddKind(kind)) return null;
    if (!snapMode.value && preferred?.x !== undefined && preferred?.y !== undefined) {
      const directInstance = sanitizeInstance(
        { instanceId: buildInstanceId(kind), kind, zIndex: getNextZIndex(), ...preferred },
        registry.value,
      );
      if (!directInstance) return null;
      replaceInstances([...instances.value, directInstance]);
      selectedInstanceId.value = directInstance.instanceId;
      return directInstance;
    }

    const placement = findPlacement(kind, registry.value, instances.value, preferred);
    if (!placement) return null;

    const nextInstance = sanitizeInstance(
      { instanceId: buildInstanceId(kind), kind, zIndex: getNextZIndex(), ...placement },
      registry.value,
    );
    if (!nextInstance) return null;

    replaceInstances([...instances.value, nextInstance]);
    selectedInstanceId.value = nextInstance.instanceId;
    return nextInstance;
  }

  function deployCardToViewportCenter(kind: NextHomeCardKind): NextHomeCardInstance | null {
    const meta = resolveMeta(registry.value, kind);
    if (!meta) return null;
    const viewportY =
      typeof window !== "undefined"
        ? (window.scrollY + window.innerHeight * 0.35) / (HOME_GRID_ROW_HEIGHT + HOME_GRID_GAP)
        : 2;
    const width = meta.defaultCols;
    const height = meta.defaultRows;
    return deployCard(kind, {
      x: (HOME_GRID_COLUMNS - width) / 2,
      y: viewportY,
      width,
      height,
      colStart: Math.floor((HOME_GRID_COLUMNS - width) / 2) + 1,
      rowStart: Math.floor(viewportY) + 1,
      colSpan: width,
      rowSpan: height,
    });
  }

  function removeInstance(instanceId: string): void {
    const instance = instances.value.find((item) => item.instanceId === instanceId);
    if (!instance) {
      return;
    }

    const meta = resolveMeta(registry.value, instance.kind);
    if (!meta?.removable) {
      return;
    }

    replaceInstances(instances.value.filter((entry) => entry.instanceId !== instanceId));
    if (selectedInstanceId.value === instanceId) selectedInstanceId.value = null;
  }

  function updateInstance(
    instanceId: string,
    updater: (
      source: NextHomeCardInstance,
      meta: NextHomeCardLayoutMeta,
    ) => NextHomePlacementRequest,
  ): boolean {
    const source = instances.value.find((instance) => instance.instanceId === instanceId);
    if (!source) return false;
    const meta = resolveMeta(registry.value, source.kind);
    if (!meta) return false;

    if (snapMode.value) {
      // Snap mode resolves against sibling instances before mutating state so the editor
      // either lands on a valid grid slot or keeps the previous placement unchanged.
      const requestedPlacement = updater(source, meta);
      const resolvedPlacement = resolveSnapPlacementForInstance(
        source,
        registry.value,
        instances.value,
        requestedPlacement,
      );
      if (!resolvedPlacement) return false;

      const candidate = sanitizeInstance({ ...source, ...resolvedPlacement }, registry.value);
      if (!candidate) return false;

      replaceInstances(
        instances.value.map((instance) =>
          instance.instanceId === source.instanceId
            ? { ...candidate, zIndex: getNextZIndex() }
            : instance,
        ),
      );
      return true;
    }

    const candidate = sanitizeInstance({ ...source, ...updater(source, meta) }, registry.value);
    if (!candidate) return false;

    replaceInstances(
      instances.value.map((instance) =>
        instance.instanceId === source.instanceId
          ? { ...candidate, zIndex: getNextZIndex() }
          : instance,
      ),
    );
    return true;
  }

  function moveInstance(
    instanceId: string,
    placement: { x?: number; y?: number; colStart?: number; rowStart?: number },
  ): boolean {
    return updateInstance(instanceId, (source) => {
      if (snapMode.value) {
        return buildSnapPlacement(source, {
          colStart: placement.colStart,
          rowStart: placement.rowStart,
        });
      }

      return {
        x: placement.x ?? source.x,
        y: placement.y ?? source.y,
        colStart: placement.colStart ?? source.colStart,
        rowStart: placement.rowStart ?? source.rowStart,
      };
    });
  }

  function resizeInstance(
    instanceId: string,
    size: { width?: number; height?: number; colSpan?: number; rowSpan?: number },
  ): boolean {
    return updateInstance(instanceId, (source) => {
      if (snapMode.value) {
        return buildSnapPlacement(source, {
          colSpan: size.colSpan,
          rowSpan: size.rowSpan,
        });
      }

      return {
        width: size.width ?? source.width,
        height: size.height ?? source.height,
        colSpan: size.colSpan ?? source.colSpan,
        rowSpan: size.rowSpan ?? source.rowSpan,
      };
    });
  }

  function syncAutoHeightInstance(instanceId: string, rowSpan: number): boolean {
    const source = instances.value.find((instance) => instance.instanceId === instanceId);
    if (!source) return false;
    const meta = resolveMeta(registry.value, source.kind);
    if (!meta?.autoHeight) return false;

    const maxRows = meta.maxRows ?? HOME_GRID_MAX_ROWS;
    const nextRowSpan = clamp(rowSpan, meta.minRows, maxRows);
    if (nextRowSpan === source.rowSpan) return false;

    const candidate = sanitizeInstance(
      {
        ...source,
        height: nextRowSpan,
        rowSpan: nextRowSpan,
      },
      registry.value,
    );
    if (!candidate) return false;

    const nextInstances = resolveAutoHeightLayout(registry.value, instances.value, candidate).map(
      (instance) =>
        instance.instanceId === candidate.instanceId
          ? { ...instance, zIndex: source.zIndex }
          : instance,
    );
    replaceInstances(nextInstances);
    return true;
  }

  function resetLayout(): void {
    const sanitizedDefaults = NEXT_HOME_DEFAULT_LAYOUT.map((instance) =>
      sanitizeInstance(instance, registry.value),
    ).filter((instance): instance is NextHomeCardInstance => instance !== null);
    replaceInstances(sanitizedDefaults);
    selectedInstanceId.value = sanitizedDefaults[0]?.instanceId ?? null;
  }

  function serializeInstances(value: NextHomeCardInstance[]): string {
    return JSON.stringify(value);
  }

  function sanitizePersistedLayout(
    rawLayout: PersistedNextHomeCardInstance[] | null | undefined,
    nextRegistry: NextHomeCardRegistry,
  ): NextHomeCardInstance[] {
    if (!Array.isArray(rawLayout)) {
      return [];
    }

    return rawLayout
      .map((instance) => sanitizeInstance(instance, nextRegistry))
      .filter((instance): instance is NextHomeCardInstance => instance !== null);
  }

  function clearPendingSettingsSave(): void {
    if (pendingSettingsSaveTimer !== null) {
      window.clearTimeout(pendingSettingsSaveTimer);
      pendingSettingsSaveTimer = null;
    }
  }

  async function persistLayoutToSettings(value: NextHomeCardInstance[]): Promise<void> {
    const snapshot = serializeInstances(value);
    if (snapshot === lastPersistedSnapshot.value) {
      return;
    }

    try {
      await settingsStore.updatePartial({ next_home_layout: value });
      lastPersistedSnapshot.value = snapshot;
    } catch (error) {
      console.error("Failed to persist next home layout via settings API.", error);
    }
  }

  async function migrateLegacyLayoutIfNeeded(nextRegistry: NextHomeCardRegistry): Promise<boolean> {
    if (typeof window === "undefined") {
      return false;
    }

    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return false;
    }

    try {
      const parsed = JSON.parse(raw) as PersistedNextHomeCardInstance[];
      const migrated = sanitizePersistedLayout(parsed, nextRegistry);
      if (migrated.length === 0) {
        window.localStorage.removeItem(STORAGE_KEY);
        return false;
      }

      replaceInstances(migrated);
      selectedInstanceId.value = migrated[0]?.instanceId ?? null;
      lastPersistedSnapshot.value = serializeInstances(migrated);
      await settingsStore.updatePartial({ next_home_layout: migrated });
      window.localStorage.removeItem(STORAGE_KEY);
      return true;
    } catch (error) {
      console.error("Failed to migrate legacy next home layout into settings API.", error);
      return false;
    }
  }

  function applySettingsLayout(
    rawLayout: PersistedNextHomeCardInstance[] | null | undefined,
    nextRegistry: NextHomeCardRegistry,
  ): void {
    const sanitized = sanitizePersistedLayout(rawLayout, nextRegistry);
    if (sanitized.length > 0) {
      replaceInstances(sanitized);
      selectedInstanceId.value = sanitized[0]?.instanceId ?? null;
      lastPersistedSnapshot.value = serializeInstances(sanitized);
      return;
    }

    resetLayout();
    lastPersistedSnapshot.value = serializeInstances(instances.value);
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (!options.editMode.value) return;

    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "c") {
      if (!selectedInstance.value) return;
      clipboardInstance.value = { ...selectedInstance.value };
      event.preventDefault();
      return;
    }

    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "v") {
      const source = selectedInstance.value ?? clipboardInstance.value;
      if (!source) return;
      deployCard(source.kind, {
        colStart: source.colStart + 1,
        rowStart: source.rowStart + 1,
        colSpan: source.colSpan,
        rowSpan: source.rowSpan,
      });
      event.preventDefault();
      return;
    }

    if ((event.key === "Delete" || event.key === "Backspace") && selectedInstance.value) {
      removeInstance(selectedInstance.value.instanceId);
      event.preventDefault();
      return;
    }

    if (event.key === "Escape") {
      selectedInstanceId.value = null;
    }
  }

  onMounted(async () => {
    if (typeof window === "undefined") {
      return;
    }

    window.addEventListener("keydown", handleKeydown);

    await settingsStore.ensureLoaded();

    isHydratingFromSettings.value = true;
    try {
      const persistedLayout = settingsStore.settings.next_home_layout;
      if (persistedLayout.length > 0) {
        applySettingsLayout(persistedLayout, registry.value);
      } else {
        const migrated = await migrateLegacyLayoutIfNeeded(registry.value);
        if (!migrated) {
          applySettingsLayout(null, registry.value);
        }
      }
    } finally {
      hasHydrated.value = true;
      isHydratingFromSettings.value = false;
    }
  });

  onUnmounted(() => {
    if (typeof window !== "undefined") {
      clearPendingSettingsSave();
      window.removeEventListener("keydown", handleKeydown);
    }
  });

  watch(
    registry,
    (nextRegistry) => {
      const sanitized = instances.value
        .map((instance) => sanitizeInstance(instance, nextRegistry))
        .filter((instance): instance is NextHomeCardInstance => instance !== null);

      if (sanitized.length !== instances.value.length) {
        replaceInstances(sanitized);
        if (
          selectedInstanceId.value &&
          !sanitized.some((instance) => instance.instanceId === selectedInstanceId.value)
        ) {
          selectedInstanceId.value = null;
        }
      }

      if (!hasHydrated.value || isHydratingFromSettings.value) {
        return;
      }

      const persistedLayout = settingsStore.settings.next_home_layout;
      const persistedSanitized = sanitizePersistedLayout(persistedLayout, nextRegistry);
      const persistedSnapshot = serializeInstances(persistedSanitized);
      const localSnapshot = serializeInstances(instances.value);

      if (persistedSnapshot === localSnapshot) {
        lastPersistedSnapshot.value = persistedSnapshot;
        return;
      }

      isHydratingFromSettings.value = true;
      try {
        if (persistedSanitized.length > 0) {
          replaceInstances(persistedSanitized);
          if (
            selectedInstanceId.value &&
            !persistedSanitized.some((instance) => instance.instanceId === selectedInstanceId.value)
          ) {
            selectedInstanceId.value = null;
          }
          lastPersistedSnapshot.value = persistedSnapshot;
        } else {
          resetLayout();
          lastPersistedSnapshot.value = serializeInstances(instances.value);
        }
      } finally {
        isHydratingFromSettings.value = false;
      }
    },
    { deep: true },
  );

  watch(
    instances,
    () => {
      if (typeof window === "undefined" || !hasHydrated.value || isHydratingFromSettings.value) {
        return;
      }

      clearPendingSettingsSave();
      pendingSettingsSaveTimer = window.setTimeout(() => {
        pendingSettingsSaveTimer = null;
        void persistLayoutToSettings(instances.value);
      }, SETTINGS_SAVE_DEBOUNCE_MS);
    },
    { deep: true },
  );

  watch(
    () => settingsStore.settings.next_home_layout,
    (value) => {
      if (!hasHydrated.value || isHydratingFromSettings.value) {
        return;
      }

      const sanitized = sanitizePersistedLayout(value, registry.value);
      const nextSnapshot = serializeInstances(sanitized);
      const currentSnapshot = serializeInstances(instances.value);
      if (nextSnapshot === currentSnapshot) {
        lastPersistedSnapshot.value = nextSnapshot;
        return;
      }

      isHydratingFromSettings.value = true;
      try {
        if (sanitized.length > 0) {
          replaceInstances(sanitized);
          if (
            selectedInstanceId.value &&
            !sanitized.some((instance) => instance.instanceId === selectedInstanceId.value)
          ) {
            selectedInstanceId.value = null;
          }
          lastPersistedSnapshot.value = nextSnapshot;
        } else {
          resetLayout();
          lastPersistedSnapshot.value = serializeInstances(instances.value);
        }
      } finally {
        isHydratingFromSettings.value = false;
      }
    },
    { deep: true },
  );

  watch(options.editMode, (isEditMode) => {
    if (!isEditMode) selectedInstanceId.value = null;
  });

  return {
    registry,
    instances: sortedInstances,
    paletteEntries,
    selectedInstanceId,
    selectedInstance,
    snapMode,
    selectInstance,
    bringToFront,
    deployCard,
    deployCardToViewportCenter,
    removeInstance,
    moveInstance,
    resizeInstance,
    syncAutoHeightInstance,
    canAddKind,
    resetLayout,
  };
}
