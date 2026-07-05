<script setup lang="ts">
import { computed, onMounted, shallowRef } from "vue";
import { ArrowLeft, Blocks, FileCog, Globe2, MonitorSmartphone, Users } from "@lucide/vue";
import { serverApi } from "@api/server";
import { isBrowserEnv } from "@api/tauri";
import SLButton from "@components/common/SLButton.vue";
import SLConfirmDialog from "@components/common/SLConfirmDialog.vue";
import { i18n } from "@language";
import { AUTH_ROUTE_NAME } from "@router/authRoute";
import { useAuthStore } from "@stores/authStore";
import { useRouter } from "vue-router";
import type { NextInstanceSection } from "@src/contracts/instance";
import type { NextShellNavItem, NextShellRailPinControl } from "@src/contracts/shell";
import { useNextShellNavigationTransition } from "@src/composables/useNextShellNavigationTransition";
import { useProvideNextInstanceWorkspace } from "@src/composables/useNextInstanceWorkspace";
import NextWorkbenchLayout from "@src/layout/NextWorkbenchLayout.vue";
import {
  pickDeleteConfirmationItem,
  shouldUseDeleteConfirmationItem,
} from "@src/utils/serverDeleteConfirmation";

const props = defineProps<{
  section: NextInstanceSection;
}>();

const router = useRouter();
const authStore = useAuthStore();
const workspace = useProvideNextInstanceWorkspace();
const isBrowserMode = isBrowserEnv();
const {
  handlePageTransitionSettled,
  isRailPinned,
  pageTransitionClass,
  railExpanded,
  setFocusWithinRail,
  setPointerInsideRail,
  toggleRailPinned,
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
    id: "instance-console",
    label: i18n.t("common.console"),
    icon: MonitorSmartphone,
    to: workspace.consoleRoute.value,
    active: props.section === "console",
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

const deleteDialogVisible = shallowRef(false);
const deleteSubmitting = shallowRef(false);
const deleteErrorMessage = shallowRef("");
const deleteConfirmationItem = shallowRef("");

const deleteDialogTitle = computed(() => i18n.t("home.delete_server"));
const deleteUsesConfirmationItem = computed(() => {
  const serverName = workspace.server.value?.name ?? "";
  return shouldUseDeleteConfirmationItem(serverName);
});
const deleteDialogMessage = computed(() => {
  const serverName = workspace.server.value?.name ?? "";
  if (!serverName) {
    return "";
  }

  if (deleteUsesConfirmationItem.value) {
    return i18n.t("home.delete_confirm_item_message", {
      item: deleteConfirmationItem.value,
    });
  }

  return i18n.t("home.delete_confirm_message", { server: serverName });
});
const deleteInputPlaceholder = computed(() =>
  deleteUsesConfirmationItem.value
    ? i18n.t("home.delete_input_placeholder_item")
    : i18n.t("home.delete_input_placeholder"),
);
const deleteExpectedInput = computed(() => {
  if (deleteUsesConfirmationItem.value) {
    return deleteConfirmationItem.value;
  }

  return workspace.server.value?.name ?? "";
});

const railPinControl = computed<NextShellRailPinControl>(() => ({
  pinned: isRailPinned.value,
  label: i18n.t(isRailPinned.value ? "shell.rail_unpin" : "shell.rail_pin"),
}));

function openDeleteDialog(): void {
  if (!workspace.server.value) {
    return;
  }

  deleteErrorMessage.value = "";
  deleteConfirmationItem.value = deleteUsesConfirmationItem.value ? pickDeleteConfirmationItem() : "";
  deleteDialogVisible.value = true;
}

function closeDeleteDialog(): void {
  if (deleteSubmitting.value) {
    return;
  }

  deleteDialogVisible.value = false;
  deleteErrorMessage.value = "";
  deleteConfirmationItem.value = "";
}

async function confirmDeleteServer(): Promise<void> {
  const server = workspace.server.value;
  if (!server) {
    return;
  }

  deleteSubmitting.value = true;
  deleteErrorMessage.value = "";

  try {
    await serverApi.deleteServer(server.id);
    workspace.serverStore.setCurrentServer(null);

    try {
      await workspace.serverStore.refreshList();
      workspace.serverStore.setCurrentServer(workspace.serverStore.servers[0]?.id ?? null);
    } catch {
      // The route must still leave the deleted workspace even if the list refresh fails.
    }

    deleteDialogVisible.value = false;
    await workspace.router.replace(workspace.backToServersRoute);
  } catch (error) {
    deleteErrorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    deleteSubmitting.value = false;
  }
}

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
    :rail-pin-control="railPinControl"
    :rail-expanded="railExpanded"
    :page-transition-class="pageTransitionClass"
    @logout="handleLogout"
    @page-transition-settled="handlePageTransitionSettled"
    @rail-focus-within-change="setFocusWithinRail"
    @rail-pointer-inside-change="setPointerInsideRail"
    @toggle-rail-pin="toggleRailPinned"
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

    <template #page-header-actions>
      <SLButton
        v-if="workspace.server.value"
        variant="danger"
        size="sm"
        :loading="deleteSubmitting"
        @click="openDeleteDialog"
      >
        {{ i18n.t("home.delete_server") }}
      </SLButton>
      <SLButton
        v-if="workspace.server.value && props.section !== 'console'"
        variant="secondary"
        size="sm"
        @click="workspace.router.push(workspace.consoleRoute.value)"
      >
        {{ i18n.t("common.console") }}
      </SLButton>
    </template>

    <template #page-content-before>
      <section
        v-if="deleteErrorMessage"
        class="next-instance-workspace-view__error"
        role="alert"
      >
        <strong>{{ i18n.t("servers.next.error_title") }}</strong>
        <span>{{ deleteErrorMessage }}</span>
      </section>
    </template>

    <slot />

    <SLConfirmDialog
      :visible="deleteDialogVisible"
      :title="deleteDialogTitle"
      :message="deleteDialogMessage"
      :confirmText="i18n.t('home.delete_server')"
      :cancelText="i18n.t('common.cancel')"
      :inputPlaceholder="deleteInputPlaceholder"
      :expectedInput="deleteExpectedInput"
      :loading="deleteSubmitting"
      confirmVariant="danger"
      dangerous
      requireInput
      @confirm="confirmDeleteServer"
      @close="closeDeleteDialog"
    />
  </NextWorkbenchLayout>
</template>

<style scoped>
.next-instance-workspace-view__error {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px 16px;
  border-radius: 18px;
  border: 1px solid rgba(239, 68, 68, 0.24);
  background: rgba(239, 68, 68, 0.1);
  color: var(--sl-error);
}

.next-instance-workspace-view__header {
  display: grid;
  gap: 8px;
  max-width: 72ch;
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
  max-width: 62ch;
  word-break: break-word;
}
</style>
