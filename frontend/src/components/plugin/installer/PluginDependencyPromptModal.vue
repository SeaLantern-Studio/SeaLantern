<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLModal from "@components/common/SLModal.vue";
import { i18n } from "@language";
import type { MissingDependency } from "@type/plugin";

defineProps<{
  visible: boolean;
  installedPluginName: string;
  missingDependencies: MissingDependency[];
  getDepDisplayName: (depId: string) => string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "go-market"): void;
}>();
</script>

<template>
  <SLModal :visible="visible" :title="i18n.t('plugins.missing_deps_title')" @close="emit('close')">
    <div class="dependency-dialog">
      <p class="dependency-intro">
        {{ i18n.t("plugins.missing_deps_intro", { name: installedPluginName }) }}
      </p>
      <ul class="dependency-list">
        <li v-for="dep in missingDependencies" :key="dep.id" class="dependency-item">
          <span class="dependency-name">{{ getDepDisplayName(dep.id) }}</span>
          <span v-if="dep.version_requirement" class="dependency-version">
            {{ dep.version_requirement }}
          </span>
          <span :class="['dependency-badge', dep.required ? 'required' : 'optional']">
            {{ dep.required ? i18n.t("plugins.dep_required") : i18n.t("plugins.dep_optional") }}
          </span>
        </li>
      </ul>
      <p class="dependency-hint">
        {{ i18n.t("plugins.missing_deps_hint") }}
      </p>
    </div>
    <template #footer>
      <SLButton variant="secondary" size="sm" @click="emit('close')">{{
        i18n.t("plugins.later")
      }}</SLButton>
      <SLButton variant="primary" size="sm" @click="emit('go-market')">{{
        i18n.t("plugins.go_market")
      }}</SLButton>
    </template>
  </SLModal>
</template>

<style scoped>
.dependency-dialog {
  padding: 4px 0;
}

.dependency-intro {
  margin: 0 0 16px 0;
  color: var(--sl-text-secondary, #6b7280);
  font-size: 14px;
  line-height: 1.6;
}

.dependency-list {
  list-style: none;
  margin: 0 0 16px 0;
  padding: 0;
}

.dependency-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  margin-bottom: 8px;
  background: var(--sl-bg-tertiary, rgba(255, 255, 255, 0.05));
  border-radius: var(--sl-radius-md);
  border: 1px solid var(--sl-border, rgba(255, 255, 255, 0.08));
}

.dependency-item:last-child {
  margin-bottom: 0;
}

.dependency-name {
  font-weight: 500;
  color: var(--sl-text-primary, #e2e8f0);
  font-size: 14px;
}

.dependency-version {
  font-size: 12px;
  color: var(--sl-text-tertiary, #64748b);
  font-family: monospace;
}

.dependency-badge {
  margin-left: auto;
  padding: 2px 8px;
  border-radius: var(--sl-radius-xs);
  font-size: 11px;
  font-weight: 500;
}

.dependency-badge.required {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.dependency-badge.optional {
  background: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
}

.dependency-hint {
  margin: 0;
  color: var(--sl-text-tertiary, #64748b);
  font-size: 13px;
  line-height: 1.5;
}
</style>
