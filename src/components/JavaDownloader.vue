<template>
  <div class="w-full">
    <div class="flex items-center justify-between py-1">
      <!-- Left Side: Label & Desc -->
      <div class="flex flex-col gap-1 min-w-0 pr-4">
        <span class="text-[0.9375rem] font-medium text-[var(--sl-text-primary)]">
          {{ i18n.t("settings.java_download") }}
        </span>
        <span class="text-[0.8125rem] text-[var(--sl-text-tertiary)] leading-snug">
          {{ i18n.t("settings.java_download_desc") }}
        </span>
      </div>

      <!-- Right Side: Interaction Area -->
      <div class="flex items-center gap-3 flex-shrink-0">
        <!-- Idle State -->
        <template v-if="!isDownloading && !isExtracting && !successMessage">
          <div class="w-24">
            <SLSelect
              v-model="selectedSource"
              :options="sourceOptions"
              :disabled="loadingUrl"
              size="sm"
            />
          </div>
          <div class="w-28">
            <SLSelect
              v-model="selectedVersion"
              :options="versionOptions"
              :disabled="loadingUrl"
              size="sm"
            />
          </div>
          <SLButton variant="primary" size="sm" :loading="loadingUrl" @click="startDownload">
            {{ downloadButtonText }}
          </SLButton>
        </template>

        <!-- Downloading State -->
        <template v-else-if="isDownloading || isExtracting">
          <div class="flex items-center gap-3">
            <div class="flex flex-col items-end gap-1 w-40">
              <div class="flex items-center gap-2 text-xs text-[var(--sl-text-primary)]">
                <span>{{ statusMessage }}</span>
                <span v-if="!isExtracting" class="font-mono opacity-70">{{
                  `${progress.toFixed(0)}%`
                }}</span>
              </div>
              <SLProgress :value="progress" :indeterminate="isExtracting" :show-percent="false" />
            </div>
            <SLButton
              size="sm"
              variant="ghost"
              class="!p-1.5 text-[var(--sl-text-tertiary)] hover:text-[var(--sl-error)]"
              title="Cancel"
              @click="cancelDownload"
            >
              <X :size="16" :stroke-width="2" />
            </SLButton>
          </div>
        </template>

        <!-- Success State -->
        <template v-else-if="successMessage">
          <div class="flex items-center gap-3 animate-fade-in">
            <div class="flex items-center gap-1.5 text-[var(--sl-success)] text-sm font-medium">
              <CheckCircle :size="16" />
              <span>{{ i18n.t("settings.java_install_success").replace(":", "") }}</span>
            </div>
            <SLButton size="sm" variant="ghost" @click="resetState">OK</SLButton>
          </div>
        </template>
      </div>
    </div>

    <!-- Error Message (Full Width below) -->
    <div
      v-if="errorMessage"
      class="mt-2 p-3 bg-red-50 dark:bg-red-900/20 text-[var(--sl-error)] text-sm rounded border border-red-200 dark:border-red-800 flex items-center justify-between animate-fade-in"
    >
      <div class="flex items-center gap-2">
        <AlertCircle class="flex-shrink-0" :size="16" />
        <span>{{ errorMessage }}</span>
      </div>
      <SLButton size="sm" variant="ghost" @click="resetState">
        {{ i18n.t("common.close_notification") }}
      </SLButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from "vue";
import { i18n } from "@language";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { javaApi } from "@api/java";
import SLButton from "@components/common/SLButton.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLProgress from "@components/common/SLProgress.vue";
import { X, CheckCircle, AlertCircle } from "lucide-vue-next";

const emit = defineEmits(["installed"]);

const selectedSource = ref("adoptium");
const selectedVersion = ref("17");
const isDownloading = ref(false);
const isExtracting = ref(false);
const loadingUrl = ref(false);
const progress = ref(0);
const statusMessage = ref("");
const errorMessage = ref("");
const successMessage = ref("");
const installedPath = ref("");
const unlistenProgress = ref<UnlistenFn | null>(null);

const sourceOptions = computed(() => [
  { label: "Adoptium", value: "adoptium" },
  { label: "OpenJDK", value: "openjdk" },
]);

const versionOptions = computed(() => {
  if (selectedSource.value === "openjdk") {
    return [
      { label: "Java 17", value: "17" },
      { label: "Java 21", value: "21" },
    ];
  }
  return [
    { label: "Java 8", value: "8" },
    { label: "Java 17", value: "17" },
    { label: "Java 21", value: "21" },
  ];
});

const downloadButtonText = computed(() => {
  return i18n.t("settings.java_download_btn", { version: selectedVersion.value });
});

const resetState = () => {
  errorMessage.value = "";
  successMessage.value = "";
  isDownloading.value = false;
  isExtracting.value = false;
  progress.value = 0;
};

