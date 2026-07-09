<script setup lang="ts">
import SLBadge from "@components/common/SLBadge.vue";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLPermissionDialog from "@components/plugin/SLPermissionDialog.vue";
import WorkbenchStatusBanner from "@src/components/workbench/WorkbenchStatusBanner.vue";
import PluginDependentSettingsList from "@src/components/plugins/PluginDependentSettingsList.vue";
import PluginPresetPickerCard from "@src/components/plugins/PluginPresetPickerCard.vue";
import PluginSettingsFormCard from "@src/components/plugins/PluginSettingsFormCard.vue";
import PluginMarketDetailContent from "@components/plugin/market/PluginMarketDetailContent.vue";
import { i18n } from "@language";
import { Layers } from "@lucide/vue";
import { usePluginDetailPage } from "./usePluginDetailPage";

const {
  bootstrapping,
  marketLoading,
  errorMessage,
  installFeedback,
  installing,
  installedPlugin,
  marketPlugin,
  marketDetail,
  displayName,
  displayDescription,
  authorName,
  versionLabel,
  categoryTags,
  iconUrl,
  stateLabel,
  stateTone,
  updateSummary,
  runtimeScene,
  showRuntimeBanner,
  supportsSettingsOnDetail,
  canOpenCategoryPage,
  showToggleControl,
  toggleUnavailableMessage,
  showMarketInstallAction,
  marketActionLabel,
  notFound,
  permissionDialogOpen,
  pendingPermissionPluginName,
  pendingPermissionList,
  permissionDialogTitle,
  permissionDialogMessage,
  clearFeedback,
  loadPage,
  installOrUpdatePlugin,
  toggleInstalledPlugin,
  closePermissionDialog,
  confirmEnablePlugin,
  openCategoryPage,
  openRepository,
  pageSettings,
  updateMainSettingsField,
  updateDependentField,
  resolveLocalizedMarketField,
} = usePluginDetailPage();

const {
  pluginPresets,
  isThemeProvider,
  settingsForm,
  dependentSettingsForms,
  dependentPlugins,
  getFieldStringValue,
  getFieldSelectValue,
  getFieldOptions,
  applyPreset,
  resetToDefault,
  saveSettings,
  saving,
} = pageSettings;

function getPermissionLevel(perm: string): "critical" | "dangerous" | "normal" {
  if (perm === "execute_program" || perm === "plugin_folder_access") {
    return "critical";
  }

  return ["fs", "network", "server", "console"].includes(perm) ? "dangerous" : "normal";
}

function getPermissionLabel(perm: string): string {
  return i18n.t(`plugins.permission.${perm}`) !== `plugins.permission.${perm}`
    ? i18n.t(`plugins.permission.${perm}`)
    : perm;
}

function getPermissionDesc(perm: string): string {
  return i18n.t(`plugins.permission.${perm}_desc`) !== `plugins.permission.${perm}_desc`
    ? i18n.t(`plugins.permission.${perm}_desc`)
    : "";
}
</script>

