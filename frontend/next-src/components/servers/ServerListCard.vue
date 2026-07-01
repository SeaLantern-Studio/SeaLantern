<script setup lang="ts">
import {
  CircleDot,
  FolderKanban,
  HardDriveDownload,
  MemoryStick,
  MonitorSmartphone,
  Tag,
} from "@lucide/vue";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import { i18n } from "@language";
import type {
  ServersPageServerItem,
  ServersPageTarget,
} from "@next-src/pages/servers/useServersPage";

interface Props {
  server: ServersPageServerItem;
}

defineProps<Props>();

const emit = defineEmits<{
  select: [serverId: string];
  navigate: [payload: { serverId: string; target: ServersPageTarget }];
}>();
</script>

<template>
  <SLCard
    variant="glass"
    hoverable
    class="server-list-card"
    :class="{ 'server-list-card--current': server.isCurrent }"
    @click="emit('select', server.id)"
  >
    <div class="server-list-card__header">
      <div class="server-list-card__title-block">
        <div class="server-list-card__title-row">
          <h3 class="server-list-card__title">{{ server.name }}</h3>
          <span v-if="server.isCurrent" class="server-list-card__current-chip">{{
            i18n.t("servers.next.card.current")
          }}</span>
        </div>
        <p v-if="server.detailSummary" class="server-list-card__detail">
          {{ server.detailSummary }}
        </p>
      </div>

      <span
        class="server-list-card__status"
        :class="`server-list-card__status--${server.statusTone}`"
      >
        <span class="server-list-card__status-dot"></span>
        <span>{{ server.statusLabel }}</span>
      </span>
    </div>

    <div class="server-list-card__summary-grid">
      <div class="server-list-card__summary-item">
        <MonitorSmartphone :size="16" />
        <div>
          <span class="server-list-card__summary-label">{{
            i18n.t("servers.next.card.runtime")
          }}</span>
          <strong class="server-list-card__summary-value">{{ server.runtimeSummary }}</strong>
        </div>
      </div>

      <div class="server-list-card__summary-item">
        <Tag :size="16" />
        <div>
          <span class="server-list-card__summary-label">{{
            i18n.t("servers.next.card.core")
          }}</span>
          <strong class="server-list-card__summary-value">{{ server.coreSummary }}</strong>
          <span v-if="server.versionSummary" class="server-list-card__summary-meta">
            {{ server.versionSummary }}
          </span>
        </div>
      </div>
    </div>

    <div class="server-list-card__meta-row">
      <div class="server-list-card__meta-chip" :title="server.pathTooltip">
        <FolderKanban :size="14" />
        <span>{{ i18n.t("servers.next.card.path") }}</span>
        <strong>{{ server.pathSummary }}</strong>
      </div>

      <div class="server-list-card__meta-chip">
        <CircleDot :size="14" />
        <span>{{ i18n.t("servers.next.card.port") }}</span>
        <strong>{{ server.portSummary }}</strong>
      </div>

      <div class="server-list-card__meta-chip">
        <MemoryStick :size="14" />
        <span>{{ i18n.t("servers.next.card.memory") }}</span>
        <strong>{{ server.memorySummary }}</strong>
      </div>
    </div>

    <template #footer>
      <div class="server-list-card__footer">
        <div class="server-list-card__selection-note">
          <HardDriveDownload :size="14" />
          <span>{{
            server.isCurrent
              ? i18n.t("servers.next.card.selected")
              : i18n.t("servers.next.card.select")
          }}</span>
        </div>

        <div class="server-list-card__actions">
          <SLButton
            v-for="action in server.actions"
            :key="action.target"
            variant="ghost"
            size="sm"
            @click.stop="emit('navigate', { serverId: server.id, target: action.target })"
          >
            {{ action.label }}
          </SLButton>
        </div>
      </div>
    </template>
  </SLCard>
</template>

<style scoped>
.server-list-card {
  min-width: 0;
}

.server-list-card--current {
  border-color: color-mix(in srgb, var(--sl-primary) 34%, white);
  box-shadow:
    var(--sl-shadow-md, 0 4px 12px rgba(0, 0, 0, 0.08)),
    0 0 0 1px color-mix(in srgb, var(--sl-primary) 18%, transparent);
}

.server-list-card__header {
  display: flex;
  gap: 12px;
  justify-content: space-between;
  align-items: flex-start;
}

.server-list-card__title-block {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.server-list-card__title-row {
  min-width: 0;
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
}

.server-list-card__title {
  min-width: 0;
  margin: 0;
  color: var(--sl-text-primary);
  font-size: 1.125rem;
}

.server-list-card__current-chip {
  padding: 4px 10px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--sl-primary) 14%, transparent);
  color: var(--sl-primary);
  font-size: 0.75rem;
  font-weight: 700;
}

.server-list-card__detail {
  margin: 0;
  color: var(--sl-text-secondary);
  line-height: 1.5;
}

.server-list-card__status {
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-height: 34px;
  padding: 0 12px;
  border-radius: 999px;
  font-size: 0.875rem;
  font-weight: 600;
}

.server-list-card__status-dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
}

.server-list-card__status--running {
  background: rgba(34, 197, 94, 0.12);
  color: var(--sl-success);
}

.server-list-card__status--running .server-list-card__status-dot {
  background: var(--sl-success);
}

.server-list-card__status--starting,
.server-list-card__status--stopping {
  background: rgba(245, 158, 11, 0.12);
  color: var(--sl-warning);
}

.server-list-card__status--starting .server-list-card__status-dot,
.server-list-card__status--stopping .server-list-card__status-dot {
  background: var(--sl-warning);
}

.server-list-card__status--error {
  background: rgba(239, 68, 68, 0.12);
  color: var(--sl-error);
}

.server-list-card__status--error .server-list-card__status-dot {
  background: var(--sl-error);
}

.server-list-card__status--stopped {
  background: color-mix(in srgb, var(--sl-bg-tertiary) 88%, transparent);
  color: var(--sl-text-tertiary);
}

.server-list-card__status--stopped .server-list-card__status-dot {
  background: var(--sl-text-tertiary);
}

.server-list-card__summary-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.server-list-card__summary-item {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  padding: 14px;
  border-radius: 16px;
  background: color-mix(in srgb, var(--sl-surface) 90%, transparent);
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
  color: var(--sl-text-secondary);
}

.server-list-card__summary-item > div {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.server-list-card__summary-label,
.server-list-card__summary-meta {
  color: var(--sl-text-secondary);
  font-size: 0.875rem;
}

.server-list-card__summary-value {
  color: var(--sl-text-primary);
  font-size: 0.95rem;
  line-height: 1.4;
}

.server-list-card__meta-row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.server-list-card__meta-chip {
  min-width: 0;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 88%, transparent);
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
  color: var(--sl-text-secondary);
}

.server-list-card__meta-chip strong {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--sl-text-primary);
  font-weight: 600;
}

.server-list-card__footer {
  display: flex;
  gap: 12px;
  justify-content: space-between;
  align-items: center;
}

.server-list-card__selection-note {
  min-width: 0;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  color: var(--sl-text-secondary);
  font-size: 0.875rem;
}

.server-list-card__actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

@media (max-width: 720px) {
  .server-list-card__header,
  .server-list-card__footer {
    flex-direction: column;
    align-items: stretch;
  }

  .server-list-card__summary-grid {
    grid-template-columns: 1fr;
  }

  .server-list-card__actions {
    justify-content: flex-start;
  }
}

@media (max-width: 520px) {
  .server-list-card__actions :deep(.sl-button) {
    flex: 1 1 calc(50% - 8px);
  }
}
</style>
