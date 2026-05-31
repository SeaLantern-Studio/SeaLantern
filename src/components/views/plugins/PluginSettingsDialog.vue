<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import { i18n } from "@language";
import type { PluginInfo } from "@type/plugin";
import type { PluginDependencyViewModel } from "./usePluginDependencies";
import { X } from "lucide-vue-next";

defineProps<{
  visible: boolean;
  plugin: PluginInfo | null;
  fieldValues: Record<string, string | number | boolean>;
  saving: boolean;
  getPermissionLabel: (perm: string) => string;
  getPermissionDesc: (perm: string) => string;
  dependencyViewModel: PluginDependencyViewModel | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save"): void;
  (e: "update-field", key: string, value: string | number | boolean): void;
}>();

function getTextValue(values: Record<string, string | number | boolean>, key: string): string {
  const value = values[key];
  return typeof value === "string" ? value : String(value ?? "");
}

function getNumberValue(values: Record<string, string | number | boolean>, key: string): string {
  const value = values[key];
  if (typeof value === "number") {
    return String(value);
  }
  return typeof value === "string" ? value : "0";
}

function getSelectValue(
  values: Record<string, string | number | boolean>,
  key: string,
): string | number | undefined {
  const value = values[key];
  return typeof value === "boolean" ? undefined : value;
}
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="modal-overlay" @click.self="emit('close')">
      <div class="settings-modal glass">
        <div class="modal-header">
          <h2 class="modal-title">
            {{ i18n.t("plugins.settings_title", { name: plugin?.manifest.name }) }}
          </h2>
          <SLButton variant="ghost" icon-only class="modal-close" @click="emit('close')">
            <X :size="20" />
          </SLButton>
        </div>
        <div class="modal-body">
          <div v-for="field in plugin?.manifest.settings" :key="field.key" class="setting-field">
            <label class="setting-label">
              {{ field.label }}
              <span v-if="field.description" class="setting-desc">{{ field.description }}</span>
            </label>
            <SLInput
              v-if="field.type === 'string'"
              :modelValue="getTextValue(fieldValues, field.key)"
              @update:modelValue="emit('update-field', field.key, $event)"
            />
            <div v-else-if="field.type === 'color'" class="setting-color-field">
              <input
                type="color"
                :value="getTextValue(fieldValues, field.key)"
                @input="emit('update-field', field.key, ($event.target as HTMLInputElement).value)"
                class="setting-color-picker"
              />
              <SLInput
                :modelValue="getTextValue(fieldValues, field.key)"
                @update:modelValue="emit('update-field', field.key, $event)"
              />
            </div>
            <SLInput
              v-else-if="field.type === 'number'"
              :modelValue="getNumberValue(fieldValues, field.key)"
              @update:modelValue="emit('update-field', field.key, Number($event))"
              type="number"
            />
            <label v-else-if="field.type === 'boolean'" class="setting-toggle">
              <SLSwitch
                :modelValue="Boolean(fieldValues[field.key])"
                @update:modelValue="emit('update-field', field.key, $event)"
                size="sm"
              />
            </label>
            <SLSelect
              v-else-if="field.type === 'select'"
              :modelValue="getSelectValue(fieldValues, field.key)"
              @update:modelValue="emit('update-field', field.key, $event)"
              :options="field.options || []"
            />
          </div>

          <div class="plugin-details-section">
            <div v-if="dependencyViewModel?.permissions.length" class="detail-block">
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

            <div v-if="dependencyViewModel?.dependencies.length" class="detail-block">
              <h4 class="detail-title">{{ i18n.t("plugins.dependencies") }}</h4>
              <ul class="dependency-list">
                <li
                  v-for="dep in dependencyViewModel.dependencies"
                  :key="dep.id"
                  class="dependency-item"
                >
                  <span class="dep-name">{{ dep.name }}</span>
                  <span v-if="dep.version" class="dep-version">{{ dep.version }}</span>
                  <span :class="['dep-status', `dep-status--${dep.status}`]">{{
                    dep.statusLabel
                  }}</span>
                </li>
              </ul>
            </div>

            <div v-if="dependencyViewModel?.optionalDependencies.length" class="detail-block">
              <h4 class="detail-title">{{ i18n.t("plugins.optional_dependencies") }}</h4>
              <ul class="dependency-list">
                <li
                  v-for="dep in dependencyViewModel.optionalDependencies"
                  :key="dep.id"
                  class="dependency-item"
                >
                  <span class="dep-name">{{ dep.name }}</span>
                  <span v-if="dep.version" class="dep-version">{{ dep.version }}</span>
                  <span :class="['dep-status', `dep-status--${dep.status}`]">{{
                    dep.statusLabel
                  }}</span>
                </li>
              </ul>
            </div>

            <div v-if="dependencyViewModel?.dependents.length" class="detail-block">
              <h4 class="detail-title">{{ i18n.t("plugins.dependents") }}</h4>
              <ul class="dependency-list">
                <li
                  v-for="dep in dependencyViewModel.dependents"
                  :key="dep.id"
                  class="dependency-item"
                >
                  <span class="dep-name">{{ dep.name }}</span>
                  <span
                    :class="[
                      'dep-type-tag',
                      dep.required ? 'dep-type-tag--required' : 'dep-type-tag--optional',
                    ]"
                  >
                    {{
                      dep.required ? i18n.t("plugins.dep_required") : i18n.t("plugins.dep_optional")
                    }}
                  </span>
                </li>
              </ul>
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <SLButton variant="secondary" size="sm" @click="emit('close')">{{
            i18n.t("plugins.cancel")
          }}</SLButton>
          <SLButton variant="primary" size="sm" :loading="saving" @click="emit('save')">{{
            i18n.t("plugins.save")
          }}</SLButton>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.settings-modal {
  width: 90%;
  max-width: 480px;
  max-height: 80vh;
  background: var(--sl-surface);
  border-radius: var(--sl-radius-lg);
  border: 1px solid var(--sl-border);
  box-shadow: var(--sl-shadow-xl);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  font-family: var(--sl-font-sans);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--sl-border);
}

.modal-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0;
}

.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--sl-text-secondary);
  cursor: pointer;
  border-radius: var(--sl-radius-md);
  transition: all 0.2s ease;
}

.modal-close:hover {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-primary);
}

.modal-body {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
}

.setting-field {
  margin-bottom: 16px;
}

.setting-field:last-child {
  margin-bottom: 0;
}

.setting-label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--sl-text-primary);
  margin-bottom: 8px;
}

.setting-desc {
  display: block;
  font-size: 12px;
  font-weight: 400;
  color: var(--sl-text-tertiary);
  margin-top: 2px;
}

.setting-color-field {
  display: flex;
  align-items: center;
  gap: 8px;
}

.setting-color-picker {
  width: 40px;
  height: 38px;
  padding: 2px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: var(--sl-radius-md);
  background: transparent;
  cursor: pointer;
  flex-shrink: 0;
}

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

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--sl-border);
}
</style>
