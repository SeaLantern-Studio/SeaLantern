<template>
  <div class="w-full">
    <div class="flex items-center justify-between py-1">
      <!-- Left Side: Label & Desc -->
      <div class="flex flex-col gap-1 min-w-0 pr-4">
        <span class="text-[0.9375rem] font-medium text-[var(--sl-text-primary)]">
          {{ i18n.t('settings.java_download') }}
        </span>
        <span class="text-[0.8125rem] text-[var(--sl-text-tertiary)] leading-snug">
          {{ i18n.t('settings.java_download_desc') }}
        </span>
      </div>

      <!-- Right Side: Interaction Area -->
      <div class="flex items-center gap-3 flex-shrink-0">
        <!-- Idle State -->
        <template v-if="!isDownloading && !isExtracting && !successMessage">
          <div class="w-36">
            <SLSelect 
              v-model="selectedVersion" 
              :options="versionOptions" 
              :disabled="loadingUrl"
              size="sm"
            />
          </div>
          <SLButton 
            variant="primary" 
            size="sm"
            :loading="loadingUrl" 
            @click="startDownload"
          >
            {{ i18n.t('settings.java_download_btn') }}
          </SLButton>
        </template>

        <!-- Downloading State -->
        <template v-else-if="isDownloading || isExtracting">
          <div class="flex items-center gap-3">
             <div class="flex flex-col items-end gap-1 w-40">
               <div class="flex items-center gap-2 text-xs text-[var(--sl-text-primary)]">
                 <span>{{ statusMessage }}</span>
                 <span class="font-mono opacity-70">{{ isExtracting ? '' : `${progress.toFixed(0)}%` }}</span>
               </div>
               <div class="w-full h-1.5 bg-[var(--sl-border)] rounded-full overflow-hidden">
                  <div 
                    class="h-full bg-[var(--sl-primary)] transition-all duration-300 ease-out"
                    :class="{ 'indeterminate-progress': isExtracting }"
                    :style="{ width: isExtracting ? '100%' : `${progress}%` }"
                  ></div>
               </div>
             </div>
             <SLButton 
               size="sm" 
               variant="ghost" 
               class="!p-1.5 text-[var(--sl-text-tertiary)] hover:text-[var(--sl-error)]"
               title="Cancel"
               @click="cancelDownload"
             >
               <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                 <line x1="18" y1="6" x2="6" y2="18"></line>
                 <line x1="6" y1="6" x2="18" y2="18"></line>
               </svg>
             </SLButton>
          </div>
        </template>

        <!-- Success State -->
        <template v-else-if="successMessage">
          <div class="flex items-center gap-3 animate-fade-in">
             <div class="flex items-center gap-1.5 text-[var(--sl-success)] text-sm font-medium">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
                </svg>
                <span>{{ i18n.t('settings.java_install_success').replace(':', '') }}</span>
             </div>
             <SLButton size="sm" variant="ghost" @click="resetState">OK</SLButton>
          </div>
        </template>
      </div>
    </div>

    <!-- Error Message (Full Width below) -->
    <div v-if="errorMessage" class="mt-2 p-3 bg-red-50 dark:bg-red-900/20 text-[var(--sl-error)] text-sm rounded border border-red-200 dark:border-red-800 flex items-center justify-between animate-fade-in">
      <div class="flex items-center gap-2">
        <svg class="flex-shrink-0" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
        </svg>
        <span>{{ errorMessage }}</span>
      </div>
      <SLButton size="sm" variant="ghost" @click="resetState">
        {{ i18n.t('common.close_notification') }}
      </SLButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue';
import { i18n } from '../locales';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { javaApi } from '../api/java';
import SLButton from './common/SLButton.vue';
import SLSelect from './common/SLSelect.vue';

const emit = defineEmits(['installed']);

