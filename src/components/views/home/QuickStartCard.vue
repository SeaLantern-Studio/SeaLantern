<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";
import { currentQuote, displayText, isTyping, updateQuote } from "@utils/quoteUtils";

const emit = defineEmits<{
  (e: "create"): void;
}>();
</script>

<template>
  <SLCard
    :title="i18n.t('home.title')"
    :subtitle="i18n.t('home.create_first')"
    variant="solid"
    class="quick-start-card"
  >
    <div class="quick-actions">
      <SLButton variant="primary" size="lg" @click="emit('create')">
        {{ i18n.t("common.create_server") }}
      </SLButton>
    </div>
    <div class="card-spacer"></div>
    <div class="quote-display" @click="updateQuote" :title="i18n.t('common.click_to_refresh')">
      <span v-if="displayText && !isTyping" class="quote-text">「{{ displayText }}」</span>
      <span v-if="currentQuote && !isTyping" class="quote-author"
        >—— {{ currentQuote.author }}</span
      >
      <span v-if="isTyping" class="quote-text">「{{ displayText }}」</span>
      <span v-if="!displayText && !isTyping" class="quote-loading">{{
        i18n.t("common.loading")
      }}</span>
    </div>
  </SLCard>
</template>

<style scoped>
.quick-start-card {
  display: flex;
  flex-direction: column;
  height: 280px;
}

.quick-actions {
  display: flex;
  gap: var(--sl-space-sm);
  margin-top: var(--sl-space-sm);
  flex-wrap: wrap;
}

.card-spacer {
  flex-grow: 1;
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
  0%, 100% {
    opacity: 0.6;
  }
  50% {
    opacity: 1;
  }
}
</style>
