import { invoke } from "@tauri-apps/api/core";

const SETTINGS_STORAGE_KEY = "sl_web_demo_settings";

function getDefaultSettings() {
  return {
    close_servers_on_exit: true,
    auto_accept_eula: false,
    default_max_memory: 4096,
    default_min_memory: 1024,
    default_port: 25565,
    default_java_path: "",
    default_jvm_args: "",
    console_font_size: 12,
    max_log_lines: 1000,
    cached_java_list: [],
    background_image: "",
    background_opacity: 0.3,
    background_blur: 0,
    background_brightness: 1,
    background_size: "cover",
    acrylic_enabled: false,
    theme: "auto",
    font_size: 14,
    font_family: "",
    color: "default",
    language: "zh-CN",
    developer_mode: false,
    close_action: "ask",
  };
}

function getWebDemoSettings() {
  if (typeof window === "undefined") {
    return getDefaultSettings();
  }

  try {
    const raw = localStorage.getItem(SETTINGS_STORAGE_KEY);
    if (!raw) return getDefaultSettings();
    return { ...getDefaultSettings(), ...(JSON.parse(raw) as Record<string, unknown>) };
  } catch {
    return getDefaultSettings();
  }
}

function saveWebDemoSettings(settings: Record<string, unknown>) {
  if (typeof window === "undefined") return;
  localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings));
}

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export const isWebDemoMode = !isTauriRuntime();

async function mockInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const settings = getWebDemoSettings();

  switch (command) {
    case "get_settings":
      return settings as T;
    case "save_settings": {
      const nextSettings = (args?.settings as Record<string, unknown>) ?? settings;
      saveWebDemoSettings(nextSettings);
      return undefined as T;
    }
    case "reset_settings": {
      const defaults = getDefaultSettings();
      saveWebDemoSettings(defaults);
      return defaults as T;
    }
    case "export_settings":
      return JSON.stringify(settings, null, 2) as T;
    case "import_settings": {
      const json = String(args?.json ?? "{}");
      const parsed = JSON.parse(json) as Record<string, unknown>;
      const nextSettings = { ...getDefaultSettings(), ...parsed };
      saveWebDemoSettings(nextSettings);
      return nextSettings as T;
    }
    case "check_acrylic_support":
      return false as T;
    case "apply_acrylic":
      return undefined as T;
    case "get_system_fonts":
      return ["Inter", "Segoe UI", "Noto Sans", "PingFang SC"] as T;
    case "get_server_list":
      return [] as T;
    case "get_server_status":
      return { status: "Stopped", pid: null, uptime: null } as T;
    case "get_server_logs":
      return [] as T;
    case "search_mods": {
      const q = String(args?.query ?? "").trim().toLowerCase();
      const projectType = String(args?.projectType ?? "mod").toLowerCase();
      const page = Number(args?.page ?? 1);
      const pageSize = Number(args?.pageSize ?? 10);
      const safePage = Number.isFinite(page) && page > 0 ? page : 1;
      const safePageSize = Number.isFinite(pageSize) ? Math.min(50, Math.max(1, pageSize)) : 10;
      const all = [
        {
          id: "demo-sodium",
          name: "Sodium (Demo)",
          summary: "Rendering optimization mod (web demo mock result)",
          download_url: "https://example.com/sodium.jar",
          file_name: "sodium-demo.jar",
          source: "modrinth",
          icon_url: "https://cdn.modrinth.com/data/AANobbMI/icon.png",
          downloads: 15400000,
        },
        {
          id: "demo-lithium",
          name: "Lithium (Demo)",
          summary: "General-purpose optimization mod (web demo mock result)",
          download_url: "https://example.com/lithium.jar",
          file_name: "lithium-demo.jar",
          source: "modrinth",
          icon_url: "https://cdn.modrinth.com/data/gvQqBUqZ/icon.png",
          downloads: 9800000,
        },
        {
          id: "demo-iris",
          name: "Iris Shaders (Demo)",
          summary: "Shaders support for Fabric/Quilt.",
          download_url: "https://example.com/iris.jar",
          file_name: "iris-demo.jar",
          source: "modrinth",
          icon_url: "https://cdn.modrinth.com/data/YL57xq9U/icon.png",
          downloads: 22300000,
        },
        {
          id: "demo-ferrite",
          name: "FerriteCore (Demo)",
          summary: "Memory usage optimizations.",
          download_url: "https://example.com/ferritecore.jar",
          file_name: "ferritecore-demo.jar",
          source: "modrinth",
          downloads: 7700000,
        },
        {
          id: "demo-essentialsx",
          name: "EssentialsX (Demo)",
          summary: "Classic Paper/Spigot plugin for server utilities.",
          download_url: "https://example.com/essentialsx.jar",
          file_name: "essentialsx-demo.jar",
          source: "modrinth",
          downloads: 5400000,
          project_type: "plugin",
        },
      ];

      const typed = all.filter((item) => (item.project_type ?? "mod") === projectType);
      const filtered = !q
        ? typed
        : typed.filter((item) => item.name.toLowerCase().includes(q) || item.summary.toLowerCase().includes(q));
      const total = filtered.length;
      const offset = (safePage - 1) * safePageSize;
      const items = filtered.slice(offset, offset + safePageSize);

      return {
        items,
        total,
        offset,
        limit: safePageSize,
      } as T;
    }
    case "install_mod":
      return undefined as T;
    case "check_update":
      return { has_update: false, latest_version: "", current_version: "web-demo" } as T;
    case "check_pending_update":
      return null as T;
    case "clear_pending_update":
    case "restart_and_install":
    case "open_folder":
      return undefined as T;
    case "get_system_info":
      return {
        os: "Web",
        arch: "browser",
        os_name: "Web Demo",
        os_version: "N/A",
        kernel_version: "N/A",
        host_name: "browser",
        cpu: { name: "Browser", count: 1, usage: 0 },
        memory: { total: 0, used: 0, available: 0, usage: 0 },
        swap: { total: 0, used: 0, usage: 0 },
        disk: { total: 0, used: 0, available: 0, usage: 0, disks: [] },
        network: { total_received: 0, total_transmitted: 0, interfaces: [] },
        uptime: 0,
        process_count: 0,
      } as T;
    case "pick_jar_file":
    case "pick_startup_file":
    case "pick_java_file":
    case "pick_folder":
    case "pick_image_file":
      return null as T;
    case "detect_java":
      return [] as T;
    case "validate_java_path":
      throw new Error("Web demo mode does not support local Java validation.");
    default:
      throw new Error(`Web demo mode does not support command: ${command}`);
  }
}

export async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauriRuntime()) {
    return invoke<T>(command, args);
  }

  return mockInvoke<T>(command, args);
}
