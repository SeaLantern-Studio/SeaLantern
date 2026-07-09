<script setup lang="ts">
import SLBadge from "@components/common/SLBadge.vue";
import SLButton from "@components/common/SLButton.vue";
import type { DownloadTaskEntry } from "@src/pages/downloads/useDownloadsPage";
import { i18n } from "@language";
import { formatBytes } from "@utils/formatters";

interface Props {
  tasks: readonly DownloadTaskEntry[];
  title: string;
  emptyTitle: string;
  emptyDescription: string;
}

defineProps<Props>();

const emit = defineEmits<{
  createInstance: [task: DownloadTaskEntry];
}>();

function formatTimestamp(value: number): string {
  return new Intl.DateTimeFormat(undefined, {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(value);
}
</script>

<template>
  <section class="download-task-list">
    <h4 class="download-task-list__title">{{ title }}</h4>

    <div v-if="tasks.length === 0" class="download-task-list__empty">
      <strong>{{ emptyTitle }}</strong>
      <p>{{ emptyDescription }}</p>
    </div>

    <div v-else class="download-task-list__items">
      <article v-for="task in tasks" :key="task.id" class="download-task-list__item">
        <div class="download-task-list__meta-row">
          <div class="download-task-list__copy">
            <strong class="download-task-list__item-title">{{ task.title }}</strong>
            <p class="download-task-list__item-detail">{{ task.detail }}</p>
          </div>

          <SLBadge :text="task.statusText" :variant="task.tone" size="small" rounded="full" />
        </div>

        <div class="download-task-list__progress">
          <div class="download-task-list__progress-bar">
            <span
              class="download-task-list__progress-fill"
              :style="{ width: `${task.progress}%` }"
            />
          </div>
          <span class="download-task-list__progress-value">{{ task.progress.toFixed(1) }}%</span>
        </div>

        <dl class="download-task-list__facts">
          <div>
            <dt>{{ i18n.t("downloads.next.tasks.target_label") }}</dt>
            <dd>{{ task.savePath }}</dd>
          </div>
          <div>
            <dt>{{ i18n.t("downloads.next.tasks.size_label") }}</dt>
            <dd>{{ formatBytes(task.downloaded) }} / {{ formatBytes(task.totalSize) }}</dd>
          </div>
          <div>
            <dt>{{ i18n.t("downloads.next.tasks.time_label") }}</dt>
            <dd>{{ formatTimestamp(task.createdAt) }}</dd>
          </div>
        </dl>

        <p v-if="task.errorMessage" class="download-task-list__error">{{ task.errorMessage }}</p>

        <div
          v-if="task.canCreateInstance && task.isFinished && !task.errorMessage"
          class="download-task-list__actions"
        >
          <SLButton variant="secondary" size="sm" @click="emit('createInstance', task)">
            {{ i18n.t("downloads.next.tasks.open_create") }}
          </SLButton>
        </div>
      </article>
    </div>
  </section>
</template>

<style scoped>
.download-task-list {
  min-width: 0;
  display: grid;
  gap: 10px;
}

.download-task-list__title,
.download-task-list__empty strong,
.download-task-list__empty p,
.download-task-list__item-title,
.download-task-list__item-detail,
.download-task-list__error,
.download-task-list__facts dt,
.download-task-list__facts dd {
  margin: 0;
}

.download-task-list__title {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}

.download-task-list__empty {
  display: grid;
  gap: 4px;
  padding: 16px 18px;
  border-radius: 18px;
  border: 1px dashed color-mix(in srgb, var(--sl-border) 72%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 68%, transparent);
}

.download-task-list__empty p {
  color: var(--sl-text-secondary);
  font-size: 0.84rem;
  line-height: 1.45;
}

.download-task-list__items {
  display: grid;
  gap: 12px;
}

.download-task-list__item {
  display: grid;
  gap: 12px;
  padding: 16px 18px;
  border-radius: 18px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 68%, transparent);
}

.download-task-list__meta-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.download-task-list__copy {
  min-width: 0;
  display: grid;
  gap: 4px;
}

.download-task-list__item-title {
  font-size: 0.94rem;
  color: var(--sl-text-primary);
}

.download-task-list__item-detail {
  color: var(--sl-text-secondary);
  font-size: 0.82rem;
  line-height: 1.45;
  word-break: break-word;
}

.download-task-list__progress {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 10px;
  align-items: center;
}

.download-task-list__progress-bar {
  height: 8px;
  overflow: hidden;
  border-radius: 999px;
  background: color-mix(in srgb, var(--sl-border) 78%, transparent);
}

.download-task-list__progress-fill {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(
    90deg,
    var(--sl-primary),
    color-mix(in srgb, var(--sl-primary) 65%, white)
  );
}

.download-task-list__progress-value {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--sl-text-secondary);
}

.download-task-list__facts {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 10px;
}

.download-task-list__facts div {
  display: grid;
  gap: 4px;
}

.download-task-list__facts dt {
  font-size: 0.76rem;
  color: var(--sl-text-tertiary);
}

.download-task-list__facts dd {
  color: var(--sl-text-primary);
  font-size: 0.82rem;
  line-height: 1.45;
  word-break: break-word;
}

.download-task-list__error {
  color: var(--sl-error);
  font-size: 0.82rem;
  line-height: 1.45;
}

.download-task-list__actions {
  display: flex;
  justify-content: flex-start;
}

@media (max-width: 720px) {
  .download-task-list__meta-row {
    flex-direction: column;
  }

  .download-task-list__progress {
    grid-template-columns: 1fr;
  }
}
</style>
