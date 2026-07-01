<script setup lang="ts">
import type { NextHostSlotId, NextHostSlotScope } from "@next-src/contracts/slots";

interface Props {
  slotId: NextHostSlotId;
  scope: NextHostSlotScope;
}

defineProps<Props>();
</script>

<template>
  <div
    class="next-host-slot"
    :class="[`next-host-slot--${scope}`, `next-host-slot--${slotId.replace(/\./g, '-')}`]"
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
