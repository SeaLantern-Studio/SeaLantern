<script setup lang="ts">
import { computed, onMounted } from "vue";
import { ArrowLeft, Blocks, FileCog, Globe2, Users } from "@lucide/vue";
import { isBrowserEnv } from "@api/tauri";
import { i18n } from "@language";
import { AUTH_ROUTE_NAME } from "@router/authRoute";
import { useAuthStore } from "@stores/authStore";
import { useRouter } from "vue-router";
import type { NextInstanceSection } from "@src/contracts/instance";
import type { NextShellNavItem } from "@src/contracts/shell";
import { useNextShellNavigationTransition } from "@src/composables/useNextShellNavigationTransition";
import { useProvideNextInstanceWorkspace } from "@src/composables/useNextInstanceWorkspace";
import NextWorkbenchLayout from "@src/layout/NextWorkbenchLayout.vue";

const props = defineProps<{
  section: NextInstanceSection;
}>();

const router = useRouter();
const authStore = useAuthStore();
const workspace = useProvideNextInstanceWorkspace();
const isBrowserMode = isBrowserEnv();
const {
  handlePageTransitionSettled,
  pageTransitionClass,
  railExpanded,
  setFocusWithinRail,
  setPointerInsideRail,
} = useNextShellNavigationTransition("servers");

const brand = computed(
  () => workspace.server.value?.name ?? i18n.t("servers.next.instance.brand_fallback"),
);

const navItems = computed<NextShellNavItem[]>(() => [
  {
    id: "back-to-servers",
    label: i18n.t("servers.next.instance.back_to_servers"),
    icon: ArrowLeft,
    to: workspace.backToServersRoute,
    active: false,
  },
  {
    id: "instance-players",
    label: i18n.t("servers.next.instance.sections.players"),
    icon: Users,
    to: workspace.playersRoute.value,
    active: props.section === "players",
  },
  {
    id: "instance-extensions",
    label: i18n.t("servers.next.instance.sections.extensions"),
    icon: Blocks,
    to: workspace.extensionsRoute.value,
    active: props.section === "extensions",
  },
  {
    id: "instance-config",
    label: i18n.t("servers.next.instance.sections.config"),
    icon: FileCog,
    to: workspace.configRoute.value,
    active: props.section === "config",
  },
  {
    id: "instance-world",
    label: i18n.t("servers.next.instance.sections.world"),
    icon: Globe2,
    to: workspace.worldRoute.value,
    active: props.section === "world",
  },
]);

const layoutPage = computed(() => ({
  kind: "servers" as const,
  title: workspace.pageTitle.value,
  subtitle: workspace.pageSubtitle.value,
}));

async function handleLogout(): Promise<void> {
  if (!isBrowserMode) {
    await router.replace({ path: "/" });
    return;
  }

  authStore.logout();
  await router.replace({ name: AUTH_ROUTE_NAME });
}

onMounted(() => {
  if (workspace.serverId.value) {
    workspace.serverStore.setCurrentServer(workspace.serverId.value);
  }
});
</script>

<template>
  <NextWorkbenchLayout
    :brand="brand"
    :rail-label="workspace.statusLabel.value"
    :page="layoutPage"
    :logout-label="i18n.t('shell.logout')"
    :show-logout="isBrowserMode"
    :nav-items="navItems"
    :rail-expanded="railExpanded"
    :page-transition-class="pageTransitionClass"
    @logout="handleLogout"
    @page-transition-settled="handlePageTransitionSettled"
    @rail-focus-within-change="setFocusWithinRail"
    @rail-pointer-inside-change="setPointerInsideRail"
  >
    <template #page-header>
      <div class="next-instance-workspace-view__header">
        <span class="next-instance-workspace-view__eyebrow">{{
          i18n.t("servers.next.instance.eyebrow")
        }}</span>
        <h2>
          {{ workspace.server.value?.name ?? i18n.t("servers.next.instance.not_found_title") }}
        </h2>
        <p>
          {{ workspace.statusLabel.value }}
          <template v-if="workspace.server.value">· {{ workspace.server.value.path }}</template>
        </p>
      </div>
    </template>

    <slot />
  </NextWorkbenchLayout>
</template>

<style scoped>
.next-instance-workspace-view__header {
  display: grid;
  gap: 6px;
}

.next-instance-workspace-view__eyebrow {
  font-size: var(--sl-font-size-xs);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--sl-text-tertiary);
}

.next-instance-workspace-view__header h2,
.next-instance-workspace-view__header p {
  margin: 0;
}

.next-instance-workspace-view__header h2 {
  color: var(--sl-text-primary);
  font-size: clamp(1.2rem, 2vw, 1.6rem);
}

.next-instance-workspace-view__header p {
  color: var(--sl-text-secondary);
  line-height: 1.5;
  word-break: break-word;
}
</style>
