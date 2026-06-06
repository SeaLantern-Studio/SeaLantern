<script setup lang="ts">
import { useAboutLinks } from "@composables/useAboutLinks";
import { i18n } from "@language";
import SLButton from "./SLButton.vue";
import SLModal from "./SLModal.vue";

interface Props {
  visible: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "agree"): void;
  (e: "close"): void;
}>();

const { openLink } = useAboutLinks();

function handleAgree() {
  emit("agree");
  emit("close");
}

function handleOpenTerms() {
  openLink("https://docs.ideaflash.cn/zh/privacy");
}
</script>

<template>
  <SLModal
    :visible="visible"
    :title="i18n.t('terms.title')"
    :close-on-overlay="false"
    :show-close-button="false"
    width="500px"
    @close=""
  >
    <div class="terms-content">
      <p>
        <span>{{ i18n.t("terms.message") }}</span
        ><span class="terms-link" @click="handleOpenTerms">
          {{ i18n.t("terms.link") }}
        </span>
      </p>
    </div>
    <template #footer>
      <div class="terms-actions">
        <SLButton variant="primary" @click="handleAgree" style="width: 100%">
          {{ i18n.t("terms.agree") }}
        </SLButton>
      </div>
    </template>
  </SLModal>
</template>

<style scoped>
.terms-content {
  margin-bottom: var(--sl-space-md);
}

.terms-content p {
  font-size: 0.875rem;
  line-height: 1.6;
  color: var(--sl-text-secondary);
  margin-bottom: var(--sl-space-md);
}

.terms-link {
  color: var(--sl-primary);
  cursor: pointer;
  font-size: 0.875rem;
}

.terms-link:hover {
  color: var(--sl-primary-hover);
}

.terms-actions {
  display: flex;
  gap: var(--sl-space-sm);
}
</style>
