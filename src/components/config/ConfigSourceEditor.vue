<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { i18n } from "@language";
import SLButton from "@components/common/SLButton.vue";
import { SearchQuery, findNext, findPrevious, search, setSearchQuery } from "@codemirror/search";
import { EditorState } from "@codemirror/state";
import { EditorView, lineNumbers } from "@codemirror/view";
import { HighlightStyle, StreamLanguage, syntaxHighlighting } from "@codemirror/language";
import { tags } from "@lezer/highlight";

interface Props {
  modelValue: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const editorRoot = ref<HTMLElement | null>(null);
const searchText = ref("");
const totalMatches = ref(0);
const currentMatch = ref(0);

let editorView: EditorView | null = null;

const searchQuery = computed(
  () =>
    new SearchQuery({
      search: searchText.value,
      caseSensitive: false,
      literal: true,
    }),
);

const matchCountText = computed(() => {
  if (!searchText.value) {
    return "0 / 0";
  }
  return `${currentMatch.value} / ${totalMatches.value}`;
});

const canNavigate = computed(() => totalMatches.value > 0);

const propertiesHighlightStyle = HighlightStyle.define([
  { tag: tags.comment, color: "var(--sl-text-tertiary)", fontStyle: "italic" },
  { tag: tags.propertyName, color: "var(--sl-primary)" },
  { tag: tags.definitionOperator, color: "var(--sl-text-secondary)" },
  { tag: tags.number, color: "var(--sl-warning)" },
  { tag: tags.bool, color: "var(--sl-success)" },
  { tag: tags.separator, color: "var(--sl-text-secondary)" },
]);

const propertiesLanguage = StreamLanguage.define<{ inValue: boolean }>({
  startState() {
    return { inValue: false };
  },
  token(stream, state) {
    if (stream.sol()) {
      state.inValue = false;
      stream.eatSpace();
      if (stream.peek() === "#") {
        stream.skipToEnd();
        return "comment";
      }
    }

    if (stream.eatSpace()) {
      return null;
    }

    const ch = stream.peek();
    if (ch === "=") {
      stream.next();
      state.inValue = true;
      return "operator";
    }

    if (ch === ",") {
      stream.next();
      return "comma";
    }

    if (!state.inValue) {
      stream.eatWhile((char) => char !== "=" && char !== "#" && char !== "\n");
      return "key";
    }

    if (stream.match(/(?:true|false)\b/i)) {
      return "boolean";
    }

    if (stream.match(/[+-]?\d+(?:\.\d+)?\b/)) {
      return "number";
    }

    stream.eatWhile((char) => char !== "," && char !== "#" && char !== "\n");
    return null;
  },
  tokenTable: {
    comment: tags.comment,
    operator: tags.definitionOperator,
    number: tags.number,
    boolean: tags.bool,
    comma: tags.separator,
    key: tags.propertyName,
  },
});

function getMatchRanges(text: string, query: string) {
  if (!query) {
    return [] as Array<{ from: number; to: number }>;
  }

  const ranges: Array<{ from: number; to: number }> = [];
  const haystack = text.toLowerCase();
  const needle = query.toLowerCase();
  let index = 0;

  while (index <= haystack.length - needle.length) {
    const found = haystack.indexOf(needle, index);
    if (found === -1) {
      break;
    }

    ranges.push({ from: found, to: found + needle.length });
    index = found + needle.length;
  }

  return ranges;
}

function updateMatchStats() {
  if (!editorView || !searchText.value) {
    totalMatches.value = 0;
    currentMatch.value = 0;
    return;
  }

  const text = editorView.state.doc.toString();
  const ranges = getMatchRanges(text, searchText.value);
  totalMatches.value = ranges.length;

  if (ranges.length === 0) {
    currentMatch.value = 0;
    return;
  }

  const selection = editorView.state.selection.main;
  const exactIndex = ranges.findIndex(
    (range) => range.from === selection.from && range.to === selection.to,
  );

  if (exactIndex !== -1) {
    currentMatch.value = exactIndex + 1;
    return;
  }

  const nearestIndex = ranges.findIndex((range) => range.from >= selection.to);
  currentMatch.value = nearestIndex === -1 ? ranges.length : nearestIndex + 1;
}

function applySearchQuery() {
  if (!editorView) {
    return;
  }

  editorView.dispatch({ effects: setSearchQuery.of(searchQuery.value) });
  updateMatchStats();
}

function navigateToPrevious() {
  if (!editorView || !canNavigate.value) {
    return;
  }

  editorView.focus();
  if (findPrevious(editorView)) {
    editorView.dispatch({ scrollIntoView: true });
    updateMatchStats();
  }
}

function navigateToNext() {
  if (!editorView || !canNavigate.value) {
    return;
  }

  editorView.focus();
  if (findNext(editorView)) {
    editorView.dispatch({ scrollIntoView: true });
    updateMatchStats();
  }
}

onMounted(() => {
  if (!editorRoot.value) {
    return;
  }

  const state = EditorState.create({
    doc: props.modelValue,
    extensions: [
      lineNumbers(),
      EditorView.lineWrapping,
      propertiesLanguage,
      syntaxHighlighting(propertiesHighlightStyle),
      search({ top: false }),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          emit("update:modelValue", update.state.doc.toString());
        }

        if (update.docChanged || update.selectionSet) {
          updateMatchStats();
        }
      }),
      EditorView.theme({
        "&": {
          border: "1px solid var(--sl-border-light)",
          borderRadius: "var(--sl-radius-md)",
          backgroundColor: "var(--sl-surface)",
          minHeight: "480px",
          overflow: "hidden",
        },
        ".cm-scroller": {
          fontFamily: "var(--sl-font-mono)",
          fontSize: "var(--sl-font-size-sm)",
          lineHeight: "1.45",
          padding: "8px 0",
        },
        ".cm-gutters": {
          backgroundColor: "var(--sl-bg-secondary)",
          color: "var(--sl-text-tertiary)",
          borderRight: "1px solid var(--sl-border-light)",
        },
        ".cm-lineNumbers .cm-gutterElement": {
          padding: "0 10px 0 12px",
        },
        ".cm-content": {
          color: "var(--sl-text-primary)",
          caretColor: "var(--sl-primary)",
          padding: "0 12px",
        },
        ".cm-activeLine": {
          backgroundColor: "transparent",
        },
        ".cm-searchMatch": {
          backgroundColor: "color-mix(in srgb, var(--sl-warning) 30%, transparent)",
          outline: "1px solid color-mix(in srgb, var(--sl-warning) 50%, transparent)",
        },
        ".cm-searchMatch.cm-searchMatch-selected": {
          backgroundColor: "color-mix(in srgb, var(--sl-primary) 25%, transparent)",
          outline: "1px solid color-mix(in srgb, var(--sl-primary) 45%, transparent)",
        },
        "&.cm-focused": {
          borderColor: "var(--sl-primary-light)",
          boxShadow: "0 0 0 2px var(--sl-primary-bg)",
        },
      }),
    ],
  });

  editorView = new EditorView({ state, parent: editorRoot.value });
  applySearchQuery();
});

