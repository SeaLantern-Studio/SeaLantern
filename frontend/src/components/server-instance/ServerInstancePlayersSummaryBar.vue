<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";

interface Props {
  totalCount: number;
  onlineCount: number;
  opCount: number;
  bannedCount: number;
  bannedIpCount: number;
  refreshing: boolean;
  running: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  refresh: [];
}>();
</script>

<template>
  <section class="server-instance-players-summary-bar">
    <div class="server-instance-players-summary-bar__stats">
      <article class="server-instance-players-summary-bar__stat">
        <strong>{{ totalCount }}</strong>
        <span>Players</span>
      </article>
      <article
        class="server-instance-players-summary-bar__stat server-instance-players-summary-bar__stat--online"
      >
        <strong>{{ onlineCount }}</strong>
        <span>Online</span>
      </article>
      <article
        class="server-instance-players-summary-bar__stat server-instance-players-summary-bar__stat--op"
      >
        <strong>{{ opCount }}</strong>
        <span>OP</span>
      </article>
      <article
        class="server-instance-players-summary-bar__stat server-instance-players-summary-bar__stat--danger"
      >
        <strong>{{ bannedCount }}</strong>
        <span>Name bans</span>
      </article>
      <article class="server-instance-players-summary-bar__stat">
        <strong>{{ bannedIpCount }}</strong>
        <span>IP bans</span>
      </article>
    </div>

    <div class="server-instance-players-summary-bar__actions">
      <span class="server-instance-players-summary-bar__hint">
        {{ running ? "Live logs + file summary" : "File summary only" }}
      </span>
      <SLButton variant="secondary" size="sm" :loading="refreshing" @click="emit('refresh')">
        Refresh
      </SLButton>
    </div>
  </section>
</template>

<style scoped>
.server-instance-players-summary-bar,
.server-instance-players-summary-bar__stats,
.server-instance-players-summary-bar__actions {
  display: flex;
}

.server-instance-players-summary-bar {
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
  padding: 18px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
  border-radius: 22px;
  background: color-mix(in srgb, var(--sl-surface) 92%, transparent);
}

.server-instance-players-summary-bar__stats {
  flex-wrap: wrap;
  gap: 12px;
}

.server-instance-players-summary-bar__stat {
  min-width: 108px;
  display: grid;
  gap: 4px;
  padding: 12px 14px;
  border-radius: 16px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 80%, transparent);
}

.server-instance-players-summary-bar__stat strong {
  color: var(--sl-text-primary);
  font-size: 1.2rem;
}

.server-instance-players-summary-bar__stat span,
.server-instance-players-summary-bar__hint {
  color: var(--sl-text-secondary);
}

.server-instance-players-summary-bar__stat--online strong {
  color: var(--sl-success);
}

.server-instance-players-summary-bar__stat--op strong {
  color: #d4a72c;
}

.server-instance-players-summary-bar__stat--danger strong {
  color: var(--sl-error);
}

.server-instance-players-summary-bar__actions {
  align-items: center;
  gap: 12px;
}
</style>
