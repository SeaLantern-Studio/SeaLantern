<script setup lang="ts">
import { i18n } from "@language";
import type { PluginDependencyViewModel } from "./usePluginDependencies";

defineProps<{
  dependencyViewModel: PluginDependencyViewModel | null;
  getPermissionLabel: (perm: string) => string;
  getPermissionDesc: (perm: string) => string;
}>();
</script>

<template>
  <div v-if="dependencyViewModel" class="plugin-details-section">
    <div v-if="dependencyViewModel.permissions.length" class="detail-block">
      <h4 class="detail-title">{{ i18n.t("plugins.permissions") }}</h4>
      <div class="permission-tags">
        <span
          v-for="perm in dependencyViewModel.permissions"
          :key="perm"
          class="permission-tag"
          :title="getPermissionDesc(perm)"
        >
          {{ getPermissionLabel(perm) }}
        </span>
      </div>
    </div>

    <div v-if="dependencyViewModel.dependencies.length" class="detail-block">
      <h4 class="detail-title">{{ i18n.t("plugins.dependencies") }}</h4>
      <ul class="dependency-list">
        <li v-for="dep in dependencyViewModel.dependencies" :key="dep.id" class="dependency-item">
          <span class="dep-name">{{ dep.name }}</span>
          <span v-if="dep.version" class="dep-version">{{ dep.version }}</span>
          <span :class="['dep-status', `dep-status--${dep.status}`]">{{ dep.statusLabel }}</span>
        </li>
      </ul>
    </div>

    <div v-if="dependencyViewModel.optionalDependencies.length" class="detail-block">
      <h4 class="detail-title">{{ i18n.t("plugins.optional_dependencies") }}</h4>
      <ul class="dependency-list">
        <li
          v-for="dep in dependencyViewModel.optionalDependencies"
          :key="dep.id"
          class="dependency-item"
        >
          <span class="dep-name">{{ dep.name }}</span>
          <span v-if="dep.version" class="dep-version">{{ dep.version }}</span>
          <span :class="['dep-status', `dep-status--${dep.status}`]">{{ dep.statusLabel }}</span>
        </li>
      </ul>
    </div>

    <div v-if="dependencyViewModel.dependents.length" class="detail-block">
      <h4 class="detail-title">{{ i18n.t("plugins.dependents") }}</h4>
      <ul class="dependency-list">
        <li v-for="dep in dependencyViewModel.dependents" :key="dep.id" class="dependency-item">
          <span class="dep-name">{{ dep.name }}</span>
          <span
            :class="[
              'dep-type-tag',
              dep.required ? 'dep-type-tag--required' : 'dep-type-tag--optional',
            ]"
          >
            {{ dep.required ? i18n.t("plugins.dep_required") : i18n.t("plugins.dep_optional") }}
          </span>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.plugin-details-section {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--sl-border, rgba(255, 255, 255, 0.1));
}

.detail-block {
  margin-bottom: 16px;
}

.detail-block:last-child {
  margin-bottom: 0;
}

.detail-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--sl-text-secondary, #94a3b8);
  margin: 0 0 8px 0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.permission-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.permission-tag {
  display: inline-flex;
  align-items: center;
  padding: 4px 10px;
  font-size: 12px;
  font-weight: 500;
  color: var(--sl-text-primary, #e2e8f0);
  background: rgba(99, 102, 241, 0.15);
  border: 1px solid rgba(99, 102, 241, 0.3);
  border-radius: var(--sl-radius-sm);
}

.dependency-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.dependency-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  margin-bottom: 4px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: var(--sl-radius-md);
  font-size: 13px;
}

.dependency-item:last-child {
  margin-bottom: 0;
}

.dep-name {
  flex: 1;
  color: var(--sl-text-primary, #e2e8f0);
  font-weight: 500;
}

.dep-version {
  color: var(--sl-text-tertiary, #64748b);
  font-size: 12px;
  font-family: var(--sl-font-mono, monospace);
}

.dep-status {
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 500;
  border-radius: var(--sl-radius-xs);
}

.dep-status--enabled {
  background: rgba(74, 222, 128, 0.15);
  color: #4ade80;
}

.dep-status--disabled {
  background: rgba(250, 204, 21, 0.15);
  color: #facc15;
}

.dep-status--not-installed {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.dep-type-tag {
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 500;
  border-radius: var(--sl-radius-xs);
}

.dep-type-tag--required {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.dep-type-tag--optional {
  background: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
}
</style>