<template>
  <div class="plugin-detail-page">
    <section
      v-if="installFeedback"
      :class="[
        'plugin-detail-page__feedback',
        `plugin-detail-page__feedback--${installFeedback.type}`,
      ]"
    >
      <span>{{ installFeedback.message }}</span>
      <button class="plugin-detail-page__feedback-close" @click="clearFeedback">×</button>
    </section>

    <section v-if="errorMessage && !notFound" class="plugin-detail-page__error-banner" role="alert">
      <div>
        <strong>{{ i18n.t("plugins.next.error_title") }}</strong>
        <p>{{ errorMessage }}</p>
      </div>
      <SLButton
        variant="secondary"
        size="sm"
        :loading="bootstrapping"
        @click="() => void loadPage()"
      >
        {{ i18n.t("plugins.next.retry") }}
      </SLButton>
    </section>

    <SLCard v-if="bootstrapping" variant="outline">
      <div class="plugin-detail-page__loading">
        <div class="plugin-detail-page__spinner"></div>
        <span>{{ i18n.t("common.loading") }}</span>
      </div>
    </SLCard>

    <SLCard v-else-if="notFound" variant="outline">
      <div class="plugin-detail-page__empty">
        <h2>{{ i18n.t("plugins.not_found") }}</h2>
        <p>{{ i18n.t("market.no_plugins") }}</p>
      </div>
    </SLCard>

    <template v-else>
      <WorkbenchStatusBanner v-if="showRuntimeBanner" tone="info">
        <strong>{{ runtimeScene.bannerTitle }}</strong>
        <span>{{ runtimeScene.bannerDescription }}</span>
      </WorkbenchStatusBanner>

      <SLCard class="plugin-detail-page__hero" variant="outline">
        <div class="plugin-detail-page__hero-main">
          <div class="plugin-detail-page__icon-shell">
            <img
              v-if="iconUrl"
              :src="iconUrl"
              :alt="displayName"
              class="plugin-detail-page__icon"
            />
            <Layers v-else :size="28" class="plugin-detail-page__icon-fallback" />
          </div>

          <div class="plugin-detail-page__copy">
            <div class="plugin-detail-page__title-row">
              <h2 class="plugin-detail-page__title">{{ displayName }}</h2>
              <span v-if="versionLabel" class="plugin-detail-page__version"
                >v{{ versionLabel }}</span
              >
            </div>

            <p v-if="authorName" class="plugin-detail-page__subtitle">
              {{ i18n.t("common.by_author", { author: authorName }) }}
            </p>
            <p v-if="displayDescription" class="plugin-detail-page__description">
              {{ displayDescription }}
            </p>

            <div class="plugin-detail-page__badges">
              <SLBadge
                v-if="stateLabel && stateTone"
                :text="stateLabel"
                :variant="stateTone"
                size="small"
                rounded="full"
              />
              <SLBadge
                v-if="updateSummary"
                :text="updateSummary"
                variant="info"
                size="small"
                rounded="full"
              />
              <span v-for="tag in categoryTags" :key="tag" class="plugin-detail-page__tag">
                {{ tag }}
              </span>
              <span
                v-if="
                  runtimeScene.tagLabel && installedPlugin && !installedPlugin.actions.can_toggle
                "
                class="plugin-detail-page__tag plugin-detail-page__tag--info"
              >
                {{ runtimeScene.tagLabel }}
              </span>
            </div>
          </div>
        </div>

        <div class="plugin-detail-page__hero-actions">
          <SLButton
            v-if="showMarketInstallAction"
            variant="primary"
            size="sm"
            :loading="installing"
            @click="installOrUpdatePlugin"
          >
            {{ marketActionLabel }}
          </SLButton>
          <SLButton
            v-if="installedPlugin && canOpenCategoryPage"
            variant="secondary"
            size="sm"
            @click="openCategoryPage"
          >
            {{ i18n.t("plugins.plugin_category") }}
          </SLButton>
          <SLButton
            v-if="
              installedPlugin?.manifest.repository ||
              marketPlugin?.repo ||
              marketDetail?.author?.url
            "
            variant="ghost"
            size="sm"
            @click="openRepository"
          >
            {{ i18n.t("plugins.open_repository") }}
          </SLButton>
          <div v-if="showToggleControl" class="plugin-detail-page__toggle-row">
            <span class="plugin-detail-page__toggle-label">{{
              i18n.t("plugins.status.enabled")
            }}</span>
            <SLSwitch
              :modelValue="Boolean(installedPlugin && installedPlugin.state === 'enabled')"
              size="sm"
              @update:modelValue="toggleInstalledPlugin(Boolean($event))"
            />
          </div>
          <div
            v-else-if="installedPlugin && toggleUnavailableMessage"
            class="plugin-detail-page__toggle-note"
          >
            {{ toggleUnavailableMessage }}
          </div>
        </div>
      </SLCard>

      <SLCard v-if="marketPlugin" class="plugin-detail-page__section" variant="outline">
        <div class="plugin-detail-page__section-header">
          <h3>{{ i18n.t("market.title") }}</h3>
          <span v-if="marketLoading">{{ i18n.t("market.loading") }}</span>
        </div>
        <PluginMarketDetailContent
          :plugin="marketPlugin"
          :plugin-detail="marketDetail"
          :resolve-i18n="(value) => resolveLocalizedMarketField(value, i18n.getLocale())"
          :get-permission-level="getPermissionLevel"
          :get-permission-label="getPermissionLabel"
          :get-permission-desc="getPermissionDesc"
        />
      </SLCard>

      <PluginPresetPickerCard
        v-if="supportsSettingsOnDetail && isThemeProvider && pluginPresets"
        :title="i18n.t('plugins.preset_theme')"
        :presets="pluginPresets"
        :selected-preset="settingsForm['preset']"
        @select="applyPreset"
      />

      <PluginSettingsFormCard
        v-if="supportsSettingsOnDetail && installedPlugin?.manifest.settings?.length"
        :title="i18n.t('plugins.plugin_settings')"
        :fields="installedPlugin.manifest.settings"
        :form="settingsForm"
        :get-field-string-value="getFieldStringValue"
        :get-field-select-value="getFieldSelectValue"
        :get-field-options="getFieldOptions"
        @update-field="updateMainSettingsField"
      />

      <PluginDependentSettingsList
        v-if="supportsSettingsOnDetail"
        :plugins="dependentPlugins"
        :forms="dependentSettingsForms"
        :item-description="i18n.t('plugins.depends_on', { name: displayName })"
        :get-field-string-value="getFieldStringValue"
        :get-field-select-value="getFieldSelectValue"
        :get-field-options="getFieldOptions"
        @update-field="updateDependentField"
      />

      <div
        v-if="supportsSettingsOnDetail && installedPlugin"
        class="plugin-detail-page__footer-actions"
      >
        <SLButton variant="secondary" size="sm" @click="resetToDefault">
          {{ i18n.t("plugins.reset_default") }}
        </SLButton>
        <SLButton variant="primary" size="sm" :loading="saving" @click="saveSettings">
          {{ i18n.t("plugins.save_settings") }}
        </SLButton>
      </div>
    </template>

    <SLPermissionDialog
      :show="permissionDialogOpen"
      :plugin-name="pendingPermissionPluginName"
      :permissions="pendingPermissionList"
      :title="permissionDialogTitle"
      :message="permissionDialogMessage"
      @confirm="confirmEnablePlugin"
      @cancel="closePermissionDialog"
    />
  </div>
