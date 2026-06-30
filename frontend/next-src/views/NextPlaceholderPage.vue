<script setup lang="ts">
import type { RouteLocationRaw } from "vue-router";
import { ArrowRight, Construction, LayoutTemplate, Waypoints } from "@lucide/vue";
import { RouterLink } from "vue-router";

interface PlaceholderTrack {
  title: string;
  description: string;
}

interface PlaceholderAction {
  label: string;
  to: RouteLocationRaw;
  hint: string;
  tone?: "primary" | "secondary";
}

interface Props {
  eyebrow: string;
  summary: string;
  description: string;
  tracks: PlaceholderTrack[];
  actions: PlaceholderAction[];
}

defineProps<Props>();
</script>

<template>
  <section class="next-placeholder-page">
    <article class="next-placeholder-page__hero">
      <div class="next-placeholder-page__hero-copy">
        <span class="next-placeholder-page__eyebrow">{{ eyebrow }}</span>
        <h2>{{ summary }}</h2>
        <p>{{ description }}</p>
      </div>

      <div class="next-placeholder-page__hero-meta">
        <div class="next-placeholder-page__meta-chip">
          <Construction :size="16" />
          <span>Preview scaffold</span>
        </div>

        <div class="next-placeholder-page__meta-card">
          <strong>Page shell 已接入</strong>
          <span>路由、pageKind、认证语义和后续扩展入口已固定。</span>
        </div>
      </div>
    </article>

    <div class="next-placeholder-page__grid">
      <article class="next-placeholder-page__card">
        <header class="next-placeholder-page__card-header">
          <LayoutTemplate :size="18" />
          <h3>预留结构</h3>
        </header>

        <div class="next-placeholder-page__track-list">
          <article
            v-for="track in tracks"
            :key="track.title"
            class="next-placeholder-page__track"
          >
            <strong>{{ track.title }}</strong>
            <p>{{ track.description }}</p>
          </article>
        </div>
      </article>

      <article class="next-placeholder-page__card">
        <header class="next-placeholder-page__card-header">
          <Waypoints :size="18" />
          <h3>后续接线口</h3>
        </header>

        <div class="next-placeholder-page__action-list">
          <RouterLink
            v-for="action in actions"
            :key="action.label"
            class="next-placeholder-page__action"
            :class="{
              'next-placeholder-page__action--primary': action.tone === 'primary',
            }"
            :to="action.to"
          >
            <div class="next-placeholder-page__action-copy">
              <strong>{{ action.label }}</strong>
              <span>{{ action.hint }}</span>
            </div>

            <ArrowRight :size="16" />
          </RouterLink>
        </div>
      </article>
    </div>
  </section>
</template>

<style scoped>
.next-placeholder-page,
.next-placeholder-page__hero-copy,
.next-placeholder-page__hero-meta,
.next-placeholder-page__meta-card,
.next-placeholder-page__card,
.next-placeholder-page__track,
.next-placeholder-page__action-copy {
  display: grid;
}

.next-placeholder-page {
  gap: 18px;
}

.next-placeholder-page__hero,
.next-placeholder-page__card {
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
  border-radius: 24px;
  background: color-mix(in srgb, var(--sl-surface) 92%, transparent);
  box-shadow: var(--sl-shadow-sm);
}

.next-placeholder-page__hero {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(260px, 0.8fr);
  gap: 18px;
  padding: 24px;
}

.next-placeholder-page__hero-copy {
  gap: 10px;
}

.next-placeholder-page__eyebrow {
  font-size: var(--sl-font-size-xs);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--sl-text-tertiary);
}

.next-placeholder-page__hero-copy h2,
.next-placeholder-page__hero-copy p,
.next-placeholder-page__card-header h3,
.next-placeholder-page__track p,
.next-placeholder-page__meta-card span,
.next-placeholder-page__action-copy span {
  margin: 0;
}

.next-placeholder-page__hero-copy h2,
.next-placeholder-page__card-header h3,
.next-placeholder-page__track strong,
.next-placeholder-page__meta-card strong,
.next-placeholder-page__action-copy strong {
  color: var(--sl-text-primary);
}

.next-placeholder-page__hero-copy h2 {
  font-size: clamp(1.8rem, 3vw, 2.4rem);
  line-height: 1;
  letter-spacing: -0.04em;
}

.next-placeholder-page__hero-copy p,
.next-placeholder-page__track p,
.next-placeholder-page__meta-card span,
.next-placeholder-page__action-copy span {
  color: var(--sl-text-secondary);
  line-height: var(--sl-line-height-relaxed);
}

.next-placeholder-page__hero-meta {
  align-content: start;
  gap: 12px;
}

.next-placeholder-page__meta-chip {
  width: fit-content;
  min-height: 34px;
  padding: 0 12px;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--sl-primary) 12%, transparent);
  color: var(--sl-primary);
}

.next-placeholder-page__meta-card {
  gap: 8px;
  padding: 16px 18px;
  border-radius: 18px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 74%, white);
}

.next-placeholder-page__grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 18px;
}

.next-placeholder-page__card {
  gap: 18px;
  padding: 22px;
}

.next-placeholder-page__card-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.next-placeholder-page__track-list,
.next-placeholder-page__action-list {
  display: grid;
  gap: 12px;
}

.next-placeholder-page__track {
  gap: 6px;
  padding: 14px 16px;
  border-radius: 18px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 68%, white);
}

.next-placeholder-page__action {
  min-width: 0;
  min-height: 72px;
  padding: 16px 18px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
  border-radius: 18px;
  background: color-mix(in srgb, var(--sl-surface) 88%, transparent);
  color: var(--sl-text-primary);
  text-decoration: none;
  transition:
    border-color var(--sl-transition-fast),
    background-color var(--sl-transition-fast),
    color var(--sl-transition-fast);
}

.next-placeholder-page__action--primary {
  border-color: color-mix(in srgb, var(--sl-primary) 26%, white);
  background: color-mix(in srgb, var(--sl-primary) 10%, transparent);
}

.next-placeholder-page__action-copy {
  min-width: 0;
  gap: 4px;
}

@media (max-width: 1023px) {
  .next-placeholder-page__hero,
  .next-placeholder-page__grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 767px) {
  .next-placeholder-page__hero,
  .next-placeholder-page__card {
    padding: 18px;
    border-radius: 20px;
  }
}
</style>
