<script setup lang="ts">
import { ref, onMounted } from "vue";

const emit = defineEmits<{
  ready: [];
}>();

const logoScale = ref(0);
const textOpacity = ref(0);

onMounted(() => {
  // Logo 缩放动画
  setTimeout(() => {
    logoScale.value = 1;
  }, 50);

  // 文字淡入
  setTimeout(() => {
    textOpacity.value = 1;
  }, 200);

  // 启动完成，通知父组件（总共只显示600ms）
  setTimeout(() => {
    emit("ready");
  }, 600);
});
</script>

<template>
  <div class="splash-screen">
    <div class="splash-content">
      <!-- Logo -->
      <div class="splash-logo" :style="{ transform: `scale(${logoScale})` }">
        <img src="../../assets/logo.svg" alt="Sea Lantern" width="120" height="120" />
      </div>

      <!-- 标题 -->
      <div class="splash-text" :style="{ opacity: textOpacity }">
        <h1 class="splash-title">Sea Lantern</h1>
        <p class="splash-subtitle">Minecraft 服务器管理工具</p>
      </div>

      <!-- 加载指示器 -->
      <div class="splash-loader" :style="{ opacity: textOpacity }">
        <div class="loader-dots">
          <span></span>
          <span></span>
          <span></span>
        </div>
      </div>

      <!-- 修复信息 -->
      <div class="splash-fix-info" :style="{ opacity: textOpacity }">
        <span class="fix-icon">🔧</span>
        <span class="fix-text">XOX-zip 已修复重大Bug</span>
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
  background: linear-gradient(135deg, var(--sl-bg) 0%, var(--sl-bg-secondary) 100%);
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

/* Fix Info */
.splash-fix-info {
  position: absolute;
  bottom: 40px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: rgba(34, 197, 94, 0.1);
  border: 1px solid rgba(34, 197, 94, 0.3);
  border-radius: 20px;
  font-size: 0.8125rem;
  color: #22c55e;
  backdrop-filter: blur(10px);
  transition: opacity 0.4s ease;
}

.fix-icon {
  font-size: 1rem;
}

.fix-text {
  font-weight: 500;
}
</style>
