<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import { i18n } from "@language";
import { useGeneralSettingsSection } from "@next-src/pages/settings/useGeneralSettingsSection";

const {
  bootstrapping,
  pending,
  state,
  closeActionOptions,
  updateCloseAction,
  updateCloseServersOnExit,
  updateCloseServersOnUpdate,
  updateAutoAcceptEula,
} = useGeneralSettingsSection();
</script>

<template>
  <SLCard variant="outline" padding="lg" class="general-section">
    <p class="general-section__description">{{ i18n.t("settings.next.general.live_hint") }}</p>

    <div class="general-section__list">
      <section class="general-section__item">
        <div class="general-section__copy">
          <span class="general-section__item-title">{{ i18n.t("settings.close_action") }}</span>
          <p class="general-section__item-description">
            {{ i18n.t("settings.close_action_desc") }}
          </p>
        </div>

        <div class="general-section__control general-section__control--select">
          <span
            v-if="pending.closeAction"
            class="general-section__saving-indicator"
            aria-hidden="true"
          />
          <div class="general-section__select-wrap">
            <SLSelect
              :model-value="state.closeAction"
              :options="closeActionOptions"
              :disabled="bootstrapping || pending.closeAction"
              dropdown-width="220px"
              @update:model-value="updateCloseAction"
            />
          </div>
        </div>
      </section>

      <section class="general-section__item">
        <div class="general-section__copy">
          <span class="general-section__item-title">{{ i18n.t("settings.auto_stop") }}</span>
          <p class="general-section__item-description">{{ i18n.t("settings.auto_stop_desc") }}</p>
        </div>

        <div class="general-section__control general-section__control--switch">
          <span
            v-if="pending.closeServersOnExit"
            class="general-section__saving-indicator"
            aria-hidden="true"
          />
          <SLSwitch
            :model-value="state.closeServersOnExit"
            :disabled="bootstrapping || pending.closeServersOnExit"
            @update:model-value="updateCloseServersOnExit"
          />
        </div>
      </section>

      <section class="general-section__item">
        <div class="general-section__copy">
          <span class="general-section__item-title">{{ i18n.t("settings.update_auto_stop") }}</span>
          <p class="general-section__item-description">
            {{ i18n.t("settings.update_auto_stop_desc") }}
          </p>
        </div>

        <div class="general-section__control general-section__control--switch">
          <span
            v-if="pending.closeServersOnUpdate"
            class="general-section__saving-indicator"
            aria-hidden="true"
          />
          <SLSwitch
            :model-value="state.closeServersOnUpdate"
            :disabled="bootstrapping || pending.closeServersOnUpdate"
            @update:model-value="updateCloseServersOnUpdate"
          />
        </div>
      </section>

      <section class="general-section__item">
        <div class="general-section__copy">
          <span class="general-section__item-title">{{ i18n.t("settings.auto_eula") }}</span>
          <p class="general-section__item-description">{{ i18n.t("settings.auto_eula_desc") }}</p>
        </div>

        <div class="general-section__control general-section__control--switch">
          <span
            v-if="pending.autoAcceptEula"
            class="general-section__saving-indicator"
            aria-hidden="true"
          />
          <SLSwitch
            :model-value="state.autoAcceptEula"
            :disabled="bootstrapping || pending.autoAcceptEula"
            @update:model-value="updateAutoAcceptEula"
          />
        </div>
      </section>
    </div>
  </SLCard>
</template>

<style scoped>
.general-section {
  min-width: 0;
  display: grid;
  gap: 12px;
  border-radius: 22px;
  border-color: color-mix(in srgb, var(--sl-border) 72%, transparent);
  background: color-mix(in srgb, var(--sl-surface) 92%, var(--sl-bg));
}

.general-section__description {
  margin: 0;
  font-size: 0.84rem;
  color: var(--sl-text-secondary);
  line-height: 1.45;
}

.general-section__list {
  display: grid;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  border-radius: 18px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 68%, transparent);
}

.general-section__item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  padding: 14px 18px;
  border-top: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
}

.general-section__item:first-child {
  border-top: none;
}

.general-section__copy {
  min-width: 0;
  display: grid;
  gap: 4px;
}

.general-section__item-title,
.general-section__item-description {
  margin: 0;
}

.general-section__item-title {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}

.general-section__item-description {
  color: var(--sl-text-secondary);
  font-size: 0.82rem;
  line-height: 1.45;
}

.general-section__control {
  flex: none;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.general-section__control--select {
  width: min(208px, 100%);
}

.general-section__select-wrap {
  width: 208px;
  max-width: 100%;
}

.general-section__saving-indicator {
  width: 12px;
  height: 12px;
  border-radius: 999px;
  border: 2px solid color-mix(in srgb, var(--sl-primary) 20%, transparent);
  border-top-color: var(--sl-primary);
  animation: general-section-spin 0.8s linear infinite;
}

@keyframes general-section-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 720px) {
  .general-section__item {
    flex-direction: column;
    align-items: flex-start;
  }

  .general-section__control,
  .general-section__control--select,
  .general-section__select-wrap {
    width: 100%;
  }

  .general-section__control--switch {
    width: auto;
  }
}
</style>
