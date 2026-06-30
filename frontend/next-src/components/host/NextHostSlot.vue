<script setup lang="ts">
import type { NextHostSlotId } from "../../host/slotIds";

interface Props {
  slotId: NextHostSlotId;
  scope: "shell" | "page" | "overlay";
}

defineProps<Props>();
</script>

<template>
  <div
    class="next-host-slot"
    :class="[
      `next-host-slot--${scope}`,
      `next-host-slot--${slotId.replace(/\./g, '-')}`,
    ]"
    :data-next-host-slot="slotId"
    :data-next-host-scope="scope"
  >
    <slot />
  </div>
</template>

<style scoped>
.next-host-slot {
  min-width: 0;
}

.next-host-slot--overlay {
  position: relative;
  z-index: 8;
}
</style>
