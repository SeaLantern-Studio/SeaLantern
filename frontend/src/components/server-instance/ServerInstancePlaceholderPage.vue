<script setup lang="ts">
import { computed } from "vue";
import { RouterLink } from "vue-router";
import { ArrowLeft, ArrowRight } from "@lucide/vue";
import { i18n } from "@language";
import { useNextInstanceWorkspaceContext } from "@src/composables/useNextInstanceWorkspace";
import NextPlaceholderPage from "@src/views/NextPlaceholderPage.vue";

const workspace = useNextInstanceWorkspaceContext();
const placeholderContent = computed(() => workspace.placeholderContent.value);

const actions = computed(() => {
  const serverId = workspace.serverId.value;

  return [
    {
      label: i18n.t("servers.next.instance.sections.players"),
      to: { name: "next-server-instance-players", params: { serverId } },
      hint: i18n.t("servers.next.instance.placeholder.actions.players"),
      tone: "primary" as const,
    },
    {
      label: i18n.t("servers.next.instance.back_to_servers"),
      to: { name: "next-servers" },
      hint: i18n.t("servers.next.instance.placeholder.actions.back"),
    },
  ];
});
</script>

<template>
  <div class="server-instance-placeholder-page">
    <div class="server-instance-placeholder-page__back-row">
      <RouterLink
        class="server-instance-placeholder-page__back-link"
        :to="{ name: 'next-servers' }"
      >
        <ArrowLeft :size="16" />
        <span>{{ i18n.t("servers.next.instance.back_to_servers") }}</span>
      </RouterLink>
    </div>

    <NextPlaceholderPage
      v-if="placeholderContent"
      :eyebrow="placeholderContent.eyebrow"
      :summary="placeholderContent.summary"
      :description="placeholderContent.description"
      :tracks="placeholderContent.tracks"
      :actions="actions"
    />

    <RouterLink
      class="server-instance-placeholder-page__return-link"
      :to="{ name: 'next-server-instance-players', params: { serverId: workspace.serverId.value } }"
    >
      <span>{{ i18n.t("servers.next.instance.placeholder.return_to_players") }}</span>
      <ArrowRight :size="16" />
    </RouterLink>
  </div>
</template>

<style scoped>
.server-instance-placeholder-page {
  display: grid;
  gap: 16px;
}

.server-instance-placeholder-page__back-row {
  display: flex;
  justify-content: flex-start;
}

.server-instance-placeholder-page__back-link,
.server-instance-placeholder-page__return-link {
  width: fit-content;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  color: var(--sl-text-secondary);
  text-decoration: none;
}

.server-instance-placeholder-page__return-link {
  color: var(--sl-text-primary);
  font-weight: 600;
}
</style>
