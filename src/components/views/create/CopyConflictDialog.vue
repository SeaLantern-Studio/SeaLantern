<script setup lang="ts">
import { computed } from "vue";
import { DialogContent, DialogDescription, DialogOverlay, DialogPortal, DialogRoot, DialogTitle } from "reka-ui";
import { AlertTriangle } from "lucide-vue-next";
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";

const props = defineProps<{
  open: boolean;
  items: string[];
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "confirm"): void;
  (e: "cancel"): void;
}>();

const previewItems = computed(() => props.items.slice(0, 8));
const hiddenCount = computed(() => Math.max(0, props.items.length - 8));
</script>

<template>
  <DialogRoot :open="open" @update:open="emit('update:open', $event)">
    <DialogPortal>
      <DialogOverlay class="copy-conflict-overlay" />
      <DialogContent class="copy-conflict-content">
        <DialogTitle class="copy-conflict-title">
          <AlertTriangle :size="16" />
          {{ i18n.t("create.copy_conflict_title") }}
        </DialogTitle>
        <DialogDescription class="copy-conflict-description">
          {{ i18n.t("create.copy_conflict_description") }}
        </DialogDescription>

        <ul class="copy-conflict-list">
          <li v-for="item in previewItems" :key="item">{{ item }}</li>
        </ul>
        <p v-if="hiddenCount > 0" class="copy-conflict-more">
          {{ i18n.t("create.copy_conflict_more", { count: hiddenCount }) }}
        </p>

        <div class="copy-conflict-actions">
          <SLButton variant="secondary" @click="emit('cancel')">
            {{ i18n.t("create.cancel") }}
          </SLButton>
          <SLButton variant="primary" @click="emit('confirm')">
            {{ i18n.t("create.copy_conflict_confirm") }}
          </SLButton>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<style src="@styles/components/views/create/CopyConflictDialog.css" scoped></style>
