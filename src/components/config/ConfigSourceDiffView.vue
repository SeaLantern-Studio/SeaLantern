<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { EditorState, RangeSetBuilder, StateField } from "@codemirror/state";
import { Decoration, EditorView, GutterMarker, gutter } from "@codemirror/view";
import {
  propertiesLanguage,
  propertiesSyntaxHighlighting,
} from "@components/config/propertiesCodeMirror";
import { buildDiffLines, type DiffLine, type DiffLineType } from "@utils/configDiff";

interface Props {
  original: string;
  modified: string;
}

const props = defineProps<Props>();

const editorRoot = ref<HTMLElement | null>(null);

let editorView: EditorView | null = null;

class LineNumberMarker extends GutterMarker {
  constructor(
    private readonly value: number | null,
    private readonly type: DiffLineType,
  ) {
    super();
  }

  toDOM() {
    const element = document.createElement("span");
    element.className = `cm-diff-gutter-marker cm-diff-gutter-marker--${this.type}`;
    element.textContent = this.value === null ? "" : String(this.value);
    return element;
  }
}

function buildContextDiffLines(lines: DiffLine[], contextRadius: number): DiffLine[] {
  if (lines.length === 0) {
    return lines;
  }

  const changedIndices: number[] = [];
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (line.type === "addition" || line.type === "deletion") {
      changedIndices.push(index);
    }
  }

  if (changedIndices.length === 0) {
    return lines;
  }

  const windows = changedIndices.map((index) => ({
    start: Math.max(0, index - contextRadius),
    end: Math.min(lines.length - 1, index + contextRadius),
  }));

  const mergedWindows: Array<{ start: number; end: number }> = [];
  for (const window of windows) {
    const previous = mergedWindows[mergedWindows.length - 1];
    if (!previous || window.start > previous.end + 1) {
      mergedWindows.push({ ...window });
      continue;
    }
    previous.end = Math.max(previous.end, window.end);
  }

  const visibleLines: DiffLine[] = [];
  for (let index = 0; index < mergedWindows.length; index += 1) {
    const window = mergedWindows[index];
    visibleLines.push(...lines.slice(window.start, window.end + 1));

    if (mergedWindows[index + 1]) {
      visibleLines.push({
        type: "omitted",
        leftNumber: null,
        rightNumber: null,
        text: "...",
      });
    }
  }

  return visibleLines;
}

function createLineDecorations(lines: DiffLine[]) {
  return StateField.define({
    create(state) {
      const builder = new RangeSetBuilder<Decoration>();
      for (let index = 0; index < lines.length; index += 1) {
        const line = state.doc.line(index + 1);
        builder.add(
          line.from,
          line.from,
          Decoration.line({
            attributes: {
              class: `cm-diff-line cm-diff-line--${lines[index].type}`,
            },
          }),
        );
      }
      return builder.finish();
    },
    update(value) {
      return value;
    },
    provide(field) {
      return EditorView.decorations.from(field);
    },
  });
}

function createLineNumberGutter(lines: DiffLine[], side: "old" | "new") {
  const maxNumber = lines.reduce((max, line) => {
    const candidate = side === "old" ? (line.leftNumber ?? 0) : (line.rightNumber ?? 0);
    return Math.max(max, candidate);
  }, 0);

  return gutter({
    class: `cm-diff-gutter cm-diff-gutter--${side}`,
    renderEmptyElements: true,
    initialSpacer: () => new LineNumberMarker(maxNumber > 0 ? maxNumber : 1, "context"),
    lineMarker(view, line) {
      const lineNumber = view.state.doc.lineAt(line.from).number;
      const diffLine = lines[lineNumber - 1];
      const value =
        side === "old" ? (diffLine?.leftNumber ?? null) : (diffLine?.rightNumber ?? null);
      return new LineNumberMarker(value, diffLine?.type ?? "context");
    },
  });
}

