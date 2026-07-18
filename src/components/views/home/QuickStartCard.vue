<script setup lang="ts">
import { i18n } from "@language";
import { currentQuote, displayText, isTyping, updateQuote } from "@utils/quoteUtils";

const emit = defineEmits<{
  (e: "create"): void;
}>();
</script>

<template>
  <cmz-card
    variant="glass"
    :title="i18n.t('home.title')"
    :subtitle="i18n.t('home.create_first')"
    class="quick-start-card"
  >
    <div class="quick-actions">
      <cmz-button variant="solid" size="lg" @click="emit('create')">
        {{ i18n.t("common.create_server") }}
      </cmz-button>
    </div>
    <div class="quote-display" @click="updateQuote" :title="i18n.t('common.click_to_refresh')">
      <span v-if="displayText || isTyping" class="quote-text">「{{ displayText }}」</span>
      <span v-else class="quote-loading">{{ i18n.t("common.loading") }}</span>
      <span
        class="quote-author"
        :class="{ 'quote-author-hidden': !displayText || isTyping || !currentQuote.author }"
        >—— {{ currentQuote.author || " " }}</span
      >
    </div>
  </cmz-card>
</template>

<style scoped>
.quick-start-card {
  display: flex;
  flex-direction: column;
}

.quick-start-card :deep(.cmz-card-body) {
  display: flex;
  flex-direction: column;
}

.quick-actions {
  display: flex;
  gap: var(--sl-space-sm);
  margin-top: var(--sl-space-sm);
  flex-wrap: wrap;
}

.quote-display {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: var(--sl-space-xs) var(--sl-space-sm);
  margin-top: var(--sl-space-md);
  border-top: 1px solid var(--sl-border-light);
  cursor: pointer;
  transition: all 0.3s ease;
  border-radius: var(--sl-radius-sm);
  position: relative;
  overflow: hidden;
}

.quote-display:hover {
  opacity: 0.9;
  background: var(--sl-bg-secondary);
  transform: translateY(-1px);
  box-shadow: var(--sl-shadow-sm);
}

.quote-text {
  font-size: 0.875rem;
  color: var(--sl-text-secondary);
  font-style: italic;
  text-align: center;
  transition: all 0.3s ease;
  opacity: 1;
}

.quote-text.fading {
  opacity: 0;
  transform: translateY(5px);
}

.quote-author {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  transition: all 0.3s ease;
  opacity: 1;
}

.quote-author-hidden {
  visibility: hidden;
}

.quote-author.fading {
  opacity: 0;
  transform: translateY(5px);
}

.quote-loading {
  font-size: 0.875rem;
  color: var(--sl-text-tertiary);
  font-style: italic;
  animation: quoteLoading 1.5s ease-in-out infinite;
}

@keyframes quoteLoading {
  0%,
  100% {
    opacity: 0.6;
  }
  50% {
    opacity: 1;
  }
}
</style>
