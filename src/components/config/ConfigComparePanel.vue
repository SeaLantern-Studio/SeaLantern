<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import { ArrowLeft } from "lucide-vue-next";
import SLTooltip from "@components/common/SLTooltip.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLButton from "@components/common/SLButton.vue";
import ConfigPropertyEditorControl from "@components/config/ConfigPropertyEditorControl.vue";
import { i18n } from "@language";

interface Option {
  label: string;
  value: string | number;
}

interface CompareControlState {
  key: string;
  value: string;
  valueType: string;
  defaultValue: string;
  numericError?: string;
}

interface ComparePanelRow {
  key: string;
  description: string;
  different: boolean;
  onlyInSource: boolean;
  onlyInTarget: boolean;
  hasSourceValue: boolean;
  hasTargetValue: boolean;
  source: CompareControlState;
  target: CompareControlState;
}

interface Props {
  compareTargetServerId: string;
  compareServerOptions: Option[];
  hasCompareTargets: boolean;
  compareLoading: boolean;
  inlineLabel: string;
  sourceServerName: string;
  targetServerName: string;
  differenceBadgeText: string;
  differentLabel: string;
  noDifferencesText: string;
  rows: ComparePanelRow[];
  gamemodeOptions: Option[];
  difficultyOptions: Option[];
}

const props = defineProps<Props>();

type CompareSide = "source" | "target";
type SwitchDirection = "to-source" | "to-target";

const activeSide = ref<CompareSide>("source");
const switchDirection = ref<SwitchDirection>("to-target");
const switchIconRotation = ref(0);
const isCompactCompare = ref(false);
const compareEntriesRef = ref<HTMLElement | null>(null);
let compareResizeObserver: ResizeObserver | null = null;

const updateCompareLayoutMode = () => {
  const compareWidth = compareEntriesRef.value?.clientWidth ?? 0;
  isCompactCompare.value = compareWidth > 0 && compareWidth < 760;
};

const compareTransitionName = computed(() =>
  switchDirection.value === "to-target" ? "compare-shift-left" : "compare-shift-right",
);

const switchButtonTitle = computed(() =>
  activeSide.value === "source" ? "切换到对照服务器" : "切换回当前服务器",
);

const switchButtonStyle = computed(() => ({
  "--compare-switch-rotate": `${switchIconRotation.value}deg`,
}));

const handleToggleSide = () => {
  if (activeSide.value === "source") {
    switchDirection.value = "to-target";
    activeSide.value = "target";
  } else {
    switchDirection.value = "to-source";
    activeSide.value = "source";
  }

  switchIconRotation.value -= 180;
};

onMounted(() => {
  if (typeof window === "undefined" || typeof ResizeObserver === "undefined") {
    return;
  }

  void nextTick(() => {
    updateCompareLayoutMode();
    if (!compareEntriesRef.value) {
      return;
    }

    compareResizeObserver = new ResizeObserver(() => {
      updateCompareLayoutMode();
    });
    compareResizeObserver.observe(compareEntriesRef.value);
  });
});

onBeforeUnmount(() => {
  compareResizeObserver?.disconnect();
});

const emit = defineEmits<{
  updateCompareTargetServer: [value: string | number];
  updateSourceValue: [payload: { key: string; value: string | boolean | number }];
  updateTargetValue: [payload: { key: string; value: string | boolean | number }];
  addSourceValue: [payload: { key: string; value: string | boolean | number }];
  addTargetValue: [payload: { key: string; value: string | boolean | number }];
}>();

const addPropertyButtonText = computed(() => {
  const key = "config.compare.add_this_property";
  const text = i18n.t(key);
  return text === key ? i18n.t("common.add") : text;
});

function getAddedInitialValue(row: ComparePanelRow, side: "source" | "target") {
  const control = side === "source" ? row.source : row.target;
  const opposite = side === "source" ? row.target : row.source;
  const oppositeHasValue = side === "source" ? row.hasTargetValue : row.hasSourceValue;

  if (oppositeHasValue) {
    return opposite.value;
  }

  if (control.valueType === "boolean") {
    return control.defaultValue === "true" ? "true" : "false";
  }

  return control.defaultValue ?? "";
}

function handleAddMissingProperty(row: ComparePanelRow, side: "source" | "target") {
  const value = getAddedInitialValue(row, side);
  if (side === "source") {
    emit("addSourceValue", { key: row.key, value });
    return;
  }

  emit("addTargetValue", { key: row.key, value });
}
</script>

