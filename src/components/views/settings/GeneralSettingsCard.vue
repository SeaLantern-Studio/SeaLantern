<script setup lang="ts">
import { computed } from "vue";
import { i18n } from "@language";

const props = defineProps<{
  closeServersOnExit: boolean;
  closeServersOnUpdate: boolean;
  autoAcceptEula: boolean;
  autoLightweightMinutes: number | null;
  closeAction: "ask" | "minimize" | "close";
}>();

type CloseAction = "ask" | "minimize" | "close";

const emit = defineEmits<{
  (e: "update:closeServersOnExit", value: boolean): void;
  (e: "update:closeServersOnUpdate", value: boolean): void;
  (e: "update:autoAcceptEula", value: boolean): void;
  (e: "update:autoLightweightMinutes", value: number | null): void;
  (e: "update:closeAction", value: CloseAction): void;
  (e: "change"): void;
}>();

function handleCloseActionChange(v: string | number) {
  emit("update:closeAction", v as CloseAction);
  emit("change");
}

function handleAutoLightweightToggle(enabled: boolean) {
  emit("update:autoLightweightMinutes", enabled ? 3 : null);
  emit("change");
}

function handleAutoLightweightDelayChange(value: string | number) {
  emit("update:autoLightweightMinutes", Number(value));
  emit("change");
}

const closeActionOptions = computed(() => [
  { label: i18n.t("settings.close_action_ask"), value: "ask" },
  { label: i18n.t("settings.close_action_minimize"), value: "minimize" },
  { label: i18n.t("settings.close_action_close"), value: "close" },
]);

const autoLightweightDelayOptions = computed(() =>
  [1, 3, 5, 10].map((minutes) => ({
    label: `${minutes} ${i18n.t("settings.minutes")}`,
    value: minutes,
  })),
);
</script>

<template>
  <cmz-card :title="i18n.t('settings.general')" :subtitle="i18n.t('settings.general_desc')">
    <div class="sl-settings-group">
      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.auto_stop") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.auto_stop_desc") }}</span>
        </div>
        <cmz-switch
          :model-value="closeServersOnExit"
          @update:model-value="
            (v: boolean) => {
              emit('update:closeServersOnExit', v);
              emit('change');
            }
          "
        />
      </div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.auto_lightweight") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.auto_lightweight_desc") }}</span>
        </div>
        <cmz-switch
          :model-value="autoLightweightMinutes !== null"
          @update:model-value="handleAutoLightweightToggle"
        />
      </div>

      <div v-if="autoLightweightMinutes !== null" class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.auto_lightweight_delay") }}</span>
          <span class="settings-entry-desc">{{
            i18n.t("settings.auto_lightweight_delay_desc")
          }}</span>
        </div>
        <cmz-select
          :model-value="autoLightweightMinutes"
          :options="autoLightweightDelayOptions"
          @update:model-value="handleAutoLightweightDelayChange"
        />
      </div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.update_auto_stop") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.update_auto_stop_desc") }}</span>
        </div>
        <cmz-switch
          :model-value="closeServersOnUpdate"
          @update:model-value="
            (v: boolean) => {
              emit('update:closeServersOnUpdate', v);
              emit('change');
            }
          "
        />
      </div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.auto_eula") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.auto_eula_desc") }}</span>
        </div>
        <cmz-switch
          :model-value="autoAcceptEula"
          @update:model-value="
            (v: boolean) => {
              emit('update:autoAcceptEula', v);
              emit('change');
            }
          "
        />
      </div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.close_action") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.close_action_desc") }}</span>
        </div>
        <cmz-select
          :model-value="closeAction"
          :options="closeActionOptions"
          @update:model-value="handleCloseActionChange"
        />
      </div>
    </div>
  </cmz-card>
</template>
