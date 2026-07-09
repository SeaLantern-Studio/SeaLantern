import { systemApi, type HostCapabilities } from "@api/system";

let hostCapabilitiesPromise: Promise<HostCapabilities> | null = null;

const fallbackHostCapabilities: HostCapabilities = {
  build_flavor: "custom",
  plugin_runtime: {
    available: false,
    local_runtime: false,
    ui_bridge: false,
  },
};

export async function getHostCapabilities(): Promise<HostCapabilities> {
  if (!hostCapabilitiesPromise) {
    hostCapabilitiesPromise = systemApi.getHostCapabilities().catch((error) => {
      hostCapabilitiesPromise = null;
      if (import.meta.env.DEV) {
        console.warn(
          "[HostCapabilities] get_host_capabilities failed; using conservative fallback and leaving cache cold for retry",
          error,
        );
      }
      return fallbackHostCapabilities;
    });
  }

  return hostCapabilitiesPromise;
}

export async function isPluginRuntimeAvailable(): Promise<boolean> {
  const capabilities = await getHostCapabilities();
  return capabilities.plugin_runtime.available;
}

export async function isPluginRuntimeUiBridgeAvailable(): Promise<boolean> {
  const capabilities = await getHostCapabilities();
  return capabilities.plugin_runtime.ui_bridge;
}

export function resetHostCapabilitiesCache() {
  hostCapabilitiesPromise = null;
}
