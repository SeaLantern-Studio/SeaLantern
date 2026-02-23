<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import logo from "@assets/logo.svg";

const props = defineProps<{
  loading?: boolean;
}>();

const emit = defineEmits<{
  ready: [];
}>();

const logoScale = ref(0);
const textOpacity = ref(0);
const animationComplete = ref(false);

onMounted(() => {
  setTimeout(() => {
    logoScale.value = 1;
  }, 50);

  setTimeout(() => {
    textOpacity.value = 1;
  }, 200);

  setTimeout(() => {
    animationComplete.value = true;
    if (!props.loading) {
      emit("ready");
    }
  }, 600);
});

watch(
  () => props.loading,
  (newLoading) => {
    if (!newLoading && animationComplete.value) {
      emit("ready");
    }
  },
);
</script>

<template>
  <div class="splash-screen">
    <div class="splash-content">
      <div class="splash-logo" :style="{ transform: `scale(${logoScale})` }">
        <img :src="logo" alt="Sea Lantern" width="120" height="120" />
      </div>

      <div class="splash-text" :style="{ opacity: textOpacity }">
        <h1 class="splash-title">Sea Lantern</h1>
        <p class="splash-subtitle">Minecraft 服务器管理工具</p>
      </div>

      <div class="splash-loader" :style="{ opacity: textOpacity }">
        <div class="loader-dots">
          <span></span>
          <span></span>
          <span></span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.splash-screen {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: var(--sl-bg, #f8fafc);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.splash-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 24px;
}

.splash-logo {
  transition: transform 0.5s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.splash-text {
  text-align: center;
  transition: opacity 0.4s ease;
}

.splash-title {
  font-size: 2.5rem;
  font-weight: 800;
  color: var(--sl-text-primary);
  margin-bottom: 8px;
  letter-spacing: -0.02em;
}

.splash-subtitle {
  font-size: 1rem;
  color: var(--sl-text-secondary);
  font-weight: 400;
}

.splash-loader {
  transition: opacity 0.4s ease;
  margin-top: 16px;
}

.loader-dots {
  display: flex;
  gap: 8px;
}

.loader-dots span {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--sl-primary);
  animation: bounce 1.4s infinite ease-in-out;
}

.loader-dots span:nth-child(1) {
  animation-delay: -0.32s;
}

.loader-dots span:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes bounce {
  0%,
  80%,
  100% {
    transform: scale(0.8);
    opacity: 0.5;
  }
  40% {
    transform: scale(1.2);
    opacity: 1;
  }
}
</style>
