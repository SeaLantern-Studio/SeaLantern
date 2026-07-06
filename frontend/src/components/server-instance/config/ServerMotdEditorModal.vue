<script setup lang="ts">
import { computed, shallowRef, watch, useTemplateRef } from "vue";
import { SLButton, SLInput, SLModal } from "@components/common";
import { i18n } from "@language";
import {
  RESET_CODE,
  SECTION_SIGN,
  detectMotdEdition,
  encodeMotdForSource,
  getEditionColorOptions,
  getEditionFormatOptions,
  getEditionNoteKey,
  parseMotdPreview,
  type MotdEdition,
  type MotdColorOption,
} from "@src/features/config-editor/motdFormat";

interface Props {
  visible: boolean;
  value: string;
  saving: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  close: [];
  save: [value: string];
  "update:value": [value: string];
}>();

const editorRef = useTemplateRef<HTMLTextAreaElement>("editorRef");

const normalizedValue = computed({
  get: () => props.value,
  set: (value: string) => emit("update:value", value),
});

const encodedValue = computed(() => encodeMotdForSource(props.value));
const edition = shallowRef<MotdEdition>(detectMotdEdition(props.value));
const colorOptions = computed(() => getEditionColorOptions(edition.value));
const formatOptions = computed(() => getEditionFormatOptions(edition.value));
const previewLines = computed(() => parseMotdPreview(props.value, edition.value));
const editionNote = computed(() => i18n.t(getEditionNoteKey(edition.value)));

watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      edition.value = detectMotdEdition(props.value);
    }
  },
);

function closeModal(): void {
  emit("close");
}

function updateValue(nextValue: string): void {
  normalizedValue.value = nextValue;
}

function insertText(text: string): void {
  const textarea = editorRef.value;
  if (!textarea) {
    updateValue(`${props.value}${text}`);
    return;
  }

  const start = textarea.selectionStart ?? props.value.length;
  const end = textarea.selectionEnd ?? start;
  const nextValue = `${props.value.slice(0, start)}${text}${props.value.slice(end)}`;
  updateValue(nextValue);

  requestAnimationFrame(() => {
    textarea.focus();
    const cursor = start + text.length;
    textarea.setSelectionRange(cursor, cursor);
  });
}

function wrapSelection(code: string): void {
  const textarea = editorRef.value;
  if (!textarea) {
    insertText(code);
    return;
  }

  const start = textarea.selectionStart ?? props.value.length;
  const end = textarea.selectionEnd ?? start;
  const selected = props.value.slice(start, end);
  const suffix = selected.length > 0 ? RESET_CODE : "";
  const inserted = `${code}${selected}${suffix}`;
  const nextValue = `${props.value.slice(0, start)}${inserted}${props.value.slice(end)}`;
  updateValue(nextValue);

  requestAnimationFrame(() => {
    textarea.focus();
    if (selected.length > 0) {
      textarea.setSelectionRange(start + code.length, start + code.length + selected.length);
      return;
    }
    const cursor = start + code.length;
    textarea.setSelectionRange(cursor, cursor);
  });
}

function insertColor(option: MotdColorOption): void {
  wrapSelection(option.code);
}

function insertFormat(code: string): void {
  wrapSelection(code);
}

function insertNewLineMarker(): void {
  insertText("\n");
}

function resetFormatting(): void {
  insertText(RESET_CODE);
}

function setEdition(value: MotdEdition): void {
  edition.value = value;
}

function handleSave(): void {
  emit("save", props.value);
}

function replaceAmpersandCodes(): void {
  updateValue(props.value.replaceAll("&", SECTION_SIGN));
}

function getColorChipStyle(option: MotdColorOption) {
  return {
    backgroundColor: option.color,
    color: option.key === "0" ? "#ffffff" : "#111111",
  };
}
</script>

