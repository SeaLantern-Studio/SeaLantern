<script setup lang="ts">
import { computed } from "vue";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import { i18n } from "@language";
import ExtensionEntryList from "@next-src/components/server-instance/extensions/ExtensionEntryList.vue";
import ExtensionsSummaryBar from "@next-src/components/server-instance/extensions/ExtensionsSummaryBar.vue";
import { useServerInstanceExtensionsPage } from "./useServerInstanceExtensionsPage";

const page = useServerInstanceExtensionsPage();
const summary = computed(() => page.summary.value);
const loading = computed(() => page.loading.value);
const refreshing = computed(() => page.refreshing.value);
const errorMessage = computed(() => page.errorMessage.value);
const hasAnyEntries = computed(() => page.hasAnyEntries.value);
const isMissingServer = computed(() => page.isMissingServer.value);
const hasNoDirectories = computed(() => page.hasNoDirectories.value);
const hasEmptyDirectories = computed(() => page.hasEmptyDirectories.value);
const pluginsTitle = computed(() => page.pluginsTitle.value);
const modsTitle = computed(() => page.modsTitle.value);
const otherTitle = computed(() => page.otherTitle.value);
</script>

<template>
  <div class="server-instance-extensions-page">
    <ExtensionsSummaryBar
      :has-plugins-dir="summary.has_plugins_dir"
      :has-mods-dir="summary.has_mods_dir"
      :plugin-count="summary.plugin_entries.length"
      :mod-count="summary.mod_entries.length"
      :other-count="summary.other_entries.length"
      :refreshing="refreshing"
      @refresh="page.refresh(true)"
    />

    <section v-if="errorMessage" class="server-instance-extensions-page__error" role="alert">
      <strong>{{ i18n.t("servers.next.error_title") }}</strong>
      <span>{{ errorMessage }}</span>
      <div>
        <SLButton variant="secondary" size="sm" @click="page.refresh(true)">
          {{ i18n.t("servers.next.instance.extensions.retry") }}
        </SLButton>
      </div>
    </section>

    <section v-else-if="loading && !hasAnyEntries" class="server-instance-extensions-page__state">
      <SLCard variant="outline">
        <div class="server-instance-extensions-page__state-card">
          <strong>{{ i18n.t("servers.next.instance.extensions.loading_title") }}</strong>
          <span>{{ i18n.t("common.loading") }}</span>
        </div>
      </SLCard>
    </section>

    <section v-else-if="isMissingServer" class="server-instance-extensions-page__state">
      <SLCard variant="outline">
        <div class="server-instance-extensions-page__state-card">
          <strong>{{ i18n.t("servers.next.instance.not_found_title") }}</strong>
          <span>{{ i18n.t("servers.next.instance.not_found_description") }}</span>
        </div>
      </SLCard>
    </section>

    <section v-else-if="hasNoDirectories" class="server-instance-extensions-page__state">
      <SLCard variant="outline">
        <div class="server-instance-extensions-page__state-card">
          <strong>{{ i18n.t("servers.next.instance.extensions.no_directories_title") }}</strong>
          <span>{{ i18n.t("servers.next.instance.extensions.no_directories_description") }}</span>
        </div>
      </SLCard>
    </section>

    <section v-else-if="hasEmptyDirectories" class="server-instance-extensions-page__state">
      <SLCard variant="outline">
        <div class="server-instance-extensions-page__state-card">
          <strong>{{ i18n.t("servers.next.instance.extensions.empty_directories_title") }}</strong>
          <span>{{
            i18n.t("servers.next.instance.extensions.empty_directories_description")
          }}</span>
        </div>
      </SLCard>
    </section>

    <div v-else class="server-instance-extensions-page__lists">
      <ExtensionEntryList
        :title="pluginsTitle"
        :description="i18n.t('servers.next.instance.extensions.plugins_description')"
        :entries="summary.plugin_entries"
        :empty-message="i18n.t('servers.next.instance.extensions.plugins_empty')"
        :format-size="page.formatSize"
        :format-modified-at="page.formatModifiedAt"
        :resolve-type-label="page.resolveTypeLabel"
      />

      <ExtensionEntryList
        :title="modsTitle"
        :description="i18n.t('servers.next.instance.extensions.mods_description')"
        :entries="summary.mod_entries"
        :empty-message="i18n.t('servers.next.instance.extensions.mods_empty')"
        :format-size="page.formatSize"
        :format-modified-at="page.formatModifiedAt"
        :resolve-type-label="page.resolveTypeLabel"
      />

      <ExtensionEntryList
        :title="otherTitle"
        :description="i18n.t('servers.next.instance.extensions.other_description')"
        :entries="summary.other_entries"
        :empty-message="i18n.t('servers.next.instance.extensions.other_empty')"
        :format-size="page.formatSize"
        :format-modified-at="page.formatModifiedAt"
        :resolve-type-label="page.resolveTypeLabel"
      />
    </div>
  </div>
</template>

<style scoped>
.server-instance-extensions-page,
.server-instance-extensions-page__lists,
.server-instance-extensions-page__state-card,
.server-instance-extensions-page__error {
  display: grid;
}

.server-instance-extensions-page {
  gap: 18px;
}

.server-instance-extensions-page__lists {
  gap: 22px;
}

.server-instance-extensions-page__error {
  gap: 8px;
  padding: 14px 16px;
  border-radius: 18px;
  border: 1px solid rgba(239, 68, 68, 0.24);
  background: rgba(239, 68, 68, 0.1);
  color: var(--sl-error);
}

.server-instance-extensions-page__state-card {
  gap: 6px;
  padding: 8px 4px;
}

.server-instance-extensions-page__state-card strong {
  color: var(--sl-text-primary);
}

.server-instance-extensions-page__state-card span {
  color: var(--sl-text-secondary);
}
</style>