const getDownloadUrl = (version: string, source: string): { url: string; versionName: string } => {
  // Detect OS and Arch
  let os = "windows";
  if (navigator.userAgent.indexOf("Mac") !== -1) os = "mac";
  if (navigator.userAgent.indexOf("Linux") !== -1) os = "linux";

  let arch = "x64";
  if (navigator.userAgent.indexOf("aarch64") !== -1 || navigator.userAgent.indexOf("arm64") !== -1)
    arch = "aarch64";

  if (source === "adoptium") {
    const baseUrl = "https://api.adoptium.net/v3/binary/latest";
    const releaseType = "ga";
    const adoptiumOs = os === "mac" ? "mac" : os;
    return {
      url: `${baseUrl}/${version}/${releaseType}/${adoptiumOs}/${arch}/jdk/hotspot/normal/eclipse`,
      versionName: `jdk-${version}`,
    };
  } else {
    // OpenJDK source - use archive URLs from jdk.java.net/archive/
    type OsArchUrls = { x64: string; aarch64: string };
    type OsUrls = { windows: OsArchUrls; mac: OsArchUrls; linux: OsArchUrls };
    type VersionUrls = Record<string, OsUrls>;

    const openjdkUrls: VersionUrls = {
      "17": {
        windows: {
          x64: "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_windows-x64_bin.zip",
          aarch64: "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_windows-x64_bin.zip",
        },
        mac: {
          x64: "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_macos-x64_bin.tar.gz",
          aarch64: "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_macos-aarch64_bin.tar.gz",
        },
        linux: {
          x64: "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_linux-x64_bin.tar.gz",
          aarch64: "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_linux-aarch64_bin.tar.gz",
        },
      },
      "21": {
        windows: {
          x64: "https://download.java.net/java/GA/jdk21.0.2/f2283984656d49d69e91c558476027ac/13/GPL/openjdk-21.0.2_windows-x64_bin.zip",
          aarch64: "https://download.java.net/java/GA/jdk21.0.2/f2283984656d49d69e91c558476027ac/13/GPL/openjdk-21.0.2_windows-aarch64_bin.zip",
        },
        mac: {
          x64: "https://download.java.net/java/GA/jdk21.0.2/f2283984656d49d69e91c558476027ac/13/GPL/openjdk-21.0.2_macos-x64_bin.tar.gz",
          aarch64: "https://download.java.net/java/GA/jdk21.0.2/f2283984656d49d69e91c558476027ac/13/GPL/openjdk-21.0.2_macos-aarch64_bin.tar.gz",
        },
        linux: {
          x64: "https://download.java.net/java/GA/jdk21.0.2/f2283984656d49d69e91c558476027ac/13/GPL/openjdk-21.0.2_linux-x64_bin.tar.gz",
          aarch64: "https://download.java.net/java/GA/jdk21.0.2/f2283984656d49d69e91c558476027ac/13/GPL/openjdk-21.0.2_linux-aarch64_bin.tar.gz",
        },
      },
    };

    const versionUrls = openjdkUrls[version] || openjdkUrls["17"];
    const osUrls = versionUrls[os as keyof OsUrls] || versionUrls["windows"];
    const url = osUrls[arch as keyof OsArchUrls] || osUrls["x64"];

    return {
      url,
      versionName: `jdk-${version}-openjdk`,
    };
  }
};

const cancelDownload = async () => {
  try {
    await javaApi.cancelInstall();
    // Reset state immediately for better UX
    isDownloading.value = false;
    isExtracting.value = false;
    loadingUrl.value = false;
    progress.value = 0;
    statusMessage.value = "";

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
    const { url, versionName } = getDownloadUrl(selectedVersion.value, selectedSource.value);
    loadingUrl.value = false;
    isDownloading.value = true;
    progress.value = 0;
    statusMessage.value = i18n.t("settings.java_installing");

    if (unlistenProgress.value) unlistenProgress.value();

    unlistenProgress.value = await listen("java-install-progress", (event: any) => {
      const payload = event.payload as {
        state: string;
        progress: number;
        total: number;
        message: string;
      };
      statusMessage.value = payload.message;

      if (payload.state === "extracting") {
        isExtracting.value = true;
        progress.value = 100; // Force full bar or let indeterminate animation take over
      } else if (payload.state === "downloading") {
        isExtracting.value = false;
        if (payload.total > 0) {
          progress.value = (payload.progress / payload.total) * 100;
        }
      } else if (payload.state === "finished") {
        progress.value = 100;
        isExtracting.value = false;
      }
    });

    const resultPath = await javaApi.installJava(url, versionName);

    installedPath.value = resultPath;
    successMessage.value = "Success"; // Just a flag, text is in template
    emit("installed", resultPath);
  } catch (e: any) {
    console.error(e);
    isDownloading.value = false;
    isExtracting.value = false;
    errorMessage.value =
      i18n.t("settings.java_install_failed") + (typeof e === "string" ? e : e.message);
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
