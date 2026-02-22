<script setup lang="ts">
import { computed } from "vue";
import SLModal from "@components/common/SLModal.vue";
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";
import { type PermissionMetadata, groupPermissionsByDangerLevel } from "@type/plugin";
import {
  AlertTriangle,
  Info,
  Globe,
  Folder,
  Server,
  Terminal,
  MousePointer,
  Monitor,
  FileText,
  HardDrive,
  Plug,
  Palette,
  LockOpen,
  HelpCircle,
} from "lucide-vue-next";

const permIconMap: Record<string, unknown> = {
  Globe,
  Folder,
  Server,
  Terminal,
  MousePointer,
  Monitor,
  FileText,
  HardDrive,
  Plug,
  Palette,
  AlertTriangle,
  LockOpen,
  HelpCircle,
};

const props = defineProps<{
  show: boolean;
  pluginName: string;
  permissions: string[];
}>();

const emit = defineEmits<{
  (e: "confirm"): void;
  (e: "cancel"): void;
}>();

const groupedPermissions = computed(() => {
  return groupPermissionsByDangerLevel(props.permissions);
});

const hasCritical = computed(() => groupedPermissions.value.critical.length > 0);

function getPermissionName(perm: PermissionMetadata): string {
  return i18n.te(perm.name) ? i18n.t(perm.name) : perm.id;
}

function getPermissionDesc(perm: PermissionMetadata): string {
  return i18n.te(perm.description) ? i18n.t(perm.description) : "";
}

function handleConfirm() {
  emit("confirm");
}

function handleCancel() {
  emit("cancel");
}
</script>

<template>
  <SLModal :visible="show" @close="handleCancel">
    <div class="permission-dialog">
      <div class="dialog-header" :class="{ critical: hasCritical }">
        <div class="header-icon" :class="{ critical: hasCritical }">
          <AlertTriangle v-if="hasCritical" :size="22" :stroke-width="2" />
          <Info v-else :size="22" :stroke-width="2" />
        </div>
        <div class="header-text">
          <h2>{{ i18n.t("plugins.permission.warning_title") }}</h2>
          <p>
            {{ i18n.t("plugins.permission.warning_message", { name: pluginName }) }}
          </p>
        </div>
      </div>

      <div class="permission-list">
        <template
          v-if="groupedPermissions.critical.length > 0 || groupedPermissions.dangerous.length > 0"
        >
          <div class="section-label danger-label">
            <span class="section-dot danger"></span>
            {{ i18n.t("plugins.permission.danger_dangerous") }}
          </div>
          <div class="perm-row-list">
            <div
              v-for="perm in [...groupedPermissions.critical, ...groupedPermissions.dangerous]"
              :key="perm.id"
              class="perm-row danger"
              :title="getPermissionDesc(perm)"
            >
              <span class="row-dot danger"></span>
              <span class="row-name">{{ getPermissionName(perm) }}</span>
              <span v-if="getPermissionDesc(perm)" class="row-tooltip">{{
                getPermissionDesc(perm)
              }}</span>
            </div>
          </div>
        </template>

        <template v-if="groupedPermissions.normal.length > 0">
          <div class="section-label normal-label">
            <span class="section-dot normal"></span>
            {{ i18n.t("plugins.permission.danger_normal") }}
          </div>
          <div class="perm-row-list">
            <div
              v-for="perm in groupedPermissions.normal"
              :key="perm.id"
              class="perm-row normal"
              :title="getPermissionDesc(perm)"
            >
              <span class="row-dot normal"></span>
              <span class="row-name">{{ getPermissionName(perm) }}</span>
              <span v-if="getPermissionDesc(perm)" class="row-tooltip">{{
                getPermissionDesc(perm)
              }}</span>
            </div>
          </div>
        </template>
      </div>

      <div v-if="hasCritical" class="critical-warning">
        <AlertTriangle :size="16" :stroke-width="2.5" />
        <span>{{ i18n.t("plugins.permission.critical_warning") }}</span>
      </div>

      <div class="dialog-actions">
        <SLButton variant="secondary" @click="handleCancel">
          {{ i18n.t("plugins.permission.warning_cancel") }}
        </SLButton>
        <SLButton :variant="hasCritical ? 'danger' : 'primary'" @click="handleConfirm">
          {{ i18n.t("plugins.permission.warning_confirm") }}
        </SLButton>
      </div>
    </div>
  </SLModal>
</template>

<style scoped>
.permission-dialog {
  width: 440px;
  max-width: 100%;
}

.dialog-header {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 20px;
  padding-bottom: 18px;
  border-bottom: 1px solid var(--sl-border);
}

.header-icon {
  flex-shrink: 0;
  width: 42px;
  height: 42px;
  border-radius: var(--sl-radius-md);
  background: rgba(245, 158, 11, 0.12);
  border: 1px solid rgba(245, 158, 11, 0.25);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--sl-warning);
}

.header-icon.critical {
  background: rgba(239, 68, 68, 0.12);
  border-color: rgba(239, 68, 68, 0.25);
  color: var(--sl-error);
}

.header-text h2 {
  margin: 0 0 3px 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  letter-spacing: -0.01em;
}

.header-text p {
  margin: 0;
  font-size: 0.8125rem;
  color: var(--sl-text-secondary);
  line-height: 1.5;
}

.permission-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 14px;
}

.section-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  margin-bottom: 4px;
}

.danger-label {
  color: var(--sl-error);
}
.normal-label {
  color: var(--sl-text-tertiary);
}

.section-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}
.section-dot.danger {
  background: var(--sl-error);
}
.section-dot.normal {
  background: var(--sl-text-tertiary);
}

.perm-row-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.perm-row {
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px 4px 8px;
  border-radius: var(--sl-radius-full);
  border: 1px solid var(--sl-border);
  background: var(--sl-bg-secondary);
  cursor: default;
  transition:
    background var(--sl-transition-fast),
    border-color var(--sl-transition-fast);
}

.perm-row.danger:hover {
  background: rgba(239, 68, 68, 0.08);
  border-color: rgba(239, 68, 68, 0.3);
}

.perm-row.normal:hover {
  background: var(--sl-surface-hover);
}

.row-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}
.row-dot.danger {
  background: var(--sl-error);
}
.row-dot.normal {
  background: var(--sl-text-tertiary);
}

.row-name {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-text-primary);
  white-space: nowrap;
}

.row-tooltip {
  display: none;
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  background: var(--sl-bg-tertiary);
  border: 1px solid var(--sl-border);
  color: var(--sl-text-secondary);
  font-size: 0.75rem;
  line-height: 1.5;
  padding: 6px 10px;
  border-radius: var(--sl-radius-md);
  width: max-content;
  max-width: 240px;
  white-space: normal;
  word-break: break-all;
  z-index: 100;
  box-shadow: var(--sl-shadow-md);
  pointer-events: none;
}

.perm-row:hover .row-tooltip {
  display: block;
}

.critical-warning {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.18);
  border-radius: var(--sl-radius-md);
  margin-bottom: 16px;
  color: var(--sl-error);
  font-size: 0.8rem;
  font-weight: 500;
  line-height: 1.5;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding-top: 16px;
  border-top: 1px solid var(--sl-border);
}
</style>
