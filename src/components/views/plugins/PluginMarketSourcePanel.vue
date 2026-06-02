<script setup lang="ts">
import { i18n } from "@language";
import type { MarketFeedback } from "./usePluginMarket";

defineProps<{
  installFeedback: MarketFeedback | null;
  showUrlEditor: boolean;
  customMarketUrl: string;
  urlInput: string;
  isUsingCustomMarket: boolean;
  activeMarketHost: string;
  pendingMarketSource: string | null;
  showCustomSourceConfirm: boolean;
}>();

const emit = defineEmits<{
  (e: "clear-feedback"): void;
  (e: "update:url-input", value: string): void;
  (e: "save-market-url"): void;
  (e: "reset-market-url"): void;
  (e: "confirm-custom-source"): void;
  (e: "cancel-custom-source"): void;
}>();

const MARKET_BASE_URL = "https://sealantern-studio.github.io/plugin-market";
</script>

<template>
  <div
    v-if="installFeedback"
    :class="['install-feedback', `install-feedback--${installFeedback.type}`]"
  >
    <span>{{ installFeedback.message }}</span>
    <button class="install-feedback-close" @click="emit('clear-feedback')">x</button>
  </div>

  <div v-if="showUrlEditor" class="url-editor glass">
    <span class="url-editor-label">{{ i18n.t("market.source_url") }}</span>
    <input
      :value="urlInput"
      type="url"
      class="url-editor-input"
      :placeholder="MARKET_BASE_URL"
      @input="emit('update:url-input', ($event.target as HTMLInputElement).value)"
      @keydown.enter="emit('save-market-url')"
    />
    <button class="url-editor-btn" @click="emit('save-market-url')">
      {{ i18n.t("market.source_save") }}
    </button>
    <button
      v-if="customMarketUrl"
      class="url-editor-btn url-editor-btn--reset"
      @click="emit('reset-market-url')"
    >
      {{ i18n.t("market.source_reset") }}
    </button>
    <p v-if="isUsingCustomMarket" class="url-editor-note">
      {{ i18n.t("market.source_current", { host: activeMarketHost }) }}
    </p>
  </div>

  <div v-if="showCustomSourceConfirm && pendingMarketSource" class="source-confirm glass">
    <p class="source-confirm-title">{{ i18n.t("market.source_confirm_title") }}</p>
    <p class="source-confirm-text">{{ i18n.t("market.source_confirm_desc") }}</p>
    <p class="source-confirm-url">{{ pendingMarketSource }}</p>
    <div class="source-confirm-actions">
      <button class="url-editor-btn" @click="emit('confirm-custom-source')">
        {{ i18n.t("market.source_save") }}
      </button>
      <button class="url-editor-btn url-editor-btn--reset" @click="emit('cancel-custom-source')">
        {{ i18n.t("common.cancel") }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.install-feedback {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 14px;
  border-radius: var(--sl-radius-md);
  border: 1px solid transparent;
  white-space: pre-wrap;
  line-height: 1.45;
  font-size: 13px;
}

.install-feedback--success {
  color: #2e7d32;
  background: rgba(46, 125, 50, 0.12);
  border-color: rgba(46, 125, 50, 0.28);
}

.install-feedback--warning {
  color: #8a6d00;
  background: rgba(245, 158, 11, 0.14);
  border-color: rgba(245, 158, 11, 0.3);
}

.install-feedback--error {
  color: var(--sl-error);
  background: rgba(239, 68, 68, 0.12);
  border-color: rgba(239, 68, 68, 0.25);
}

.install-feedback-close {
  border: none;
  background: transparent;
  color: inherit;
  font-weight: 700;
  line-height: 1;
  cursor: pointer;
}

.url-editor,
.source-confirm {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  background: var(--sl-bg-secondary);
}

.url-editor-label,
.source-confirm-title {
  color: var(--sl-text-primary);
  font-size: 13px;
  font-weight: 600;
}

.url-editor-input {
  flex: 1 1 320px;
  min-width: 220px;
  padding: 8px 10px;
  border-radius: var(--sl-radius-sm);
  border: 1px solid var(--sl-border);
  background: var(--sl-surface);
  color: var(--sl-text-primary);
}

.url-editor-note,
.source-confirm-text {
  flex-basis: 100%;
  margin: 0;
  color: var(--sl-text-secondary);
  font-size: 12px;
  line-height: 1.5;
}

.source-confirm-url {
  flex-basis: 100%;
  margin: 0;
  color: var(--sl-text-primary);
  font-size: 12px;
  word-break: break-all;
}

.source-confirm-actions {
  display: flex;
  gap: 8px;
}

.url-editor-btn {
  padding: 8px 12px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  background: var(--sl-surface);
  color: var(--sl-text-primary);
  cursor: pointer;
}

.url-editor-btn--reset {
  color: var(--sl-text-secondary);
}
</style>
