<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { Bold, Copy, Eraser, Italic, Sparkles, Strikethrough, Underline } from "lucide-vue-next";
import { i18n } from "@language";
import unknownServerIcon from "@assets/motd/unknown_server.jpg";
import {
  MOTD_COLOR_CODES,
  MOTD_COLOR_MAP,
  MOTD_RAINBOW_COLOR_CODES,
  colorCodeFromCss,
  htmlToMotd,
  motdToExportText,
  motdToHtml,
  normalizeMotdText,
  unicodeEscape,
  type MotdFormatState,
} from "@utils/motdCodes";
import { MOTD_TEMPLATES } from "@data/motdTemplates";
import "@styles/components/MotdVisualEditor.css";

interface Props {
  /** 当前 server.properties 中的 motd 草稿值（换行为字面 \n 的存储格式） */
  modelValue: string;
  /** 服务器列表中的名称行（展示实际服务器名） */
  serverName?: string;
  /** 是否为独立工具页内联模式（不包裹弹窗，且渲染底部应用按钮） */
  embedded?: boolean;
  /** 底部应用按钮文案（embedded 模式生效） */
  applyText?: string;
  /** 是否禁用底部应用按钮（如正在写入服务器） */
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  serverName: "Minecraft Server",
  embedded: false,
  applyText: "",
  disabled: false,
});

const emit = defineEmits<{
  apply: [value: string];
}>();

const editorRef = ref<HTMLDivElement | null>(null);
const html = ref("");
const escapeUnicodeOut = ref(false);
const importText = ref("");
const statusMsg = ref<string | null>(null);
const fmt = ref<MotdFormatState>({
  bold: false,
  italic: false,
  underline: false,
  strike: false,
  colorCode: "f",
});

let statusTimer: ReturnType<typeof setTimeout> | null = null;
let selectionAbort: AbortController | null = null;

const templatePreviews = computed(() =>
  MOTD_TEMPLATES.map((template) => ({
    ...template,
    name: `${i18n.t(`config.motd.styles.${template.name}`)} ${template.index + 1}`,
    previewHtml: motdToHtml(normalizeMotdText(template.value)),
  })),
);

const motd = computed(() => {
  const raw = htmlToMotd(html.value);
  return escapeUnicodeOut.value ? unicodeEscape(raw) : raw;
});

const exportMotd = computed(() => motdToExportText(motd.value));

function setStatus(message: string) {
  statusMsg.value = message;
  if (statusTimer) {
    clearTimeout(statusTimer);
  }
  statusTimer = setTimeout(() => {
    statusMsg.value = null;
  }, 2500);
}

function syncToolbarState() {
  const editor = editorRef.value;
  const sel = window.getSelection();
  // 仅在选区位于编辑器内时同步工具栏状态，避免全局 selectionchange 的副作用
  if (!editor || !sel || !sel.anchorNode || !editor.contains(sel.anchorNode)) {
    return;
  }
  // 从选区锚点的计算样式推导工具栏状态，避免依赖已弃用的 queryCommandState/Value
  const node = sel.anchorNode;
  const el = node.nodeType === Node.TEXT_NODE ? node.parentElement : (node as Element | null);
  if (!el) return;
  const cs = window.getComputedStyle(el);
  fmt.value = {
    bold: Number(cs.fontWeight) >= 600 || cs.fontWeight === "bold",
    italic: cs.fontStyle === "italic",
    underline: cs.textDecorationLine.includes("underline"),
    strike: cs.textDecorationLine.includes("line-through"),
    colorCode: colorCodeFromCss(cs.color),
  };
}

function setEditorHtml(nextHtml: string) {
  html.value = nextHtml;
  if (editorRef.value) {
    editorRef.value.innerHTML = nextHtml;
  }
  syncToolbarState();
}

/**
 * 通过 execCommand 施加编辑区内联格式。
 * execCommand 已弃用且在部分浏览器中行为不完全一致，统一在此封装：做存在性检测并吞掉异常，
 * 避免在不支持的环境直接抛错。后续若整体迁移到基于 Selection/Range 的 DOM 操作，调用方无需改动。
 */
function execFormatCommand(command: string, value?: string): boolean {
  if (typeof document.execCommand !== "function") return false;
  try {
    return document.execCommand(command, false, value);
  } catch {
    return false;
  }
}

function apply(cmd: string, val?: string) {
  editorRef.value?.focus();
  execFormatCommand("styleWithCSS", "true");
  execFormatCommand(cmd, val);
  setEditorHtml(editorRef.value?.innerHTML ?? "");
}