function destroyEditor() {
  editorView?.destroy();
  editorView = null;
}

function createEditor() {
  if (!editorRoot.value) {
    return;
  }

  destroyEditor();

  const lines = buildContextDiffLines(buildDiffLines(props.original, props.modified), 3);
  const state = EditorState.create({
    doc: lines.map((line) => line.text).join("\n"),
    extensions: [
      EditorState.readOnly.of(true),
      EditorView.editable.of(false),
      EditorView.lineWrapping,
      propertiesLanguage,
      propertiesSyntaxHighlighting,
      createLineDecorations(lines),
      createLineNumberGutter(lines, "old"),
      createLineNumberGutter(lines, "new"),
      EditorView.theme({
        "&": {
          border: "1px solid var(--sl-border-light)",
          borderRadius: "var(--sl-radius-md)",
          backgroundColor: "var(--sl-surface)",
          overflow: "hidden",
        },
        ".cm-scroller": {
          fontFamily: "var(--sl-font-mono)",
          fontSize: "var(--sl-font-size-xl)",
          lineHeight: "1.55",
          padding: "0",
          maxHeight: "min(56vh, 560px)",
          overflowY: "auto",
        },
        ".cm-content": {
          padding: "0",
        },
        ".cm-gutters": {
          backgroundColor: "var(--sl-bg-secondary)",
          color: "var(--sl-text-tertiary)",
          borderRight: "1px solid var(--sl-border-light)",
        },
        ".cm-diff-gutter": {
          minWidth: "56px",
        },
        ".cm-diff-gutter .cm-gutterElement": {
          padding: "0 10px 0 12px",
          textAlign: "right",
        },
        ".cm-diff-gutter--old": {
          borderRight: "1px solid var(--sl-border-light)",
        },
        ".cm-diff-gutter--new": {
          borderRight: "1px solid var(--sl-border-light)",
        },
        ".cm-diff-line": {
          borderTop: "1px solid rgba(148, 163, 184, 0.08)",
        },
        ".cm-line": {
          padding: "0 12px",
        },
        ".cm-diff-line--addition": {
          backgroundColor: "rgba(34, 197, 94, 0.12)",
        },
        ".cm-diff-gutter-marker": {
          display: "flex",
          alignItems: "center",
          justifyContent: "flex-end",
          width: "calc(100% + 22px)",
          minHeight: "100%",
          boxSizing: "border-box",
          margin: "0 -10px 0 -12px",
          padding: "0 10px 0 12px",
        },
        ".cm-diff-gutter-marker--addition": {
          backgroundColor: "rgba(34, 197, 94, 0.12)",
        },
        ".cm-diff-line--deletion": {
          backgroundColor: "rgba(239, 68, 68, 0.12)",
        },
        ".cm-diff-gutter-marker--deletion": {
          backgroundColor: "rgba(239, 68, 68, 0.12)",
        },
        ".cm-diff-line--omitted": {
          backgroundColor: "color-mix(in srgb, var(--sl-bg-secondary) 82%, transparent)",
          color: "var(--sl-text-tertiary)",
        },
        ".cm-diff-gutter-marker--omitted": {
          backgroundColor: "color-mix(in srgb, var(--sl-bg-secondary) 82%, transparent)",
        },
        ".cm-activeLine": {
          backgroundColor: "transparent",
        },
        "&.cm-focused": {
          outline: "none",
        },
      }),
    ],
  });

  editorView = new EditorView({ state, parent: editorRoot.value });
}

onMounted(() => {
  createEditor();
});

watch(
  () => [props.original, props.modified],
  async () => {
    await nextTick();
    createEditor();
  },
);

onBeforeUnmount(() => {
  destroyEditor();
});
</script>

<template>
  <div ref="editorRoot" class="source-diff-cm-root"></div>
</template>

<style scoped>
.source-diff-cm-root {
  min-height: 0;
}
</style>
