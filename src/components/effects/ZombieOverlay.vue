<script setup lang="ts">
// Zombie Theme Overlay - Scanlines, Noise, and Glitches
</script>

<template>
  <div class="zombie-overlay">
    <div class="scanlines"></div>
    <div class="noise"></div>
    <div class="glitch-flash"></div>
    
    <svg style="display: none;">
      <filter id="zombie-noise">
        <feTurbulence type="fractalNoise" baseFrequency="0.8" numOctaves="4" stitchTiles="stitch" />
        <feColorMatrix type="saturate" values="0" />
      </filter>
    </svg>
  </div>
</template>

<style scoped>
.zombie-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  pointer-events: none;
  overflow: hidden;
}

.scanlines {
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    0deg,
    transparent 0px,
    transparent 1px,
    rgba(200, 255, 176, 0.05) 1px,
    rgba(200, 255, 176, 0.05) 2px
  );
  background-size: 100% 4px;
  animation: scanline 10s linear infinite;
}

.noise {
  position: absolute;
  inset: -200%;
  opacity: 0.08;
  filter: url(#zombie-noise);
  animation: noise 0.2s steps(2) infinite;
}

.glitch-flash {
  position: absolute;
  inset: 0;
  background: transparent;
  animation: glitch-flash 12s ease-in-out infinite;
}

@keyframes scanline {
  from { background-position: 0 0; }
  to { background-position: 0 100%; }
}

@keyframes noise {
  0% { transform: translate(0, 0); }
  50% { transform: translate(1%, 1%); }
  100% { transform: translate(-1%, -1%); }
}

@keyframes glitch-flash {
  0%, 94%, 100% { 
    background: transparent; 
    backdrop-filter: none;
    transform: none;
  }
  95% { 
    background: rgba(139, 0, 0, 0.1); 
    backdrop-filter: invert(1) hue-rotate(90deg);
    transform: translateX(5px) skewX(2deg);
  }
  96% { 
    background: rgba(200, 255, 176, 0.1); 
    backdrop-filter: contrast(2) brightness(1.5);
    transform: translateX(-5px) skewX(-2deg);
  }
  97% {
    background: transparent;
    backdrop-filter: sepia(100%) hue-rotate(320deg);
    transform: translateY(3px);
  }
}
</style>