watch(
  () => props.modelValue,
  (value) => {
    if (!editorView) {
      return;
    }

    const currentValue = editorView.state.doc.toString();
    if (value === currentValue) {
      return;
    }

    editorView.dispatch({
      changes: { from: 0, to: editorView.state.doc.length, insert: value },
    });
  },
);

watch(searchText, () => {
  applySearchQuery();
});

onBeforeUnmount(() => {
  editorView?.destroy();
  editorView = null;
});
</script>

<template>
  <div class="source-editor-panel">
    <div class="plugins-toolbar source-search-toolbar">
      <div class="toolbar-left">
        <input
          v-model="searchText"
          type="text"
          class="plugin-search"
          :placeholder="i18n.t('config.source_search_placeholder')"
        />
        <span class="source-search-count text-caption">{{ matchCountText }}</span>
      </div>
      <div class="toolbar-right">
        <SLButton
          variant="secondary"
          size="sm"
          :disabled="!canNavigate"
          @click="navigateToPrevious"
        >
          {{ i18n.t("config.source_search_prev") }}
        </SLButton>
        <SLButton variant="secondary" size="sm" :disabled="!canNavigate" @click="navigateToNext">
          {{ i18n.t("config.source_search_next") }}
        </SLButton>
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

.source-search-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-md);
  padding: var(--sl-space-xs);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}

.plugin-search {
  width: 220px;
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
}

.source-cm-root :deep(.cm-editor) {
  width: 100%;
}
</style>
