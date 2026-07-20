<script setup lang="ts">
import JavaDownloader from "@components/JavaDownloader.vue";
import { i18n } from "@language";

defineProps<{
  maxMemory: string;
  minMemory: string;
  port: string;
  defaultJavaPath: string;
  defaultJvmArgs: string;
  defaultRunPath: string;
}>();

const emit = defineEmits<{
  (e: "update:maxMemory", value: string): void;
  (e: "update:minMemory", value: string): void;
  (e: "update:port", value: string): void;
  (e: "update:defaultJavaPath", value: string): void;
  (e: "update:defaultJvmArgs", value: string): void;
  (e: "update:defaultRunPath", value: string): void;
  (e: "change"): void;
  (e: "javaInstalled", path: string): void;
  (e: "browseJavaPath"): void;
  (e: "browseRunPath"): void;
}>();
</script>

<template>
  <cmz-card
    :title="i18n.t('settings.server_defaults')"
    :subtitle="i18n.t('settings.server_defaults_desc')"
  >
    <div class="sl-settings-group">
      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.default_memory") }} (MB)</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.max_memory_desc") }}</span>
        </div>
        <div class="sl-input-sm">
          <cmz-input
            :model-value="maxMemory"
            type="number"
            @update:model-value="
              (v) => {
                emit('update:maxMemory', v);
                emit('change');
              }
            "
          />
        </div>
      </div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.min_memory") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.min_memory_desc") }}</span>
        </div>
        <div class="sl-input-sm">
          <cmz-input
            :model-value="minMemory"
            type="number"
            @update:model-value="
              (v) => {
                emit('update:minMemory', v);
                emit('change');
              }
            "
          />
        </div>
      </div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.default_port") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.port_desc") }}</span>
        </div>
        <div class="sl-input-sm">
          <cmz-input
            :model-value="port"
            type="number"
            @update:model-value="
              (v) => {
                emit('update:port', v);
                emit('change');
              }
            "
          />
        </div>
      </div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.default_java") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.default_java_desc") }}</span>
        </div>
        <div class="sl-input-lg">
          <cmz-input
            :model-value="defaultJavaPath"
            :placeholder="i18n.t('settings.default_java_desc')"
            @update:model-value="
              (v) => {
                emit('update:defaultJavaPath', v);
                emit('change');
              }
            "
          >
            <template #suffix>
              <button type="button" class="sl-input-action" @click="emit('browseJavaPath')">
                {{ i18n.t("settings.browse") }}
              </button>
            </template>
          </cmz-input>
        </div>
      </div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.default_run_path") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.default_run_path_desc") }}</span>
        </div>
        <div class="sl-input-lg">
          <cmz-input
            :model-value="defaultRunPath"
            :placeholder="i18n.t('settings.default_run_path_desc')"
            @update:model-value="
              (v) => {
                emit('update:defaultRunPath', v);
                emit('change');
              }
            "
          >
            <template #suffix>
              <button type="button" class="sl-input-action" @click="emit('browseRunPath')">
                {{ i18n.t("settings.browse") }}
              </button>
            </template>
          </cmz-input>
        </div>
      </div>

      <JavaDownloader
        @installed="
          (path) => {
            emit('javaInstalled', path);
            emit('change');
          }
        "
      />

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.jvm_args") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.jvm_args_desc") }}</span>
        </div>
        <cmz-textarea
          :model-value="defaultJvmArgs"
          :placeholder="i18n.t('settings.jvm_args_placeholder')"
          rows="4"
          class="sl-input-lg"
          @update:model-value="
            (v) => {
              emit('update:defaultJvmArgs', v);
              emit('change');
            }
          "
        />
      </div>
    </div>
  </cmz-card>
</template>
