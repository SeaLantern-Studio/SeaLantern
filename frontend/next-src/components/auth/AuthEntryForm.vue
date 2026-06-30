<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import { Eye, EyeOff, KeyRound } from "@lucide/vue";
import { i18n } from "@language";
import SLCheckbox from "@components/common/SLCheckbox.vue";
import SLInput from "@components/common/SLInput.vue";

interface Props {
  token: string;
  rememberBrowser: boolean;
  submitting: boolean;
  canClearSaved: boolean;
  errorMessage: string | null;
  revealCredential: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  "update:token": [value: string];
  "update:rememberBrowser": [value: boolean];
  toggleReveal: [];
  submit: [];
  clearSaved: [];
  paste: [];
}>();

const tokenInputRef = useTemplateRef<InstanceType<typeof SLInput>>("tokenInput");
const tokenType = computed(() => (props.revealCredential ? "text" : "password"));
const revealLabel = computed(() =>
  props.revealCredential ? i18n.t("auth.hide") : i18n.t("auth.show"),
);

defineExpose({
  focusInput: () => tokenInputRef.value?.focus(),
});
</script>

<template>
  <form class="next-auth-form" @submit.prevent="emit('submit')">
    <div class="next-auth-form__field-block">
      <div class="next-auth-form__field-meta">
        <span class="next-auth-form__field-label">{{ i18n.t("auth.token_label") }}</span>
        <p class="next-auth-form__field-help">{{ i18n.t("auth.token_help") }}</p>
      </div>

      <SLInput
        ref="tokenInput"
        :model-value="token"
        :type="tokenType"
        :placeholder="i18n.t('auth.token_placeholder')"
        :disabled="submitting"
        @update:model-value="emit('update:token', $event)"
      >
        <template #prefix>
          <KeyRound :size="16" class="next-auth-form__field-icon" />
        </template>
        <template #suffix>
          <button
            class="next-auth-form__inline-action"
            type="button"
            :disabled="submitting"
            @click="emit('toggleReveal')"
          >
            <component :is="revealCredential ? EyeOff : Eye" :size="14" />
            <span>{{ revealLabel }}</span>
          </button>
        </template>
      </SLInput>

      <div class="next-auth-form__actions-inline">
        <button class="next-auth-form__ghost-action" type="button" @click="emit('paste')">
          {{ i18n.t("auth.paste") }}
        </button>
      </div>
    </div>

    <div class="next-auth-form__remember-block">
      <SLCheckbox
        :model-value="rememberBrowser"
        :disabled="submitting"
        :label="i18n.t('auth.remember')"
        @update:model-value="emit('update:rememberBrowser', $event)"
      />
      <p class="next-auth-form__remember-help">{{ i18n.t("auth.remember_help") }}</p>
    </div>

    <div v-if="errorMessage" class="next-auth-form__error" aria-live="polite">
      <p>{{ errorMessage }}</p>
    </div>

    <div class="next-auth-form__footer">
      <button class="next-auth-form__submit" type="submit" :disabled="submitting">
        {{ i18n.t("auth.submit") }}
      </button>

      <button
        v-if="canClearSaved"
        class="next-auth-form__secondary-action"
        type="button"
        :disabled="submitting"
        @click="emit('clearSaved')"
      >
        {{ i18n.t("auth.clear_saved") }}
      </button>
    </div>
  </form>
</template>

<style scoped>
.next-auth-form {
  display: grid;
  gap: var(--sl-space-lg);
}

.next-auth-form__field-block,
.next-auth-form__remember-block {
  display: grid;
  gap: var(--sl-space-sm);
}

.next-auth-form__field-meta {
  display: grid;
  gap: var(--sl-space-xs);
}

.next-auth-form__field-label {
  font-size: var(--sl-font-size-sm);
  font-weight: 600;
  color: var(--sl-text-primary);
}

.next-auth-form__field-help,
.next-auth-form__remember-help,
.next-auth-form__error p {
  margin: 0;
  font-size: var(--sl-font-size-sm);
  line-height: var(--sl-line-height-relaxed);
}

.next-auth-form__field-help,
.next-auth-form__remember-help {
  color: var(--sl-text-secondary);
}

.next-auth-form__field-icon {
  color: var(--sl-text-tertiary);
}

.next-auth-form__inline-action,
.next-auth-form__ghost-action,
.next-auth-form__secondary-action,
.next-auth-form__submit {
  font: inherit;
}

.next-auth-form__inline-action,
.next-auth-form__ghost-action,
.next-auth-form__secondary-action {
  border: 0;
  background: transparent;
  color: var(--sl-primary);
  cursor: pointer;
}

.next-auth-form__inline-action {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0;
}

.next-auth-form__actions-inline {
  display: flex;
  justify-content: flex-end;
}

.next-auth-form__ghost-action,
.next-auth-form__secondary-action {
  padding: 0;
  font-size: var(--sl-font-size-sm);
}

.next-auth-form__error {
  padding: 12px 14px;
  border: 1px solid var(--sl-error-bg);
  border-radius: var(--sl-radius-md);
  background: color-mix(in srgb, var(--sl-error-bg) 70%, transparent);
}

.next-auth-form__error p {
  color: var(--sl-error);
}

.next-auth-form__footer {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-md);
}

.next-auth-form__submit {
  min-width: 180px;
  min-height: 44px;
  padding: 0 18px;
  border: 0;
  border-radius: var(--sl-radius-full);
  background: var(--sl-primary);
  color: var(--sl-text-inverse);
  cursor: pointer;
}

@media (max-width: 767px) {
  .next-auth-form__footer {
    align-items: stretch;
    flex-direction: column;
  }

  .next-auth-form__submit,
  .next-auth-form__secondary-action {
    width: 100%;
  }

  .next-auth-form__secondary-action {
    text-align: left;
  }
}
</style>