function applyRainbowToSelection() {
  const editor = editorRef.value;
  if (!editor) return;

  editor.focus();
  const selection = window.getSelection();
  if (!selection || selection.rangeCount === 0 || selection.isCollapsed) {
    setStatus(i18n.t("config.motd.select_text_first"));
    return;
  }

  const range = selection.getRangeAt(0);
  const withinEditor = editor.contains(range.startContainer) && editor.contains(range.endContainer);
  if (!withinEditor) {
    setStatus(i18n.t("config.motd.select_in_editor"));
    return;
  }

  const extracted = range.extractContents();
  const wrapper = document.createElement("span");
  wrapper.appendChild(extracted);

  const walker = document.createTreeWalker(wrapper, NodeFilter.SHOW_TEXT);
  const textNodes: Text[] = [];
  while (walker.nextNode()) {
    textNodes.push(walker.currentNode as Text);
  }

  let colorIndex = 0;
  for (const textNode of textNodes) {
    const text = textNode.nodeValue ?? "";
    if (!text) continue;
    const colored = document.createDocumentFragment();
    for (const ch of text) {
      if (ch === "\n" || ch === "\r") {
        colored.appendChild(document.createTextNode(ch));
        continue;
      }
      const span = document.createElement("span");
      const code = MOTD_RAINBOW_COLOR_CODES[colorIndex % MOTD_RAINBOW_COLOR_CODES.length];
      span.style.color = MOTD_COLOR_MAP[code];
      span.textContent = ch;
      colored.appendChild(span);
      colorIndex += 1;
    }
    textNode.parentNode?.replaceChild(colored, textNode);
  }

  range.insertNode(wrapper);
  while (wrapper.firstChild) {
    wrapper.parentNode?.insertBefore(wrapper.firstChild, wrapper);
  }
  wrapper.remove();

  selection.removeAllRanges();
  setEditorHtml(editor.innerHTML);
}

function onEditorInput() {
  html.value = editorRef.value?.innerHTML ?? "";
}

function onEditorKeyDown(event: KeyboardEvent) {
  if (event.key === "Enter") {
    const text = editorRef.value?.innerText ?? "";
    const linesCount = text
      .replace(/\r/g, "")
      .split("\n")
      .filter((line) => line.length > 0).length;
    // MOTD 在服务器列表中最多展示两行
    if (linesCount >= 2) {
      event.preventDefault();
    }
  }
}

function applyTemplate(rawTemplate: string | string[]) {
  setEditorHtml(motdToHtml(normalizeMotdText(rawTemplate)));
  editorRef.value?.focus();
}

function importForEdit() {
  const raw = importText.value.trim();
  if (!raw) {
    setStatus(i18n.t("config.motd.import_empty"));
    return;
  }
  setEditorHtml(motdToHtml(normalizeMotdText(raw)));
  editorRef.value?.focus();
  setStatus(i18n.t("config.motd.imported"));
}

async function copyMotd() {
  try {
    await navigator.clipboard.writeText(exportMotd.value);
    setStatus(i18n.t("config.motd.copied"));
  } catch {
    setStatus(i18n.t("config.motd.copy_failed"));
  }
}

function handleApply() {
  emit("apply", exportMotd.value);
}

onMounted(() => {
  setEditorHtml(motdToHtml(normalizeMotdText(props.modelValue || "")));
  selectionAbort = new AbortController();
  document.addEventListener("selectionchange", syncToolbarState, {
    signal: selectionAbort.signal,
  });
});

onBeforeUnmount(() => {
  selectionAbort?.abort();
  selectionAbort = null;
  if (statusTimer) {
    clearTimeout(statusTimer);
  }
});

defineExpose({
  /** 供弹窗/父组件触发应用，等同点击底部应用按钮 */
  requestApply: handleApply,
});
</script>

