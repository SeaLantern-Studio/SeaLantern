<script setup lang="ts">
import { computed } from "vue";
import { SLBadge, SLButton, SLCard } from "@components/common";

interface StatusBadge {
  text: string;
  variant: "primary" | "success" | "warning" | "neutral" | "info";
}

interface Props {
  title: string;
  description: string;
  sourceLabel: string;
  running: boolean;
  pendingRestart: boolean;
  disabled: boolean;
  loading?: boolean;
  actionLabel: string;
  statusBadges: StatusBadge[];
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
});

const emit = defineEmits<{
  select: [];
  restart: [];
}>();

const cardClasses = computed(() => ({
  "ui-shell-card": true,
  "ui-shell-card--running": props.running,
  "ui-shell-card--pending": props.pendingRestart,
}));

function handleAction() {
  if (props.pendingRestart) {
    emit("restart");
    return;
  }

  emit("select");
}
</script>

<template>
  <SLCard :class="cardClasses" padding="md">
    <div class="ui-shell-card-header">
      <div class="ui-shell-card-title-group">
        <h3 class="ui-shell-card-title">{{ title }}</h3>
        <p class="ui-shell-card-description">{{ description }}</p>
      </div>
      <SLBadge class="ui-shell-card-source" :text="sourceLabel" variant="neutral" size="small" />
    </div>

    <div class="ui-shell-card-badges">
      <SLBadge
        v-for="badge in statusBadges"
        :key="`${title}-${badge.text}`"
        :text="badge.text"
        :variant="badge.variant"
        size="small"
      />
    </div>

    <template #footer>
      <div class="ui-shell-card-footer">
        <SLButton
          size="sm"
          :variant="pendingRestart ? 'secondary' : 'primary'"
          :disabled="disabled"
          :loading="loading"
          @click="handleAction"
        >
          {{ actionLabel }}
        </SLButton>
      </div>
    </template>
  </SLCard>
</template>

<style scoped>
.ui-shell-card {
  border: 1px solid var(--sl-border-light);
}

.ui-shell-card--running {
  border-color: rgba(var(--sl-success), 0.35);
}

.ui-shell-card--pending {
  border-color: rgba(var(--sl-warning), 0.4);
}

.ui-shell-card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--sl-space-sm);
}

.ui-shell-card-title-group {
  min-width: 0;
}

.ui-shell-card-title {
  margin: 0;
  font-size: 1rem;
  color: var(--sl-text-primary);
}

.ui-shell-card-description {
  margin: 0.375rem 0 0;
  font-size: 0.875rem;
  line-height: 1.5;
  color: var(--sl-text-secondary);
}

.ui-shell-card-source {
  flex-shrink: 0;
}

.ui-shell-card-badges {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sl-space-xs);
  margin-top: var(--sl-space-md);
}

.ui-shell-card-footer {
  display: flex;
  justify-content: flex-end;
}
</style>