<template>
  <div
    ref="compareEntriesRef"
    class="compare-entries"
    :class="{ 'compare-entries--compact': isCompactCompare }"
  >
    <div class="compare-header glass-card">
      <div class="compare-column-head compare-column-meta">
        <div class="compare-header-control">
          <span class="text-caption compare-target-label">{{ inlineLabel }}</span>
          <SLSelect
            :modelValue="compareTargetServerId"
            :options="compareServerOptions"
            :disabled="!hasCompareTargets || compareLoading"
            class="compare-target-select"
            @update:modelValue="emit('updateCompareTargetServer', $event)"
          />
        </div>
      </div>
      <div class="compare-column-head">
        <div class="compare-server-heading">
          <template v-if="isCompactCompare">
            <div class="compare-transition-stage compare-transition-stage--header">
              <Transition :name="compareTransitionName">
                <div
                  :key="`header-${activeSide}`"
                  class="compare-side-switch-content compare-transition-pane"
                >
                  <template v-if="activeSide === 'source'">
                    <span class="text-caption compare-server-title compare-side-switch-label">{{
                      sourceServerName
                    }}</span>
                  </template>
                  <template v-else>
                    <span class="text-caption compare-server-title compare-side-switch-label">{{
                      targetServerName
                    }}</span>
                    <span class="compare-count-badge">{{ differenceBadgeText }}</span>
                  </template>
                </div>
              </Transition>
            </div>
            <SLTooltip :content="switchButtonTitle" :delay="500">
              <button
                type="button"
                class="compare-side-switch-btn"
                :aria-label="switchButtonTitle"
                :style="switchButtonStyle"
                @click="handleToggleSide"
              >
                <span class="compare-side-switch-icon">
                  <ArrowLeft :size="14" />
                </span>
              </button>
            </SLTooltip>
          </template>
          <span v-else class="text-caption compare-server-title">{{ sourceServerName }}</span>
        </div>
      </div>
      <div class="compare-column-head">
        <div class="compare-server-heading">
          <span class="text-caption compare-server-title">{{ targetServerName }}</span>
          <span class="compare-count-badge">{{ differenceBadgeText }}</span>
        </div>
      </div>
    </div>

    <div
      v-for="row in rows"
      :key="row.key"
      class="compare-entry glass-card"
      :class="{
        different: row.different,
        'only-source': row.onlyInSource,
        'only-target': row.onlyInTarget,
      }"
    >
      <div class="compare-meta">
        <div class="entry-key-row">
          <span class="entry-key text-mono">{{ row.key }}</span>
          <span v-if="row.different" class="compare-diff-badge">
            {{ differentLabel }}
          </span>
        </div>
        <p v-if="row.description" class="entry-desc text-caption">
          {{ row.description }}
        </p>
      </div>
      <div v-if="isCompactCompare" class="compare-value-block compare-single-side-block">
        <div class="compare-transition-stage compare-entry-control">
          <Transition :name="compareTransitionName">
            <div :key="`${activeSide}-${row.key}`" class="compare-transition-pane">
              <ConfigPropertyEditorControl
                v-if="activeSide === 'source' && row.hasSourceValue"
                :propertyKey="row.source.key"
                :modelValue="row.source.value"
                :valueType="row.source.valueType"
                :defaultValue="row.source.defaultValue"
                :numericError="row.source.numericError"
                :gamemodeOptions="gamemodeOptions"
                :difficultyOptions="difficultyOptions"
                @update:modelValue="emit('updateSourceValue', { key: row.key, value: $event })"
              />
              <SLButton
                v-else-if="activeSide === 'source'"
                variant="secondary"
                size="sm"
                class="compare-add-property-btn"
                @click="handleAddMissingProperty(row, 'source')"
              >
                {{ addPropertyButtonText }}
              </SLButton>
              <ConfigPropertyEditorControl
                v-else-if="row.hasTargetValue"
                :propertyKey="row.target.key"
                :modelValue="row.target.value"
                :valueType="row.target.valueType"
                :defaultValue="row.target.defaultValue"
                :numericError="row.target.numericError"
                :gamemodeOptions="gamemodeOptions"
                :difficultyOptions="difficultyOptions"
                @update:modelValue="emit('updateTargetValue', { key: row.key, value: $event })"
              />
              <SLButton
                v-else
                variant="secondary"
                size="sm"
                class="compare-add-property-btn"
                @click="handleAddMissingProperty(row, 'target')"
              >
                {{ addPropertyButtonText }}
              </SLButton>
            </div>
          </Transition>
        </div>
      </div>
      <div v-if="!isCompactCompare" class="compare-value-block compare-source-block">
        <div class="entry-control compare-entry-control">
          <ConfigPropertyEditorControl
            v-if="row.hasSourceValue"
            :propertyKey="row.source.key"
            :modelValue="row.source.value"
            :valueType="row.source.valueType"
            :defaultValue="row.source.defaultValue"
            :numericError="row.source.numericError"
            :gamemodeOptions="gamemodeOptions"
            :difficultyOptions="difficultyOptions"
            @update:modelValue="emit('updateSourceValue', { key: row.key, value: $event })"
          />
          <SLButton
            v-else
            variant="secondary"
            size="sm"
            class="compare-add-property-btn"
            @click="handleAddMissingProperty(row, 'source')"
          >
            {{ addPropertyButtonText }}
          </SLButton>
        </div>
      </div>
      <div v-if="!isCompactCompare" class="compare-value-block compare-target-block">
        <div class="entry-control compare-entry-control">
          <ConfigPropertyEditorControl
            v-if="row.hasTargetValue"
            :propertyKey="row.target.key"
            :modelValue="row.target.value"
            :valueType="row.target.valueType"
            :defaultValue="row.target.defaultValue"
            :numericError="row.target.numericError"
            :gamemodeOptions="gamemodeOptions"
            :difficultyOptions="difficultyOptions"
            @update:modelValue="emit('updateTargetValue', { key: row.key, value: $event })"
          />
          <SLButton
            v-else
            variant="secondary"
            size="sm"
            class="compare-add-property-btn"
            @click="handleAddMissingProperty(row, 'target')"
          >
            {{ addPropertyButtonText }}
          </SLButton>
        </div>
      </div>
    </div>

    <div v-if="rows.length === 0" class="empty-state glass-card">
      <p class="text-caption">{{ noDifferencesText }}</p>
    </div>
  </div>
</template>