<template>
  <div class="motd-editor">
    <div class="motd-editor__surface">
      <div class="motd-editor__inner motd-editor-dirt-bg">
        <img :src="unknownServerIcon" alt="server icon" class="motd-editor__icon" />
        <div class="motd-editor__text">
          <div class="motd-editor__name-line">{{ serverName }}</div>
          <div
            ref="editorRef"
            class="motd-editor__output"
            contenteditable="true"
            spellcheck="false"
            @input="onEditorInput"
            @keydown="onEditorKeyDown"
          />
        </div>
      </div>
    </div>

    <div class="motd-editor__toolbar">
      <div class="motd-editor__toolbar-group">
        <button
          v-for="code in MOTD_COLOR_CODES"
          :key="code"
          type="button"
          class="motd-editor__tool-btn"
          :class="{ 'is-active': fmt.colorCode === code }"
          :style="{ color: MOTD_COLOR_MAP[code] }"
          :title="code"
          @mousedown.prevent
          @click="apply('foreColor', MOTD_COLOR_MAP[code])"
        >
          {{ code }}
        </button>
      </div>
      <div class="motd-editor__toolbar-group">
        <button
          type="button"
          class="motd-editor__tool-btn"
          :class="{ 'is-active': fmt.bold }"
          :title="i18n.t('config.motd.bold')"
          @mousedown.prevent
          @click="apply('bold')"
        >
          <Bold :size="13" />
        </button>
        <button
          type="button"
          class="motd-editor__tool-btn"
          :class="{ 'is-active': fmt.italic }"
          :title="i18n.t('config.motd.italic')"
          @mousedown.prevent
          @click="apply('italic')"
        >
          <Italic :size="13" />
        </button>
        <button
          type="button"
          class="motd-editor__tool-btn"
          :class="{ 'is-active': fmt.underline }"
          :title="i18n.t('config.motd.underline')"
          @mousedown.prevent
          @click="apply('underline')"
        >
          <Underline :size="13" />
        </button>
        <button
          type="button"
          class="motd-editor__tool-btn"
          :class="{ 'is-active': fmt.strike }"
          :title="i18n.t('config.motd.strikethrough')"
          @mousedown.prevent
          @click="apply('strikeThrough')"
        >
          <Strikethrough :size="13" />
        </button>
        <button
          type="button"
          class="motd-editor__tool-btn"
          :title="i18n.t('config.motd.rainbow')"
          @mousedown.prevent
          @click="applyRainbowToSelection"
        >
          <Sparkles :size="13" />
        </button>
        <button
          type="button"
          class="motd-editor__tool-btn"
          :title="i18n.t('config.motd.clear_format')"
          @mousedown.prevent
          @click="apply('removeFormat')"
        >
          <Eraser :size="13" />
        </button>
      </div>
    </div>

    <div class="motd-editor__panel">
      <div class="motd-editor__panel-header">
        <h3 class="motd-editor__panel-title">{{ i18n.t("config.motd.export_title") }}</h3>
        <div class="motd-editor__panel-tools">
          <label class="motd-editor__escape-row">
            <span>{{ i18n.t("config.motd.escape_unicode") }}</span>
            <cmz-switch
              :modelValue="escapeUnicodeOut"
              @update:modelValue="escapeUnicodeOut = $event"
            />
          </label>
          <div class="motd-editor__import-box">
            <input
              v-model="importText"
              class="motd-editor__import-input"
              :placeholder="i18n.t('config.motd.import_placeholder')"
            />
            <cmz-button size="sm" variant="outline" @click="importForEdit">
              {{ i18n.t("config.motd.import_button") }}
            </cmz-button>
          </div>
        </div>
      </div>
      <pre class="motd-editor__export-pre">{{ exportMotd }}</pre>
      <div class="motd-editor__panel-tools" style="margin-top: var(--sl-space-sm)">
        <cmz-button size="sm" variant="outline" @click="copyMotd">
          <Copy :size="13" />
          {{ i18n.t("config.motd.copy") }}
        </cmz-button>
      </div>
      <p class="motd-editor__status">{{ statusMsg }}</p>
    </div>

    <div class="motd-editor__templates">
      <h3 class="motd-editor__panel-title">{{ i18n.t("config.motd.templates_title") }}</h3>
      <div class="motd-editor__templates-list">
        <button
          v-for="template in templatePreviews"
          :key="template.name"
          type="button"
          class="motd-editor__template-btn"
          @click="applyTemplate(template.value)"
        >
          <div class="motd-editor__template-header">
            <span class="motd-editor__template-title">{{ template.name }}</span>
          </div>
          <div class="motd-editor__template-preview motd-editor-dirt-bg">
            <img :src="unknownServerIcon" alt="template icon" class="motd-editor__template-icon" />
            <div class="motd-editor__template-text">
              <div class="motd-editor__template-name-line">Minecraft Server</div>
              <!-- 预览 HTML 由 motdToHtml 生成，文本部分已做 HTML 转义 -->
              <div class="motd-editor__template-output" v-html="template.previewHtml"></div>
            </div>
          </div>
        </button>
      </div>
    </div>

    <div v-if="embedded" class="motd-editor__footer">
      <cmz-button :disabled="disabled" @click="handleApply">
        {{ applyText || i18n.t("config.motd.apply") }}
      </cmz-button>
    </div>
  </div>
</template>
