<script setup lang="ts">
import { computed, ref } from "vue";
import { i18n } from "@language";
import SLButton from "@components/common/SLButton.vue";
import { ChevronDown, ChevronUp } from "@lucide/vue";
import { usePropertiesSourceEditor } from "@src/features/config-editor/usePropertiesSourceEditor";

interface Props {
  modelValue: string;
  title?: string;
  readOnly?: boolean;
  iconNavOnly?: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const editorRoot = ref<HTMLElement | null>(null);
const sourceEditor = usePropertiesSourceEditor({
  editorRoot,
  modelValue: computed(() => props.modelValue),
  readOnly: computed(() => !!props.readOnly),
  onUpdateModelValue: (value) => {
    emit("update:modelValue", value);
  },
});
</script>

<template>
  <div class="source-editor-panel">
    <div v-if="title" class="source-editor-title text-caption">{{ title }}</div>
    <div class="plugins-toolbar source-search-toolbar">
      <div class="toolbar-left">
        <input
          v-model="sourceEditor.searchText"
          type="text"
          class="plugin-search"
          :placeholder="i18n.t('config.source_search_placeholder')"
        />
        <span class="source-search-count text-caption">{{ sourceEditor.matchCountText }}</span>
      </div>
      <div class="toolbar-right">
        <template v-if="props.iconNavOnly">
          <SLButton
            variant="secondary"
            size="sm"
            iconOnly
            :disabled="!sourceEditor.canNavigate"
            :aria-label="i18n.t('config.source_search_prev')"
            @click="sourceEditor.navigateToPrevious"
          >
            <ChevronUp :size="14" />
          </SLButton>
          <SLButton
            variant="secondary"
            size="sm"
            iconOnly
            :disabled="!sourceEditor.canNavigate"
            :aria-label="i18n.t('config.source_search_next')"
            @click="sourceEditor.navigateToNext"
          >
            <ChevronDown :size="14" />
          </SLButton>
        </template>
        <template v-else>
          <SLButton
            variant="secondary"
            size="sm"
            :disabled="!sourceEditor.canNavigate"
            @click="sourceEditor.navigateToPrevious"
          >
            {{ i18n.t("config.source_search_prev") }}
          </SLButton>
          <SLButton
            variant="secondary"
            size="sm"
            :disabled="!sourceEditor.canNavigate"
            @click="sourceEditor.navigateToNext"
          >
            {{ i18n.t("config.source_search_next") }}
          </SLButton>
        </template>
      </div>
    </div>
    <div ref="editorRoot" class="source-cm-root"></div>
  </div>
</template>

<style scoped>
.source-editor-panel {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}

.source-editor-title {
  color: var(--sl-text-secondary);
  font-weight: 600;
}

.source-search-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: nowrap;
  gap: var(--sl-space-md);
  padding: var(--sl-space-xs);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  min-width: 0;
}

.toolbar-left {
  display: flex;
  flex: 1 1 auto;
  min-width: 0;
  flex-direction: row;
  flex-wrap: nowrap;
  align-items: center;
  gap: var(--sl-space-sm);
}

.toolbar-right {
  display: flex;
  flex: 0 0 auto;
  flex-wrap: nowrap;
  align-items: center;
  gap: var(--sl-space-sm);
}

.plugin-search {
  flex: 1 1 220px;
  width: auto;
  min-width: 0;
  padding: 6px 12px;
  border-radius: var(--sl-radius-sm);
  border: 1px solid var(--sl-border);
  background: var(--sl-bg-secondary);
  color: var(--sl-text-primary);
  font-size: 13px;
  transition: all var(--sl-transition-fast);
}

.plugin-search:focus {
  outline: none;
  border-color: var(--sl-primary);
}

.source-search-count {
  color: var(--sl-text-secondary);
  min-width: 64px;
  flex-shrink: 0;
  white-space: nowrap;
}

.source-cm-root :deep(.cm-editor) {
  width: 100%;
}
</style>
