<script setup lang="ts">
import { AvatarRoot, AvatarImage, AvatarFallback } from "reka-ui";
import tauriIcon64 from "@src-tauri/icons/64x64.png";

defineProps<{
  name: string;
  size?: number;
}>();
</script>

<template>
  <AvatarRoot
    class="player-avatar-root"
    :style="{ width: `${size || 36}px`, height: `${size || 36}px` }"
  >
    <AvatarImage
      :src="`https://api.rms.net.cn/head/${name}`"
      :alt="name"
      class="player-avatar-img"
    />
    <AvatarFallback class="player-avatar-fallback">
      <AvatarRoot class="player-avatar-root">
        <AvatarImage src="https://api.rms.net.cn/head/Alex" alt="Alex" class="player-avatar-img" />
        <AvatarFallback class="player-avatar-fallback">
          <img :src="tauriIcon64" alt="default" class="player-avatar-img" />
        </AvatarFallback>
      </AvatarRoot>
    </AvatarFallback>
  </AvatarRoot>
</template>

<style scoped>
.player-avatar-root {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  vertical-align: middle;
  overflow: hidden;
  user-select: none;
  border-radius: var(--sl-radius-sm);
  background-color: var(--sl-surface);
  flex-shrink: 0;
}

.player-avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.player-avatar-fallback {
  display: flex;
  width: 100%;
  height: 100%;
  align-items: center;
  justify-content: center;
  background-color: var(--sl-primary-bg);
  color: var(--sl-primary);
  font-size: 0.75rem;
  font-weight: 600;
}
</style>
