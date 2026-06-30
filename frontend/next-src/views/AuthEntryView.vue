<script setup lang="ts">
import { computed, onMounted, shallowRef, useTemplateRef } from "vue";
import { useRoute, useRouter } from "vue-router";
import { ShieldCheck } from "@lucide/vue";
import { i18n } from "@language";
import AuthEntryForm from "../components/auth/AuthEntryForm.vue";
import { sanitizeRedirectPath } from "@router/authRoute";
import { resolveErrorMessage } from "@utils/appError";
import { useAuthStore } from "@stores/authStore";

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();
const token = shallowRef("");
const rememberBrowser = shallowRef(false);
const revealCredential = shallowRef(false);
const formRef = useTemplateRef<InstanceType<typeof AuthEntryForm>>("formRef");

const redirectTarget = computed(() => sanitizeRedirectPath(route.query.redirect));
const instanceHost = computed(() => window.location.host);
const errorMessage = computed(() =>
  authStore.lastErrorCode ? resolveErrorMessage(authStore.lastErrorCode) : null,
);
const isSubmitting = computed(() => authStore.isSubmitting);

async function handleSubmit(): Promise<void> {
  const success = await authStore.login(token.value, rememberBrowser.value);
  if (!success) {
    formRef.value?.focusInput();
    return;
  }

  void router.replace(redirectTarget.value);
}

async function handlePaste(): Promise<void> {
  try {
    token.value = await navigator.clipboard.readText();
  } catch {
    // clipboard access may be blocked in some browsers
  }
}

onMounted(() => {
  formRef.value?.focusInput();
});
</script>

<template>
  <main class="next-auth-page">
    <section class="next-auth-page__panel">
      <div class="next-auth-page__hero">
        <div class="next-auth-page__mark">
          <ShieldCheck :size="22" />
        </div>

        <div class="next-auth-page__copy">
          <span class="next-auth-page__eyebrow">{{ i18n.t("auth.status_headless") }}</span>
          <h1>{{ i18n.t("auth.heading") }}</h1>
          <p>{{ i18n.t("auth.description") }}</p>
          <div class="next-auth-page__instance">
            <span>{{ i18n.t("auth.identity_label") }}</span>
            <strong>{{ instanceHost }}</strong>
          </div>
        </div>
      </div>

      <div class="next-auth-page__form-card">
        <AuthEntryForm
          ref="formRef"
          :token="token"
          :remember-browser="rememberBrowser"
          :submitting="isSubmitting"
          :can-clear-saved="authStore.hasSavedCredential"
          :error-message="errorMessage"
          :reveal-credential="revealCredential"
          @update:token="token = $event"
          @update:remember-browser="rememberBrowser = $event"
          @toggle-reveal="revealCredential = !revealCredential"
          @submit="handleSubmit"
          @clear-saved="authStore.clearSavedTokens()"
          @paste="handlePaste"
        />
      </div>
    </section>
  </main>
</template>

<style scoped>
.next-auth-page {
  min-height: 100vh;
  padding: clamp(16px, 4vw, 40px);
  display: grid;
  place-items: center;
  background:
    radial-gradient(circle at top left, color-mix(in srgb, var(--sl-primary) 12%, transparent), transparent 26%),
    linear-gradient(180deg, color-mix(in srgb, var(--sl-surface) 84%, transparent), var(--sl-bg));
}

.next-auth-page__panel {
  width: min(960px, 100%);
  display: grid;
  grid-template-columns: minmax(0, 0.95fr) minmax(0, 1.05fr);
  gap: clamp(20px, 4vw, 40px);
  padding: clamp(20px, 4vw, 36px);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-xl);
  background: color-mix(in srgb, var(--sl-surface) 92%, transparent);
  box-shadow: var(--sl-shadow-lg);
}

.next-auth-page__hero,
.next-auth-page__copy {
  display: grid;
}

.next-auth-page__hero {
  align-content: start;
  gap: var(--sl-space-lg);
}

.next-auth-page__copy {
  gap: var(--sl-space-md);
}

.next-auth-page__mark {
  width: 52px;
  height: 52px;
  display: grid;
  place-items: center;
  border-radius: 18px;
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
}

.next-auth-page__eyebrow {
  font-size: var(--sl-font-size-xs);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--sl-text-tertiary);
}

.next-auth-page__copy h1,
.next-auth-page__copy p {
  margin: 0;
}

.next-auth-page__copy h1 {
  font-size: clamp(2rem, 4vw, 3.4rem);
  line-height: 0.95;
  letter-spacing: -0.04em;
}

.next-auth-page__copy p {
  max-width: 34ch;
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-lg);
  line-height: var(--sl-line-height-relaxed);
}

.next-auth-page__instance {
  display: inline-flex;
  flex-wrap: wrap;
  gap: var(--sl-space-sm);
  width: fit-content;
  padding: 10px 12px;
  border-radius: var(--sl-radius-full);
  background: color-mix(in srgb, var(--sl-bg-secondary) 72%, transparent);
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-sm);
}

.next-auth-page__instance strong {
  color: var(--sl-text-primary);
}

.next-auth-page__form-card {
  display: grid;
  align-content: center;
}

@media (max-width: 1023px) {
  .next-auth-page__panel {
    grid-template-columns: 1fr;
  }

  .next-auth-page__copy p {
    max-width: 48ch;
  }
}

@media (max-width: 767px) {
  .next-auth-page {
    align-items: stretch;
    padding: 0;
    background: var(--sl-bg);
  }

  .next-auth-page__panel {
    min-height: 100vh;
    border: 0;
    border-radius: 0;
    box-shadow: none;
    padding: 24px 18px 32px;
  }

  .next-auth-page__copy h1 {
    font-size: 2.4rem;
  }

  .next-auth-page__copy p {
    font-size: var(--sl-font-size-base);
  }
}
</style>