const selectedVersion = ref('17');
const isDownloading = ref(false);
const isExtracting = ref(false);
const loadingUrl = ref(false);
const progress = ref(0);
const statusMessage = ref('');
const errorMessage = ref('');
const successMessage = ref('');
const installedPath = ref('');
const unlistenProgress = ref<UnlistenFn | null>(null);

const versionOptions = computed(() => [
  { label: 'Java 8 (LTS)', value: '8' },
  { label: 'Java 17 (LTS)', value: '17' },
  { label: 'Java 21 (LTS)', value: '21' }
]);

const resetState = () => {
  errorMessage.value = '';
  successMessage.value = '';
  isDownloading.value = false;
  isExtracting.value = false;
  progress.value = 0;
};

const getDownloadUrl = async (version: string): Promise<string> => {
  // Construct URL for Adoptium API
  const baseUrl = "https://api.adoptium.net/v3/binary/latest";
  const featureVersion = version;
  const releaseType = "ga";
  
  // Detect OS and Arch
  let os = "windows";
  // Adoptium API uses 'mac' for macOS
  if (navigator.userAgent.indexOf("Mac") !== -1) os = "mac";
  if (navigator.userAgent.indexOf("Linux") !== -1) os = "linux";

  let arch = "x64"; 
  if (navigator.userAgent.indexOf("aarch64") !== -1 || navigator.userAgent.indexOf("arm64") !== -1) arch = "aarch64";

  return `${baseUrl}/${featureVersion}/${releaseType}/${os}/${arch}/jdk/hotspot/normal/eclipse`;
};

const cancelDownload = async () => {
  try {
    await javaApi.cancelInstall();
    // Reset state immediately for better UX
    isDownloading.value = false;
    isExtracting.value = false;
    loadingUrl.value = false;
    progress.value = 0;
    statusMessage.value = '';
    
    if (unlistenProgress.value) {
      unlistenProgress.value();
      unlistenProgress.value = null;
    }
  } catch (e) {
    console.error("Cancellation failed:", e);
  }
};

const startDownload = async () => {
  resetState();
  loadingUrl.value = true;

  try {
    const url = await getDownloadUrl(selectedVersion.value);
    loadingUrl.value = false;
    isDownloading.value = true;
    progress.value = 0;
    statusMessage.value = i18n.t('settings.java_installing');

    if (unlistenProgress.value) unlistenProgress.value();
    
    unlistenProgress.value = await listen('java-install-progress', (event: any) => {
      const payload = event.payload as { state: string, progress: number, total: number, message: string };
      statusMessage.value = payload.message;
      
      if (payload.state === 'extracting') {
        isExtracting.value = true;
        progress.value = 100; // Force full bar or let indeterminate animation take over
      } else if (payload.state === 'downloading') {
        isExtracting.value = false;
        if (payload.total > 0) {
          progress.value = (payload.progress / payload.total) * 100;
        }
      } else if (payload.state === 'finished') {
        progress.value = 100;
        isExtracting.value = false;
      }
    });

    const resultPath = await javaApi.installJava(url, `jdk-${selectedVersion.value}`);
    
    installedPath.value = resultPath;
    successMessage.value = 'Success'; // Just a flag, text is in template
    emit('installed', resultPath);
    
  } catch (e: any) {
    console.error(e);
    isDownloading.value = false;
    isExtracting.value = false;
    errorMessage.value = i18n.t('settings.java_install_failed') + (typeof e === 'string' ? e : e.message);
  } finally {
    isDownloading.value = false;
    isExtracting.value = false;
    if (unlistenProgress.value) {
      unlistenProgress.value();
      unlistenProgress.value = null;
    }
  }
};

onUnmounted(() => {
  if (unlistenProgress.value) unlistenProgress.value();
});
</script>

<style scoped>
@keyframes indeterminate {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(100%); }
}

.indeterminate-progress {
  position: relative;
  overflow: hidden;
}

.indeterminate-progress::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  width: 50%;
  background: rgba(255, 255, 255, 0.3);
  animation: indeterminate 1.5s infinite linear;
}
</style>
