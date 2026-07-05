<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import SLButton from "@components/common/SLButton.vue";
import SLConfirmDialog from "@components/common/SLConfirmDialog.vue";
import ConsoleInput from "@components/console/ConsoleInput.vue";
import ConsoleOutput from "@components/console/ConsoleOutput.vue";
import WorkbenchPanel from "@src/components/workbench/WorkbenchPanel.vue";
import WorkbenchStatusBanner from "@src/components/workbench/WorkbenchStatusBanner.vue";
import { i18n } from "@language";
import { useServerInstanceConsolePage } from "./useServerInstanceConsolePage";

const page = useServerInstanceConsolePage();

onMounted(async () => {
  await page.activate();
  await page.refresh(false);
});

onUnmounted(() => {
  page.deactivate();
});
</script>

<template>
  <div class="server-instance-console-page">
    <section v-if="page.errorMessage.value" class="server-instance-console-page__error" role="alert">
      <strong>{{ i18n.t("servers.next.error_title") }}</strong>
      <span>{{ page.errorMessage.value }}</span>
    </section>

    <section v-if="page.successMessage.value" class="server-instance-console-page__success">
      <span>{{ page.successMessage.value }}</span>
    </section>

    <WorkbenchStatusBanner v-if="!page.isActive.value" tone="warning">
      <strong>{{ page.statusLabel.value }}</strong>
      <span>{{ i18n.t("common.message_server_required") }}</span>
    </WorkbenchStatusBanner>

    <WorkbenchPanel :title="i18n.t('common.console')" :description="page.statusLabel.value">
      <template #actions>
        <SLButton
          :variant="page.primaryActionVariant.value"
          size="sm"
          :loading="page.lifecycleSubmitting.value"
          :disabled="page.primaryActionDisabled.value"
          @click="page.runPrimaryAction"
        >
          {{ page.primaryActionLabel.value }}
        </SLButton>
        <SLButton
          v-if="page.canForceStop.value"
          variant="danger"
          size="sm"
          :loading="page.forceStopSubmitting.value"
          @click="page.openForceStopDialog"
        >
          {{ i18n.t("console.force_stop") }}
        </SLButton>
        <SLButton variant="secondary" size="sm" @click="page.refresh(true)">
          {{ i18n.t("common.refresh") }}
        </SLButton>
        <SLButton variant="secondary" size="sm" @click="page.scrollToBottom">
          {{ i18n.t("console.back_to_bottom") }}
        </SLButton>
        <SLButton variant="secondary" size="sm" @click="page.copyLogs">
          {{ i18n.t("console.copy_log") }}
        </SLButton>
        <SLButton variant="ghost" size="sm" @click="page.clearLogs">
          {{ i18n.t("console.clear_log") }}
        </SLButton>
      </template>

      <div class="server-instance-console-page__console-shell">
        <div class="server-instance-console-page__output-wrap">
          <ConsoleOutput
            :ref="page.setConsoleOutputRef"
            :console-font-size="page.consoleFontSize.value"
            :console-font-family="page.consoleFontFamily.value"
            :console-letter-spacing="page.consoleLetterSpacing.value"
            :max-log-lines="page.maxLogLines.value"
            :user-scrolled-up="page.userScrolledUp.value"
            @scroll="(value) => (page.userScrolledUp.value = value)"
            @scroll-to-bottom="page.scrollToBottom"
          />
        </div>

        <ConsoleInput
          :console-font-size="page.consoleFontSize.value"
          @send-command="page.sendCommand"
        />
      </div>
    </WorkbenchPanel>

    <SLConfirmDialog
      :visible="page.forceStopDialogVisible.value"
      :title="i18n.t('console.force_stop')"
      :message="i18n.t('console.force_stop_confirm')"
      :confirmText="i18n.t('console.force_stop')"
      :cancelText="i18n.t('common.cancel')"
      :loading="page.forceStopSubmitting.value"
      confirmVariant="danger"
      dangerous
      @confirm="page.confirmForceStop"
      @close="page.closeForceStopDialog"
    />
  </div>
</template>

<style scoped>
.server-instance-console-page {
  display: grid;
  gap: 18px;
}

.server-instance-console-page__error,
.server-instance-console-page__success {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px 16px;
  border-radius: 18px;
}

.server-instance-console-page__error {
  border: 1px solid rgba(239, 68, 68, 0.24);
  background: rgba(239, 68, 68, 0.1);
  color: var(--sl-error);
}

.server-instance-console-page__success {
  border: 1px solid rgba(34, 197, 94, 0.24);
  background: rgba(34, 197, 94, 0.1);
  color: rgb(var(--sl-success));
}

.server-instance-console-page__console-shell {
  display: grid;
  gap: 14px;
}

.server-instance-console-page__output-wrap {
  min-height: 380px;
  overflow: hidden;
  border-radius: 18px;
}
</style>
