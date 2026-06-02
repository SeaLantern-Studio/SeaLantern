<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import PluginSettingsDetails from "@components/views/plugins/PluginSettingsDetails.vue";
import PluginSettingsFields from "@components/views/plugins/PluginSettingsFields.vue";
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
          <PluginSettingsFields
            :fields="plugin?.manifest.settings"
            :field-values="fieldValues"
            @update-field="(key, value) => emit('update-field', key, value)"
          />

          <PluginSettingsDetails
            :dependency-view-model="dependencyViewModel"
            :get-permission-label="getPermissionLabel"
            :get-permission-desc="getPermissionDesc"
          />
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

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--sl-border);
}
</style>
