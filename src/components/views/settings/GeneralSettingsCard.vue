<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLSelect from "@components/common/SLSelect.vue";
import { i18n } from "@language";

const props = defineProps<{
  closeServersOnExit: boolean;
  autoAcceptEula: boolean;
  closeAction: "ask" | "minimize" | "close";
}>();

type CloseAction = "ask" | "minimize" | "close";

const emit = defineEmits<{
  (e: "update:closeServersOnExit", value: boolean): void;
  (e: "update:autoAcceptEula", value: boolean): void;
  (e: "update:closeAction", value: CloseAction): void;
  (e: "change"): void;
}>();

function handleCloseActionChange(v: string | number) {
  emit("update:closeAction", v as CloseAction);
  emit("change");
}

const closeActionOptions = [
  { label: i18n.t("settings.close_action_ask"), value: "ask" },
  { label: i18n.t("settings.close_action_minimize"), value: "minimize" },
  { label: i18n.t("settings.close_action_close"), value: "close" },
];
</script>

<template>
  <SLCard :title="i18n.t('settings.general')" :subtitle="i18n.t('settings.general_desc')">
    <div class="sl-settings-group">
      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.auto_stop") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.auto_stop_desc") }}</span>
        </div>
        <SLSwitch
          :model-value="closeServersOnExit"
          @update:model-value="
            (v) => {
              emit('update:closeServersOnExit', v);
              emit('change');
            }
          "
        />
      </div>

      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.auto_eula") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.auto_eula_desc") }}</span>
        </div>
        <SLSwitch
          :model-value="autoAcceptEula"
          @update:model-value="
            (v) => {
              emit('update:autoAcceptEula', v);
              emit('change');
            }
          "
        />
      </div>

      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.close_action") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.close_action_desc") }}</span>
        </div>
        <div class="sl-input-md">
          <SLSelect
            :model-value="closeAction"
            :options="closeActionOptions"
            @update:model-value="handleCloseActionChange"
          />
        </div>
      </div>
    </div>
  </SLCard>
</template>