<template>
  <SLModal
    :visible="visible"
    :title="i18n.t('config.next_v1.motd.title')"
    width="980px"
    :close-on-overlay="!saving"
    @close="closeModal"
  >
    <div class="motd-editor-modal">
      <div class="motd-editor-modal__main">
        <div class="motd-editor-modal__toolbar-block">
          <div class="motd-editor-modal__toolbar-header">
            <strong>{{ i18n.t("config.next_v1.motd.palette_title") }}</strong>
            <span>{{ i18n.t("config.next_v1.motd.palette_note") }}</span>
            <span>{{ editionNote }}</span>
          </div>

          <div class="motd-editor-modal__mode-switch">
            <span class="motd-editor-modal__toolbar-label">{{
              i18n.t("config.next_v1.motd.mode_label")
            }}</span>
            <div class="motd-editor-modal__mode-actions">
              <SLButton
                :variant="edition === 'java' ? 'primary' : 'secondary'"
                size="sm"
                @click="setEdition('java')"
              >
                {{ i18n.t("config.next_v1.motd.mode_java") }}
              </SLButton>
              <SLButton
                :variant="edition === 'bedrock' ? 'primary' : 'secondary'"
                size="sm"
                @click="setEdition('bedrock')"
              >
                {{ i18n.t("config.next_v1.motd.mode_bedrock") }}
              </SLButton>
            </div>
          </div>

          <div class="motd-editor-modal__toolbar-group">
            <span class="motd-editor-modal__toolbar-label">{{
              i18n.t("config.next_v1.motd.color_group")
            }}</span>
            <div class="motd-editor-modal__toolbar-grid motd-editor-modal__toolbar-grid--colors">
              <button
                v-for="option in colorOptions"
                :key="option.key"
                type="button"
                class="motd-editor-modal__color-button"
                :title="`${i18n.t(option.labelKey)} (${option.code})`"
                :style="getColorChipStyle(option)"
                @click="insertColor(option)"
              >
                {{ option.code }}
              </button>
            </div>
          </div>

          <div class="motd-editor-modal__toolbar-group">
            <span class="motd-editor-modal__toolbar-label">{{
              i18n.t("config.next_v1.motd.format_group")
            }}</span>
            <div class="motd-editor-modal__toolbar-grid">
              <SLButton
                v-for="option in formatOptions"
                :key="option.key"
                variant="secondary"
                size="sm"
                @click="insertFormat(option.code)"
              >
                {{ i18n.t(option.labelKey) }}
              </SLButton>
              <SLButton variant="secondary" size="sm" @click="insertNewLineMarker">
                {{ i18n.t("config.next_v1.motd.insert_new_line") }}
              </SLButton>
              <SLButton variant="ghost" size="sm" @click="resetFormatting">
                {{ i18n.t("config.next_v1.motd.reset") }}
              </SLButton>
            </div>
          </div>
        </div>

        <div class="motd-editor-modal__editor-block">
          <div class="motd-editor-modal__field-header">
            <strong>{{ i18n.t("config.next_v1.motd.editor_label") }}</strong>
            <span>{{ i18n.t("config.next_v1.motd.editor_hint") }}</span>
          </div>

          <div class="motd-editor-modal__helper-actions">
            <SLButton variant="ghost" size="sm" @click="replaceAmpersandCodes">
              {{ i18n.t("config.next_v1.motd.convert_ampersand") }}
            </SLButton>
          </div>

          <textarea
            ref="editorRef"
            class="motd-editor-modal__textarea"
            :value="normalizedValue"
            :placeholder="i18n.t('config.next_v1.motd.editor_placeholder')"
            rows="7"
            @input="updateValue(($event.target as HTMLTextAreaElement).value)"
          />

          <SLInput
            :model-value="encodedValue"
            :label="i18n.t('config.next_v1.motd.encoded_label')"
            disabled
          />
        </div>
      </div>

      <div class="motd-editor-modal__preview-block">
        <div class="motd-editor-modal__field-header">
          <strong>{{ i18n.t("config.next_v1.motd.preview_title") }}</strong>
          <span>{{ i18n.t("config.next_v1.motd.preview_hint") }}</span>
        </div>

        <div class="motd-editor-modal__preview-shell">
          <div class="motd-editor-modal__preview-header">
            <span>{{ i18n.t("config.next_v1.motd.preview_server_name") }}</span>
            <span>25565</span>
          </div>
          <div class="motd-editor-modal__preview-body">
            <div
              v-for="(line, lineIndex) in previewLines"
              :key="lineIndex"
              class="motd-editor-modal__preview-line"
            >
              <template v-if="line.length > 0">
                <span
                  v-for="(token, tokenIndex) in line"
                  :key="`${lineIndex}-${tokenIndex}`"
                  class="motd-editor-modal__preview-token"
                  :style="{
                    color: token.color,
                    fontWeight: token.bold ? '700' : '400',
                    fontStyle: token.italic ? 'italic' : 'normal',
                    textDecoration:
                      `${token.underline ? 'underline ' : ''}${token.strikethrough ? 'line-through' : ''}`.trim() ||
                      'none',
                  }"
                >
                  {{ token.text }}
                </span>
              </template>
              <span v-else class="motd-editor-modal__preview-empty-line">&nbsp;</span>
            </div>
          </div>
        </div>

        <div class="motd-editor-modal__wiki-card">
          <strong>{{ i18n.t("config.next_v1.motd.wiki_title") }}</strong>
          <p>{{ i18n.t("config.next_v1.motd.wiki_note") }}</p>
          <div class="motd-editor-modal__wiki-list">
            <span
              v-for="option in colorOptions"
              :key="`wiki-color-${option.key}`"
              class="motd-editor-modal__wiki-chip"
            >
              {{ option.code }} {{ i18n.t(option.labelKey) }}
            </span>
            <span
              v-for="option in formatOptions"
              :key="`wiki-format-${option.key}`"
              class="motd-editor-modal__wiki-chip"
            >
              {{ option.code }} {{ i18n.t(option.labelKey) }}
            </span>
            <span class="motd-editor-modal__wiki-chip"
              >{{ RESET_CODE }} {{ i18n.t("config.next_v1.motd.reset") }}</span
            >
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="motd-editor-modal__footer">
        <SLButton variant="secondary" :disabled="saving" @click="closeModal">
          {{ i18n.t("config.next_v1.discard_cancel") }}
        </SLButton>
        <SLButton variant="primary" :loading="saving" @click="handleSave">
          {{ i18n.t("config.save") }}
        </SLButton>
      </div>
    </template>
  </SLModal>
