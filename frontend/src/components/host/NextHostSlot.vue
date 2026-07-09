<script setup lang="ts">
import { computed } from "vue";
import type { NextHostSlotId, NextHostSlotScope } from "@src/contracts/slots";
import { useNextHostRuntime } from "@src/host/runtime";

interface Props {
  slotId: NextHostSlotId;
  scope: NextHostSlotScope;
}

const props = defineProps<Props>();

const nextHostRuntime = useNextHostRuntime();
const registrations = computed(() =>
  nextHostRuntime.slots.registrations.value.filter(
    (entry) => entry.slotId === props.slotId && entry.scope === props.scope,
  ),
);
</script>

<template>
  <div
    class="next-host-slot"
    :class="[`next-host-slot--${scope}`, `next-host-slot--${slotId.replace(/\./g, '-')}`]"
    :data-next-host-slot="slotId"
    :data-next-host-scope="scope"
  >
    <component
      :is="entry.component"
      v-for="entry in registrations"
      :key="entry.registrationId"
      v-bind="entry.props"
    />

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
