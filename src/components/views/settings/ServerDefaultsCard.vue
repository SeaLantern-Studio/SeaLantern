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
      <cmz-form-field
        :label="i18n.t('settings.default_memory') + ' (MB)'"
        :hint="i18n.t('settings.max_memory_desc')"
        label-position="left"
      >
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
      </cmz-form-field>

      <cmz-form-field
        :label="i18n.t('settings.min_memory')"
        :hint="i18n.t('settings.min_memory_desc')"
        label-position="left"
      >
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
      </cmz-form-field>

      <cmz-form-field
        :label="i18n.t('settings.default_port')"
        :hint="i18n.t('settings.port_desc')"
        label-position="left"
      >
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
      </cmz-form-field>

      <cmz-form-field
        :label="i18n.t('settings.default_java')"
        :hint="i18n.t('settings.default_java_desc')"
        label-position="left"
      >
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
      </cmz-form-field>

      <cmz-form-field
        :label="i18n.t('settings.default_run_path')"
        :hint="i18n.t('settings.default_run_path_desc')"
        label-position="left"
      >
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
      </cmz-form-field>

      <JavaDownloader
        @installed="
          (path) => {
            emit('javaInstalled', path);
            emit('change');
          }
        "
      />

      <cmz-form-field :label="i18n.t('settings.jvm_args')" :hint="i18n.t('settings.jvm_args_desc')">
        <cmz-textarea
          :model-value="defaultJvmArgs"
          :placeholder="i18n.t('settings.jvm_args_placeholder')"
          :rows="3"
          @update:model-value="
            (v) => {
              emit('update:defaultJvmArgs', v);
              emit('change');
            }
          "
        />
      </cmz-form-field>
    </div>
  </cmz-card>
</template>
