<script setup lang="ts">
import { computed } from "vue";
import { i18n } from "@language";

const props = defineProps<{
  closeServersOnExit: boolean;
  closeServersOnUpdate: boolean;
  autoAcceptEula: boolean;
  closeAction: "ask" | "minimize" | "close";
}>();

type CloseAction = "ask" | "minimize" | "close";

const emit = defineEmits<{
  (e: "update:closeServersOnExit", value: boolean): void;
  (e: "update:closeServersOnUpdate", value: boolean): void;
  (e: "update:autoAcceptEula", value: boolean): void;
  (e: "update:closeAction", value: CloseAction): void;
  (e: "change"): void;
}>();

function handleCloseActionChange(v: string | number) {
  emit("update:closeAction", v as CloseAction);
  emit("change");
}

const closeActionOptions = computed(() => [
  { label: i18n.t("settings.close_action_ask"), value: "ask" },
  { label: i18n.t("settings.close_action_minimize"), value: "minimize" },
  { label: i18n.t("settings.close_action_close"), value: "close" },
]);
</script>

<template>
  <cmz-card :title="i18n.t('settings.general')" :subtitle="i18n.t('settings.general_desc')">
    <div class="sl-settings-group">
      <cmz-form-field
        :label="i18n.t('settings.auto_stop')"
        :hint="i18n.t('settings.auto_stop_desc')"
        label-position="left"
      >
        <cmz-switch
          :model-value="closeServersOnExit"
          @update:model-value="
            (v) => {
              emit('update:closeServersOnExit', v);
              emit('change');
            }
          "
        />
      </cmz-form-field>

      <cmz-form-field
        :label="i18n.t('settings.update_auto_stop')"
        :hint="i18n.t('settings.update_auto_stop_desc')"
        label-position="left"
      >
        <cmz-switch
          :model-value="closeServersOnUpdate"
          @update:model-value="
            (v) => {
              emit('update:closeServersOnUpdate', v);
              emit('change');
            }
          "
        />
      </cmz-form-field>

      <cmz-form-field
        :label="i18n.t('settings.auto_eula')"
        :hint="i18n.t('settings.auto_eula_desc')"
        label-position="left"
      >
        <cmz-switch
          :model-value="autoAcceptEula"
          @update:model-value="
            (v) => {
              emit('update:autoAcceptEula', v);
              emit('change');
            }
          "
        />
      </cmz-form-field>

      <cmz-form-field
        :label="i18n.t('settings.close_action')"
        :hint="i18n.t('settings.close_action_desc')"
        label-position="left"
      >
        <div class="sl-input-md">
          <cmz-select
            :model-value="closeAction"
            :options="closeActionOptions"
            @update:model-value="handleCloseActionChange"
          />
        </div>
      </cmz-form-field>
    </div>
  </cmz-card>
</template>