</template>

<style scoped>
.motd-editor-modal,
.motd-editor-modal__main,
.motd-editor-modal__toolbar-block,
.motd-editor-modal__editor-block,
.motd-editor-modal__preview-block,
.motd-editor-modal__preview-body,
.motd-editor-modal__wiki-card {
  display: grid;
  gap: 12px;
}

.motd-editor-modal {
  grid-template-columns: minmax(0, 1.15fr) minmax(320px, 0.85fr);
  align-items: start;
}

.motd-editor-modal__main {
  gap: 16px;
}

.motd-editor-modal__toolbar-block,
.motd-editor-modal__editor-block,
.motd-editor-modal__preview-shell,
.motd-editor-modal__wiki-card {
  padding: 14px;
  border: 1px solid var(--sl-border);
  border-radius: 16px;
  background: color-mix(in srgb, var(--sl-surface) 92%, transparent);
}

.motd-editor-modal__toolbar-header,
.motd-editor-modal__field-header {
  display: grid;
  gap: 4px;
}

.motd-editor-modal__toolbar-header span,
.motd-editor-modal__field-header span,
.motd-editor-modal__wiki-card p,
.motd-editor-modal__helper-actions {
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-sm);
}

.motd-editor-modal__toolbar-group {
  display: grid;
  gap: 8px;
}

.motd-editor-modal__mode-switch,
.motd-editor-modal__mode-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.motd-editor-modal__toolbar-label {
  font-size: var(--sl-font-size-sm);
  color: var(--sl-text-secondary);
}

.motd-editor-modal__toolbar-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.motd-editor-modal__toolbar-grid--colors {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.motd-editor-modal__color-button {
  min-height: 38px;
  border: 1px solid rgba(255, 255, 255, 0.16);
  border-radius: 10px;
  font: inherit;
  font-weight: 700;
  cursor: pointer;
  transition: transform 0.15s ease;
}

.motd-editor-modal__color-button:hover {
  transform: translateY(-1px);
}

.motd-editor-modal__helper-actions {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.motd-editor-modal__textarea {
  width: 100%;
  min-height: 160px;
  padding: 12px 14px;
  border: 1px solid var(--sl-border);
  border-radius: 12px;
  background: var(--sl-surface);
  color: var(--sl-text-primary);
  font: inherit;
  line-height: 1.5;
  resize: vertical;
}

.motd-editor-modal__textarea:focus {
  outline: none;
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 3px var(--sl-primary-bg);
}

.motd-editor-modal__preview-shell {
  gap: 0;
  padding: 0;
  overflow: hidden;
  background: #10141c;
  border-color: rgba(255, 255, 255, 0.08);
}

.motd-editor-modal__preview-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  font-size: var(--sl-font-size-sm);
  color: rgba(255, 255, 255, 0.72);
  background: rgba(255, 255, 255, 0.04);
}

.motd-editor-modal__preview-body {
  min-height: 130px;
  padding: 14px;
  gap: 6px;
  font-size: 15px;
  line-height: 1.35;
}

.motd-editor-modal__preview-line {
  min-height: 20px;
  word-break: break-word;
}

.motd-editor-modal__preview-token {
  text-shadow: 1px 1px 0 rgba(0, 0, 0, 0.6);
}

.motd-editor-modal__preview-empty-line {
  opacity: 0.3;
}

.motd-editor-modal__wiki-card p {
  margin: 0;
}

.motd-editor-modal__wiki-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.motd-editor-modal__wiki-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.06);
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-sm);
}

.motd-editor-modal__footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

@media (max-width: 980px) {
  .motd-editor-modal {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 640px) {
  .motd-editor-modal__toolbar-grid--colors {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
