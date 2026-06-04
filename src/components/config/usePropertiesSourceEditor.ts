import { computed, onBeforeUnmount, onMounted, ref, watch, type Ref } from "vue";
import { SearchQuery, findNext, findPrevious, search, setSearchQuery } from "@codemirror/search";
import { EditorState } from "@codemirror/state";
import { EditorView, lineNumbers } from "@codemirror/view";
import {
  propertiesLanguage,
  propertiesSyntaxHighlighting,
} from "@components/config/propertiesCodeMirror";

interface UsePropertiesSourceEditorOptions {
  editorRoot: Ref<HTMLElement | null>;
  modelValue: Ref<string>;
  readOnly: Ref<boolean>;
  onUpdateModelValue: (value: string) => void;
}

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

export function usePropertiesSourceEditor(options: UsePropertiesSourceEditorOptions) {
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

  function createEditor() {
    if (!options.editorRoot.value) {
      return;
    }

    const state = EditorState.create({
      doc: options.modelValue.value,
      extensions: [
        lineNumbers(),
        EditorView.lineWrapping,
        EditorState.readOnly.of(!!options.readOnly.value),
        propertiesLanguage,
        propertiesSyntaxHighlighting,
        search({ top: false }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            options.onUpdateModelValue(update.state.doc.toString());
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
            height: "480px",
            overflow: "hidden",
          },
          ".cm-scroller": {
            fontFamily: "var(--sl-font-mono)",
            fontSize: "var(--sl-font-size-sm)",
            lineHeight: "1.45",
            padding: "0",
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

    editorView = new EditorView({ state, parent: options.editorRoot.value });
    applySearchQuery();
  }

  onMounted(() => {
    createEditor();
  });

  watch(options.modelValue, (value) => {
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
  });

  watch(searchText, () => {
    applySearchQuery();
  });

  onBeforeUnmount(() => {
    editorView?.destroy();
    editorView = null;
  });

  return {
    searchText,
    matchCountText,
    canNavigate,
    navigateToPrevious,
    navigateToNext,
  };
}
