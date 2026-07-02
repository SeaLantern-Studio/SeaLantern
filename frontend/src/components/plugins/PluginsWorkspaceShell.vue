<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import WorkbenchSectionHeader from "@src/components/workbench/WorkbenchSectionHeader.vue";
import WorkbenchSplitView from "@src/components/workbench/WorkbenchSplitView.vue";
import { i18n } from "@language";
import { usePluginsWorkspace } from "@src/pages/plugins/usePluginsWorkspace";

const {
  isConfigRoute,
  activeSectionId,
  sectionItems,
  currentSectionTitle,
  currentSectionDescription,
  selectSection,
  goBack,
} = usePluginsWorkspace();
</script>

<template>
  <WorkbenchSplitView
    :items="sectionItems"
    :active-id="activeSectionId"
    :aria-label="i18n.t('plugins.next.workspace.nav_aria_label')"
    :ariaLabel="i18n.t('plugins.next.workspace.nav_aria_label')"
    @select="selectSection"
  >
    <template #content-header>
      <WorkbenchSectionHeader
        :title="currentSectionTitle"
        :description="currentSectionDescription"
      >
        <template v-if="isConfigRoute" #actions>
          <SLButton variant="ghost" size="sm" @click="goBack">
            &lt; {{ i18n.t("plugins.back") }}
          </SLButton>
        </template>
      </WorkbenchSectionHeader>
    </template>

    <slot />
  </WorkbenchSplitView>
</template>
