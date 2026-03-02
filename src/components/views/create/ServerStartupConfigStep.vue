<script setup lang="ts">
import SLInput from "@components/common/SLInput.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import { i18n } from "@language";

const props = defineProps<{
  serverName: string;
  maxMemory: string;
  minMemory: string;
  port: string;
  onlineMode: boolean;
}>();

const emit = defineEmits<{
  (e: "update:serverName", value: string): void;
  (e: "update:maxMemory", value: string): void;
  (e: "update:minMemory", value: string): void;
  (e: "update:port", value: string): void;
  (e: "update:onlineMode", value: boolean): void;
}>();

function handleNumberInput(event: Event, type: "maxMemory" | "minMemory" | "port") {
  const target = event.target as HTMLInputElement;
  const value = target.value;
  if (value === "" || /^\d+$/.test(value)) {
    emit(`update:${type}`, value);
  }
}
</script>

<template>
  <div class="startup-step">
    <div class="startup-list">
      <div class="startup-row">
        <span class="startup-row-label">{{ i18n.t("create.server_name") }}</span>
        <SLInput
          :placeholder="i18n.t('create.server_name')"
          :model-value="serverName"
          @update:model-value="$emit('update:serverName', $event)"
        />
      </div>

      <div class="startup-pair-row">
        <div class="startup-pair-item">
          <span class="startup-row-label">{{ i18n.t("create.max_memory") }}</span>
          <SLInput
            type="text"
            :model-value="maxMemory"
            @input="handleNumberInput($event, 'maxMemory')"
          />
        </div>
        <div class="startup-pair-item">
          <span class="startup-row-label">{{ i18n.t("create.min_memory") }}</span>
          <SLInput
            type="text"
            :model-value="minMemory"
            @input="handleNumberInput($event, 'minMemory')"
          />
        </div>
      </div>

      <div class="startup-pair-row">
        <div class="startup-pair-item">
          <span class="startup-row-label">{{ i18n.t("settings.default_port") }}</span>
          <SLInput
            type="text"
            :model-value="port"
            :placeholder="i18n.t('create.default_port_placeholder')"
            @input="handleNumberInput($event, 'port')"
          />
        </div>

        <div class="startup-pair-item">
          <span class="startup-row-label">{{ i18n.t("create.online_mode") }}</span>
          <div class="startup-row-control startup-online-box">
            <span class="startup-online-text">
              {{ onlineMode ? i18n.t("create.online_mode_on") : i18n.t("create.online_mode_off") }}
            </span>
            <SLSwitch
              :model-value="onlineMode"
              @update:model-value="$emit('update:onlineMode', $event)"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style src="@styles/components/views/create/ServerStartupConfigStep.css" scoped></style>
