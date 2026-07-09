<script setup lang="ts">
import AppBootstrapShellHost from "@components/startup/AppBootstrapShellHost.vue";
import ToastContainer from "@components/common/ToastContainer.vue";
import SLContextMenu from "@components/common/SLContextMenu.vue";
import { PluginComponentRenderer } from "@components/plugin";
import { useGlobalContextMenu } from "@composables/useGlobalContextMenu";
import { useTauriGlobalEvents } from "@composables/useTauriGlobalEvents";

interface Props {
  bootstrapEnabled?: boolean;
  includeUpdateModal?: boolean;
  enableDesktopShellFeatures?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  bootstrapEnabled: true,
  includeUpdateModal: true,
  enableDesktopShellFeatures: true,
});

if (props.enableDesktopShellFeatures) {
  useGlobalContextMenu();
  useTauriGlobalEvents();
}
</script>

<template>
  <AppBootstrapShellHost
    :enabled="props.bootstrapEnabled"
    :include-update-modal="props.includeUpdateModal"
  >
    <slot />
    <ToastContainer />
    <PluginComponentRenderer v-if="props.enableDesktopShellFeatures" />
  </AppBootstrapShellHost>

  <SLContextMenu v-if="props.enableDesktopShellFeatures" />
</template>
