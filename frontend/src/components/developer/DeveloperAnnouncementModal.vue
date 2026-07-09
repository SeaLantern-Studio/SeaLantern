<script setup lang="ts">
import { computed } from "vue";
import SLButton from "@components/common/SLButton.vue";
import SLModal from "@components/common/SLModal.vue";
import { i18n } from "@language";
import type { DeveloperAnnouncementState } from "@composables/useDeveloperTools";

const props = defineProps<{
  announcement: DeveloperAnnouncementState | null;
}>();

const emit = defineEmits<{
  close: [];
}>();

const visible = computed(() => props.announcement !== null);
const toneLabel = computed(() => {
  const tone = props.announcement?.tone;
  if (tone === "error") {
    return i18n.t("developer.next.tools.banner.error_button");
  }
  if (tone === "warning") {
    return i18n.t("developer.next.tools.banner.warning_button");
  }
  return i18n.t("developer.next.tools.banner.info_button");
});

const modalTitle = computed(() => props.announcement?.title ?? "");
</script>

<template>
  <SLModal :visible="visible" :title="modalTitle" width="560px" @close="emit('close')">
    <div v-if="announcement" class="developer-announcement-modal">
      <span
        class="developer-announcement-modal__tone"
        :class="`developer-announcement-modal__tone--${announcement.tone}`"
      >
        {{ toneLabel }}
      </span>

      <p class="developer-announcement-modal__message">
        {{ announcement.message }}
      </p>
    </div>

    <template #footer>
      <SLButton variant="primary" size="sm" @click="emit('close')">
        {{ i18n.t("developer.next.tools.banner.clear_button") }}
      </SLButton>
    </template>
  </SLModal>
</template>

<style scoped>
.developer-announcement-modal {
  display: grid;
  gap: 14px;
}

.developer-announcement-modal__tone {
  width: fit-content;
  max-width: 100%;
  padding: 6px 12px;
  border-radius: 999px;
  font-size: 0.78rem;
  font-weight: 700;
  line-height: 1.2;
}

.developer-announcement-modal__tone--info {
  color: color-mix(in srgb, var(--sl-primary) 82%, black);
  background: color-mix(in srgb, var(--sl-primary) 12%, var(--sl-surface));
}

.developer-announcement-modal__tone--warning {
  color: color-mix(in srgb, var(--sl-warning) 72%, black);
  background: color-mix(in srgb, var(--sl-warning) 15%, var(--sl-surface));
}

.developer-announcement-modal__tone--error {
  color: var(--sl-error);
  background: rgba(239, 68, 68, 0.12);
}

.developer-announcement-modal__message {
  margin: 0;
  font-size: 0.96rem;
  line-height: 1.75;
  color: var(--sl-text-primary);
  white-space: pre-wrap;
}
</style>