</template>

<style scoped>
.plugin-detail-page {
  min-width: 0;
  display: grid;
  gap: 16px;
}

.plugin-detail-page__feedback,
.plugin-detail-page__error-banner {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 16px;
  border-radius: 18px;
}

.plugin-detail-page__feedback {
  border: 1px solid transparent;
  white-space: pre-wrap;
}

.plugin-detail-page__feedback--success {
  color: #2e7d32;
  background: rgba(46, 125, 50, 0.12);
  border-color: rgba(46, 125, 50, 0.24);
}

.plugin-detail-page__feedback--warning {
  color: #8a6d00;
  background: rgba(245, 158, 11, 0.14);
  border-color: rgba(245, 158, 11, 0.28);
}

.plugin-detail-page__feedback--error,
.plugin-detail-page__error-banner {
  color: var(--sl-error);
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.22);
}

.plugin-detail-page__feedback-close {
  border: none;
  background: transparent;
  color: inherit;
  font-size: 1rem;
  cursor: pointer;
}

.plugin-detail-page__error-banner p {
  margin: 4px 0 0;
}

.plugin-detail-page__loading,
.plugin-detail-page__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  min-height: 180px;
}

.plugin-detail-page__spinner {
  width: 24px;
  height: 24px;
  border-radius: 999px;
  border: 3px solid color-mix(in srgb, var(--sl-primary) 18%, transparent);
  border-top-color: var(--sl-primary);
  animation: plugin-detail-page-spin 0.9s linear infinite;
}

.plugin-detail-page__hero {
  display: grid;
  gap: 16px;
}

.plugin-detail-page__hero-main {
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

.plugin-detail-page__icon-shell {
  width: 64px;
  height: 64px;
  border-radius: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--sl-border) 78%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 88%, transparent);
}

.plugin-detail-page__icon {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.plugin-detail-page__icon-fallback {
  color: var(--sl-text-tertiary);
}

.plugin-detail-page__copy {
  min-width: 0;
  display: grid;
  gap: 10px;
  flex: 1;
}

.plugin-detail-page__title-row,
.plugin-detail-page__badges,
.plugin-detail-page__hero-actions,
.plugin-detail-page__toggle-row,
.plugin-detail-page__footer-actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  align-items: center;
}

.plugin-detail-page__title {
  margin: 0;
  font-size: 1.1rem;
  color: var(--sl-text-primary);
}

.plugin-detail-page__version,
.plugin-detail-page__subtitle,
.plugin-detail-page__toggle-label {
  font-size: 0.85rem;
  color: var(--sl-text-secondary);
}

.plugin-detail-page__subtitle,
.plugin-detail-page__description {
  margin: 0;
  line-height: 1.6;
}

.plugin-detail-page__description {
  color: var(--sl-text-secondary);
}

.plugin-detail-page__tag {
  padding: 4px 9px;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 74%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 92%, transparent);
  color: var(--sl-text-secondary);
  font-size: 0.74rem;
  line-height: 1;
}

.plugin-detail-page__tag--info {
  border-color: color-mix(in srgb, var(--sl-primary) 26%, transparent);
  color: var(--sl-primary);
}

.plugin-detail-page__toggle-note {
  font-size: 0.85rem;
  color: var(--sl-text-secondary);
  line-height: 1.5;
}

.plugin-detail-page__section {
  display: grid;
  gap: 12px;
}

.plugin-detail-page__section-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
}

.plugin-detail-page__section-header h3 {
  margin: 0;
  color: var(--sl-text-primary);
}

.plugin-detail-page__footer-actions {
  justify-content: flex-end;
}

@keyframes plugin-detail-page-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 720px) {
  .plugin-detail-page__hero-main,
  .plugin-detail-page__feedback,
  .plugin-detail-page__error-banner {
    flex-direction: column;
  }
}
</style>
