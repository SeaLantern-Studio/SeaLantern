<template>
  <div class="java-downloader p-4 bg-gray-50 dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
    <h3 class="text-lg font-medium mb-2 flex items-center gap-2">
      <svg class="text-primary-500" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
      </svg>
      {{ $t('settings.java_download') }}
    </h3>
    <p class="text-sm text-gray-500 disconnect-y-1 mb-4">
      {{ $t('settings.java_download_desc') }}
    </p>

    <div v-if="!isDownloading" class="flex items-center gap-4">
      <select 
        v-model="selectedVersion" 
        class="bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded px-3 py-1 text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
      >
        <option value="8">Java 8 (LTS)</option>
        <option value="17">Java 17 (LTS)</option>
        <option value="21">Java 21 (LTS)</option>
      </select>

      <button 
        @click="startDownload"
        class="px-4 py-1.5 bg-primary-600 hover:bg-primary-700 text-white rounded text-sm font-medium transition-colors flex items-center gap-2"
        :disabled="loadingUrl"
      >
        <svg v-if="loadingUrl" class="animate-spin" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
           <circle cx="12" cy="12" r="10" stroke-opacity="0.25" />
           <path d="M4 12a8 8 0 018-8" stroke-opacity="0.75" stroke-linecap="round" />
        </svg>
        <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
        </svg>
        {{ $t('settings.java_download_btn') }}
      </button>
    </div>

    <div v-else class="space-y-2">
      <div class="flex justified-between text-sm text-gray-600 dark:text-gray-400">
        <span>{{ statusMessage }}</span>
        <span>{{ progress }}%</span>
      </div>
      <div class="h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
        <div 
          class="h-full bg-primary-500 transition-all duration-300 ease-out" 
          :style="{ width: `${progress}%` }"
        ></div>
      </div>
    </div>

    <div v-if="errorMessage" class="mt-4 p-3 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 text-sm rounded border border-red-200 dark:border-red-800 flex items-start gap-2">
      <svg class="flex-shrink-0 mt-0.5" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
      </svg>
      <span>{{ errorMessage }}</span>
    </div>

    <div v-if="successMessage" class="mt-4 p-3 bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400 text-sm rounded border border-green-200 dark:border-green-800 flex items-start gap-2">
      <svg class="flex-shrink-0 mt-0.5" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
      </svg>
      <span>{{ successMessage }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { javaApi } from '../api/java';

const { t } = useI18n();
const emit = defineEmits(['installed']);

const selectedVersion = ref('17');
const isDownloading = ref(false);
const loadingUrl = ref(false);
const progress = ref(0);
const statusMessage = ref('');
const errorMessage = ref('');
const successMessage = ref('');
const unlistenProgress = ref<UnlistenFn | null>(null);

const getDownloadUrl = async (version: string): Promise<string> => {
  // Simple mapping for now - ideally query Adoptium API
  // Using generic "latest" links which redirect
  const baseUrl = "https://api.adoptium.net/v3/binary/latest";
  const featureVersion = version;
  const releaseType = "ga";
  
  // Detect OS and Arch
  let os = "windows";
  if (navigator.userAgent.indexOf("Mac") !== -1) os = "mac";
  if (navigator.userAgent.indexOf("Linux") !== -1) os = "linux"; // simple check

  let arch = "x64"; // Defaulting to x64 for now
  if (navigator.userAgent.indexOf("aarch64") !== -1 || navigator.userAgent.indexOf("arm64") !== -1) arch = "aarch64";

  // Construct URL
  // Example: https://api.adoptium.net/v3/binary/latest/17/ga/windows/x64/jdk/hotspot/normal/eclipse
  return `${baseUrl}/${featureVersion}/${releaseType}/${os}/${arch}/jdk/hotspot/normal/eclipse`;
};

const startDownload = async () => {
  errorMessage.value = '';
  successMessage.value = '';
  loadingUrl.value = true;

  try {
    const url = await getDownloadUrl(selectedVersion.value);
    loadingUrl.value = false;
    isDownloading.value = true;
    progress.value = 0;
    statusMessage.value = t('settings.java_installing');

    // Listen for progress events
    if (unlistenProgress.value) unlistenProgress.value();
    unlistenProgress.value = await listen('java-install-progress', (event: any) => {
      const payload = event.payload as { status: string, progress: number };
      statusMessage.value = payload.status;
      progress.value = Math.round(payload.progress * 100);
    });

    const resultPath = await javaApi.installJava(url, `jdk-${selectedVersion.value}`);
    
    isDownloading.value = false;
    successMessage.value = t('settings.java_install_success') + resultPath;
    emit('installed', resultPath);
    
  } catch (e: any) {
    console.error(e);
    isDownloading.value = false;
    errorMessage.value = t('settings.java_install_failed') + (typeof e === 'string' ? e : e.message);
  } finally {
    if (unlistenProgress.value) {
      unlistenProgress.value();
      unlistenProgress.value = null;
    }
  }
};

onUnmounted(() => {
  if (unlistenProgress.value) {
    unlistenProgress.value();
  }
});
</script>
