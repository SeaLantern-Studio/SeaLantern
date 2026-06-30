<script setup lang="ts">
import SLButton from "@src/components/common/SLButton.vue";
import { i18n } from "@language";
import type { NextHomeServerCardModel } from "../../pages/home/useNextHomePage";

defineProps<{
  server: NextHomeServerCardModel;
  featured?: boolean;
  previewDataset?: boolean;
}>();

const emit = defineEmits<{
  toggle: [serverId: string];
}>();
</script>

<template>
  <article class="next-home-server-card" :class="{ 'next-home-server-card--featured': featured }">
    <div class="next-home-server-card__topline">
      <span class="next-home-server-card__status" :class="`is-${server.statusTone}`">
        <i class="next-home-server-card__status-dot"></i>
        {{ server.statusText }}
      </span>
      <span class="next-home-server-card__runtime">{{ server.runtimeLabel }}</span>
    </div>

    <div class="next-home-server-card__body">
      <div class="next-home-server-card__title-group">
        <h3>{{ server.name }}</h3>
        <p>{{ server.detail }}</p>
      </div>

      <dl class="next-home-server-card__facts">
        <div>
          <dt>{{ i18n.t("shell.home_server_port") }}</dt>
          <dd>{{ server.portLabel }}</dd>
        </div>
        <div>
          <dt>{{ i18n.t("shell.home_server_memory") }}</dt>
          <dd>{{ server.memoryLabel }}</dd>
        </div>
        <div>
          <dt>{{ i18n.t("shell.home_server_path") }}</dt>
          <dd :title="server.pathLabel">{{ server.pathLabel }}</dd>
        </div>
      </dl>
    </div>

    <div class="next-home-server-card__actions">
      <SLButton
        :variant="server.canStart ? 'primary' : 'danger'"
        :loading="server.isBusy"
        :disabled="previewDataset"
        @click="emit('toggle', server.id)"
      >
        {{ server.actionLabel }}
      </SLButton>

      <span v-if="previewDataset" class="next-home-server-card__hint">
        {{ i18n.t("shell.home_server_preview_hint") }}
      </span>
    </div>
  </article>
</template>

<style scoped>
.next-home-server-card {
  min-width: 0;
  display: grid;
  gap: 16px;
  padding: 18px 20px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
  border-radius: 18px;
  background: color-mix(in srgb, var(--sl-surface) 94%, transparent);
}

.next-home-server-card--featured {
  padding: 22px 24px;
}

.next-home-server-card__topline,
.next-home-server-card__actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.next-home-server-card__status {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-height: 28px;
  padding: 0 12px;
  border-radius: 999px;
  font-size: var(--sl-font-size-sm);
  font-weight: 600;
}

.next-home-server-card__status-dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
  background: currentColor;
}

.next-home-server-card__status.is-success {
  background: color-mix(in srgb, var(--sl-success) 12%, transparent);
  color: var(--sl-success);
}

.next-home-server-card__status.is-warning {
  background: color-mix(in srgb, var(--sl-warning) 12%, transparent);
  color: var(--sl-warning);
}

.next-home-server-card__status.is-danger {
  background: color-mix(in srgb, var(--sl-danger) 12%, transparent);
  color: var(--sl-danger);
}

.next-home-server-card__status.is-neutral {
  background: color-mix(in srgb, var(--sl-text-tertiary) 10%, transparent);
  color: var(--sl-text-secondary);
}

.next-home-server-card__runtime,
.next-home-server-card__hint {
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-sm);
}

.next-home-server-card__body,
.next-home-server-card__title-group {
  display: grid;
  gap: 10px;
}

.next-home-server-card__title-group h3,
.next-home-server-card__title-group p {
  margin: 0;
}

.next-home-server-card__title-group h3 {
  color: var(--sl-text-primary);
  font-size: 1.1rem;
}

.next-home-server-card__title-group p {
  color: var(--sl-text-secondary);
  line-height: var(--sl-line-height-relaxed);
}

.next-home-server-card__facts {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  margin: 0;
}

.next-home-server-card__facts div {
  min-width: 0;
  display: grid;
  gap: 4px;
  padding: 12px;
  border-radius: 14px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 68%, transparent);
}

.next-home-server-card__facts dt {
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-xs);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.next-home-server-card__facts dd {
  margin: 0;
  color: var(--sl-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 767px) {
  .next-home-server-card__topline,
  .next-home-server-card__actions {
    align-items: flex-start;
    flex-direction: column;
  }

  .next-home-server-card__facts {
    grid-template-columns: 1fr;
  }
}
</style>
