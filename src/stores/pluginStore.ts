import { defineStore } from "pinia";
import { ref } from "vue";
import { listen, emit, type UnlistenFn } from "@tauri-apps/api/event";
import { registerPluginLocale, addPluginTranslations, removePluginTranslations } from "@language";
import { useComponentRegistry } from "@composables/useComponentRegistry";
import { useToast } from "@composables/useToast";
import DOMPurify from "dompurify";
import * as pluginApi from "@api/plugin";
import type { BufferedComponentEvent } from "@api/plugin";
import { setThemeProviderOverrides } from "@utils/theme";
import type {
  PluginInfo,
  PluginNavItem,
  PluginUpdateInfo,
  PluginInstallResult,
  MissingDependency,
  BatchInstallResult,
  SidebarItem,
  PluginDependency,
  SidebarMode,
  PluginUiAction,
  PluginPermissionLog,
  PluginLogEvent,
} from "@type/plugin";

interface PluginUiEvent {
  plugin_id: string;
  action: PluginUiAction;
  element_id: string;
  html: string;
  target?: string;
}

interface PluginSidebarEvent {
  plugin_id: string;
  action: "register" | "unregister";
  label: string;
  icon: string;
  mode?: SidebarMode;
}

function sanitizeHtml(html: string): string {
  return DOMPurify.sanitize(html, {
    FORBID_TAGS: ["script", "iframe", "object", "embed", "form", "link", "meta", "style"],

    FORBID_ATTR: ["style"],
    ALLOW_DATA_ATTR: false,
  });
}

function executePluginScripts(container: HTMLElement, rawHtml: string) {
  const scriptRegex = /<script\b[^>]*>([\s\S]*?)<\/script>/gi;
  let match;
  while ((match = scriptRegex.exec(rawHtml)) !== null) {
    const scriptContent = match[1].trim();
    if (scriptContent) {
      try {
        const scriptEl = document.createElement("script");
        scriptEl.textContent = scriptContent;
        container.appendChild(scriptEl);
      } catch (e) {
        console.error("[PluginUI] Script execution error:", e);
      }
    }
  }
}

export const PLUGIN_THEME_CUSTOM_STYLE_ID = "plugin-theme-custom";

function sanitizeCss(css: string): string {
  let sanitized = css.replace(/@import\s+[^;]+;/gi, "");

  sanitized = sanitized.replace(
    /url\s*\(\s*(['"]?)\s*(https?:\/\/|\/\/)[^)]*\1\s*\)/gi,
    "url(about:blank)",
  );

  sanitized = sanitized.replace(/url\s*\(\s*(['"]?)\s*data:[^)]*\1\s*\)/gi, "url(about:blank)");

  sanitized = sanitized.replace(/expression\s*\(/gi, "(");

  sanitized = sanitized.replace(/-moz-binding\s*:/gi, ":");

  return sanitized;
}

export const usePluginStore = defineStore("plugin", () => {
  const plugins = ref<PluginInfo[]>([]);
  const navItems = ref<PluginNavItem[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const icons = ref<Record<string, string>>({});
  const updates = ref<Record<string, PluginUpdateInfo>>({});

  const pendingDependencies = ref<MissingDependency[]>([]);

  const sidebarItems = ref<SidebarItem[]>([]);

  const permissionLogs = ref<Record<string, PluginPermissionLog[]>>({});

  const pluginLogs = ref<Record<string, PluginLogEvent[]>>({});

  const eventListenerRegistry = new Map<
    string,
    Array<{ element: Element; eventType: string; handler: EventListener }>
  >();

  const pendingComponentCreates = new Map<
    string,
    Array<{
      component_type: string;
      component_id: string;
      props: Record<string, any>;
    }>
  >();

  function consumePendingComponentCreates(pluginId: string) {
    const creates = pendingComponentCreates.get(pluginId) || [];
    pendingComponentCreates.delete(pluginId);
    return creates;
  }

  const pendingComponentDeletes = new Map<string, string[]>();

  function consumePendingComponentDeletes(pluginId: string): string[] {
    const deletes = pendingComponentDeletes.get(pluginId) || [];
    pendingComponentDeletes.delete(pluginId);
    return deletes;
  }

  function removePluginComponents(pluginId: string) {
    const creates = pendingComponentCreates.get(pluginId) || [];
    const componentIds = creates.map((c) => c.component_id);
    if (componentIds.length > 0) {
      pendingComponentDeletes.set(pluginId, componentIds);
    }
    pendingComponentCreates.delete(pluginId);
  }

  async function loadPlugins() {
    loading.value = true;
    error.value = null;
    try {
      plugins.value = await pluginApi.listPlugins();
      setThemeProviderOverrides(
        plugins.value
          .filter((p) => p.state === "enabled")
          .filter((p) => p.manifest.capabilities?.includes("theme-provider"))
          .map((p) => p.manifest.id),
      );
      await loadPluginIcons();
      await injectAllPluginCss();

      collectSidebarItems();

      await replayUiSnapshot();
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function refreshPlugins() {
    loading.value = true;
    error.value = null;
    try {
      plugins.value = await pluginApi.scanPlugins();
      setThemeProviderOverrides(
        plugins.value
          .filter((p) => p.state === "enabled")
          .filter((p) => p.manifest.capabilities?.includes("theme-provider"))
          .map((p) => p.manifest.id),
      );
      await loadPluginIcons();
      await injectAllPluginCss();

      collectSidebarItems();

      await replayUiSnapshot();
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function togglePlugin(
    pluginId: string,
    enable: boolean,
  ): Promise<{ success: boolean; error?: string; disabledPlugins?: string[] }> {
    if (enable) {
      try {
        await pluginApi.enablePlugin(pluginId);

        setThemeProviderOverrides(
          plugins.value
            .filter((p) => p.manifest.id === pluginId || p.state === "enabled")
            .filter((p) => p.manifest.capabilities?.includes("theme-provider"))
            .map((p) => p.manifest.id),
        );

        await injectPluginCss(pluginId);

        const pluginIndex = plugins.value.findIndex((p) => p.manifest.id === pluginId);
        if (pluginIndex !== -1) {
          plugins.value[pluginIndex].state = "enabled";
        }

        setThemeProviderOverrides(
          plugins.value
            .filter((p) => p.state === "enabled")
            .filter((p) => p.manifest.capabilities?.includes("theme-provider"))
            .map((p) => p.manifest.id),
        );
        await loadNavItems();

        const currentPath = window.location.hash.replace(/^#/, "") || "/";
        await pluginApi.onPageChanged(currentPath);
        await replayUiSnapshot();
        setTimeout(() => replayUiSnapshot(), 300);

        return { success: true };
      } catch (e) {
        const errorMsg = String(e);

        const pluginIndex = plugins.value.findIndex((p) => p.manifest.id === pluginId);
        if (pluginIndex !== -1) {
          plugins.value[pluginIndex].state = "disabled";
          removePluginCss(pluginId);
        }

        setThemeProviderOverrides(
          plugins.value
            .filter((p) => p.state === "enabled")
            .filter((p) => p.manifest.capabilities?.includes("theme-provider"))
            .map((p) => p.manifest.id),
        );

        return { success: false, error: errorMsg };
      }
    } else {
      try {
        const disabledPlugins = await pluginApi.disablePlugin(pluginId);

        setThemeProviderOverrides(
          plugins.value
            .filter((p) => p.manifest.id !== pluginId)
            .filter((p) => !disabledPlugins.includes(p.manifest.id))
            .filter((p) => p.state === "enabled")
            .filter((p) => p.manifest.capabilities?.includes("theme-provider"))
            .map((p) => p.manifest.id),
        );

        removePluginCss(pluginId);
        removePluginUiElements(pluginId);
        cleanupPluginEventListeners(pluginId);
        removePluginProxies(pluginId);
        removePluginComponents(pluginId);

        const pluginIndex = plugins.value.findIndex((p) => p.manifest.id === pluginId);
        if (pluginIndex !== -1) {
          plugins.value[pluginIndex].state = "disabled";
        }

        setThemeProviderOverrides(
          plugins.value
            .filter((p) => p.state === "enabled")
            .filter((p) => p.manifest.capabilities?.includes("theme-provider"))
            .map((p) => p.manifest.id),
        );

        for (const disabledId of disabledPlugins) {
          const idx = plugins.value.findIndex((p) => p.manifest.id === disabledId);
          if (idx !== -1) {
            plugins.value[idx].state = "disabled";
            removePluginCss(disabledId);
            removePluginUiElements(disabledId);
            cleanupPluginEventListeners(disabledId);
            removePluginProxies(disabledId);
            removePluginComponents(disabledId);
          }
        }

        await loadNavItems();
        return { success: true, disabledPlugins };
      } catch (e) {
        const errorMsg = String(e);
        return { success: false, error: errorMsg };
      }
    }
  }

  async function loadNavItems() {
    try {
      navItems.value = await pluginApi.getPluginNavItems();

      collectSidebarItems();
    } catch (e) {
      console.error("Failed to load nav items:", e);
    }
  }

  function collectSidebarItems() {
    // 已禁用插件注册的侧栏按钮功能
    sidebarItems.value = [];
  }

  async function installFromZip(zipPath: string): Promise<PluginInstallResult> {
    loading.value = true;
    try {
      const result = await pluginApi.installPlugin(zipPath);

      if (result.missing_dependencies.length > 0) {
        pendingDependencies.value = result.missing_dependencies;
      }
      await loadPlugins();
      return result;
    } catch (e) {
      console.error("[Plugin] installFromZip failed:", e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function installBatch(paths: string[]): Promise<BatchInstallResult> {
    loading.value = true;
    try {
      const result = await pluginApi.installPluginsBatch(paths);
      if (result.failed.length > 0) {
        for (const item of result.failed) {
          console.error(`[Plugin] Batch install failed for "${item.path}":`, item.error);
        }
      }
      await loadPlugins();
      return result;
    } catch (e) {
      console.error("[Plugin] installBatch failed:", e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function loadPluginIcons() {
    await Promise.all(
      plugins.value.map(async (plugin) => {
        if (plugin.manifest.icon && !icons.value[plugin.manifest.id]) {
          try {
            const iconData = await pluginApi.getPluginIcon(plugin.manifest.id);
            if (iconData) {
              icons.value[plugin.manifest.id] = iconData;
            }
          } catch (e) {
            console.error(`Failed to load icon for ${plugin.manifest.id}:`, e);
          }
        }
      }),
    );
  }

  async function getPluginSettings(pluginId: string): Promise<any> {
    try {
      return await pluginApi.getPluginSettings(pluginId);
    } catch (e) {
      console.error(`Failed to get settings for ${pluginId}:`, e);
      return {};
    }
  }

  async function setPluginSettings(pluginId: string, settings: any): Promise<void> {
    try {
      await pluginApi.setPluginSettings(pluginId, settings);

      if (hasCapability(pluginId, "theme-provider")) {
        await applyThemeProviderSettings(pluginId);
      }
      if (hasCapability(pluginId, "theme-widgets-provider")) {
        await applyThemeWidgetsProviderSettings(pluginId);
      }
    } catch (e) {
      console.error(`Failed to set settings for ${pluginId}:`, e);
      throw e;
    }
  }

  async function injectPluginCss(pluginId: string) {
    try {
      const css = await pluginApi.getPluginCss(pluginId);
      if (css) {
        removePluginCss(pluginId);
        const style = document.createElement("style");
        style.id = `plugin-manifest-css-${pluginId}`;
        style.setAttribute("data-plugin-id", pluginId);
        style.setAttribute("data-plugin-source", "manifest");
        style.textContent = sanitizeCss(css);
        document.head.appendChild(style);

        if (hasCapability(pluginId, "theme-provider")) {
          await applyThemeProviderSettings(pluginId);
        }
        if (hasCapability(pluginId, "theme-widgets-provider")) {
          await applyThemeWidgetsProviderSettings(pluginId);
        }
      }
    } catch (e) {
      console.error(`Failed to inject CSS for ${pluginId}:`, e);
    }
  }

  function removePluginCss(pluginId: string) {
    document.querySelectorAll(`style[data-plugin-id="${pluginId}"]`).forEach((el) => el.remove());

    if (hasCapability(pluginId, "theme-provider")) {
      removeThemeSettings();
    }
    if (hasCapability(pluginId, "theme-widgets-provider")) {
      removeThemeWidgetsSettings();
    }
  }

  async function injectAllPluginCss() {
    try {
      const allCss = await pluginApi.getAllPluginCss();

      removeThemeSettings();
      removeThemeWidgetsSettings();

      setThemeProviderOverrides(
        plugins.value
          .filter((p) => p.state === "enabled")
          .filter((p) => p.manifest.capabilities?.includes("theme-provider"))
          .map((p) => p.manifest.id),
      );

      document.querySelectorAll("style[data-plugin-id]").forEach((el) => {
        const id = (el as HTMLElement).id || "";
        if (id.startsWith("plugin-css-")) return;
        el.remove();
      });

      for (const [pluginId, css] of allCss) {
        const plugin = plugins.value.find((p) => p.manifest.id === pluginId);
        if (!plugin?.manifest.permissions?.includes("ui")) continue;
        if (css) {
          const style = document.createElement("style");
          style.id = `plugin-manifest-css-${pluginId}`;
          style.setAttribute("data-plugin-id", pluginId);
          style.setAttribute("data-plugin-source", "manifest");
          style.textContent = sanitizeCss(css);
          document.head.appendChild(style);
        }
      }

      for (const [pluginId] of allCss) {
        if (hasCapability(pluginId, "theme-provider")) {
          await applyThemeProviderSettings(pluginId);
        }
        if (hasCapability(pluginId, "theme-widgets-provider")) {
          await applyThemeWidgetsProviderSettings(pluginId);
        }
      }
    } catch (e) {
      console.error("Failed to inject all plugin CSS:", e);
    }
  }

  function playSound(soundUrl: string) {
    try {
      const audio = new Audio(soundUrl);
      audio.play().catch((e) => console.error("Failed to play sound:", e));
    } catch (e) {
      console.error("Failed to create audio:", e);
    }
  }

  function hasCapability(pluginId: string, capability: string): boolean {
    const plugin = plugins.value.find((p) => p.manifest.id === pluginId);
    return plugin?.manifest.capabilities?.includes(capability) ?? false;
  }

  function getPluginDefaults(pluginId: string): Record<string, string> {
    const plugin = plugins.value.find((p) => p.manifest.id === pluginId);
    if (!plugin?.manifest.settings) return {};
    const defaults: Record<string, string> = {};
    for (const field of plugin.manifest.settings) {
      if (field.default !== undefined) {
        defaults[field.key] = String(field.default);
      }
    }
    return defaults;
  }

  function injectThemeStyle(
    settings: Record<string, any>,
    varMap: Record<string, string>,
    defaults: Record<string, string>,
  ) {
    const noPrefixVarMap: Record<string, string> = {
      bg_primary: "--bg-primary",
      bg_secondary: "--bg-secondary",
      bg_tertiary: "--bg-tertiary",
      accent_primary: "--accent-primary",
      accent_secondary: "--accent-secondary",
      text_primary: "--text-primary",
      text_secondary: "--text-secondary",
      border_color: "--border-color",
      glass_blur: "--sl-glass-blur",
      border_radius: "--sl-radius-lg",
    };

    let customCss = ":root {\n";
    let hasCustom = false;
    for (const [key, cssVar] of Object.entries(varMap)) {
      const value = settings[key] ?? defaults[key];
      if (value === undefined) continue;

      customCss += `  ${cssVar}: ${value};\n`;
      const noPrefixVar = noPrefixVarMap[key];
      if (noPrefixVar) {
        customCss += `  ${noPrefixVar}: ${value};\n`;
      }
      hasCustom = true;

      if (key === "border_radius") {
        const radiusValue = parseFloat(String(value));
        if (!isNaN(radiusValue)) {
          customCss += `  --sl-radius-md: ${radiusValue * 0.625}px;\n`;
          customCss += `  --sl-radius-sm: ${radiusValue * 0.375}px;\n`;
          customCss += `  --sl-radius-xl: ${radiusValue * 1.5}px;\n`;
          customCss += `  --radius-md: ${radiusValue * 0.625}px;\n`;
          customCss += `  --radius-sm: ${radiusValue * 0.375}px;\n`;
        }
      }
    }
    customCss += "}\n";

    let customEl = document.getElementById(PLUGIN_THEME_CUSTOM_STYLE_ID) as HTMLStyleElement | null;
    if (hasCustom) {
      if (!customEl) {
        customEl = document.createElement("style");
        customEl.id = PLUGIN_THEME_CUSTOM_STYLE_ID;
        document.head.appendChild(customEl);
      }
      customEl.textContent = sanitizeCss(customCss);
    } else if (customEl) {
      customEl.remove();
    }
  }

  async function applyThemeProviderSettings(pluginId: string) {
    const plugin = plugins.value.find((p) => p.manifest.id === pluginId);
    if (!plugin) return;
    const varMap = plugin.manifest.theme_var_map;
    if (!varMap) return;

    try {
      const settings = await getPluginSettings(pluginId);
      if (!settings) return;

      if (settings.preset && settings.preset !== "default") {
        document.documentElement.setAttribute("data-theme-preset", settings.preset);
      } else {
        document.documentElement.removeAttribute("data-theme-preset");
      }

      const defaults = getPluginDefaults(pluginId);
      injectThemeStyle(settings, varMap, defaults);

      const root = document.documentElement;
      const varsToClear = [
        "--sl-bg",
        "--sl-bg-secondary",
        "--sl-bg-tertiary",
        "--sl-primary",
        "--sl-primary-light",
        "--sl-primary-dark",
        "--sl-primary-bg",
        "--sl-accent",
        "--sl-accent-light",
        "--sl-text-primary",
        "--sl-text-secondary",
        "--sl-text-tertiary",
        "--sl-border",
        "--sl-border-light",
        "--sl-surface",
        "--sl-surface-hover",
        "--sl-shadow-sm",
        "--sl-shadow-md",
        "--sl-shadow-lg",
        "--sl-shadow-xl",
      ];
      for (const v of varsToClear) {
        root.style.removeProperty(v);
      }
    } catch (e) {
      console.error("Failed to apply theme provider settings:", e);
    }
  }

  function removeThemeSettings() {
    document.documentElement.removeAttribute("data-theme-preset");

    const customEl = document.getElementById(PLUGIN_THEME_CUSTOM_STYLE_ID);
    if (customEl) {
      customEl.remove();
    }
  }

  async function applyThemeWidgetsProviderSettings(pluginId: string) {
    const plugin = plugins.value.find((p) => p.manifest.id === pluginId);
    if (!plugin) return;

    try {
      const settings = await getPluginSettings(pluginId);
      const root = document.documentElement;

      const booleanKeys =
        plugin.manifest.settings?.filter((s) => s.type === "boolean").map((s) => s.key) ?? [];

      for (const key of booleanKeys) {
        const enabled = settings[key] === true || settings[key] === "true";
        root.classList.toggle(`plugin-${key.replace(/_/g, "-")}`, enabled);
      }

      const numberKeys =
        plugin.manifest.settings?.filter((s) => s.type === "number").map((s) => s.key) ?? [];

      for (const key of numberKeys) {
        const value = settings[key];
        if (value !== undefined) {
          root.style.setProperty(`--plugin-${key.replace(/_/g, "-")}`, String(value));
        }
      }
    } catch (e) {
      console.error("Failed to apply theme widgets provider settings:", e);
    }
  }

  function removeThemeWidgetsSettings() {
    const root = document.documentElement;
    root.removeAttribute("data-glow-intensity");
    root.removeAttribute("data-gradient-text");
    root.removeAttribute("data-particles");
  }

  async function deletePlugin(pluginId: string, deleteData?: boolean) {
    try {
      await pluginApi.deletePlugin(pluginId, deleteData);
      delete icons.value[pluginId];
      delete updates.value[pluginId];
      removePluginCss(pluginId);
      await loadPlugins();
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function deletePlugins(pluginIds: string[], deleteData?: boolean) {
    try {
      await pluginApi.deletePlugins(pluginIds, deleteData);
      for (const pluginId of pluginIds) {
        delete icons.value[pluginId];
        delete updates.value[pluginId];
        removePluginCss(pluginId);
      }
      await loadPlugins();
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function checkUpdate(pluginId: string) {
    try {
      const update = await pluginApi.checkPluginUpdate(pluginId);
      if (update) {
        updates.value[pluginId] = update;
      }
      return update;
    } catch (e) {
      console.error(`Failed to check update for ${pluginId}:`, e);
      return null;
    }
  }

  async function checkAllUpdates() {
    try {
      const allUpdates = await pluginApi.checkAllPluginUpdates();
      for (const update of allUpdates) {
        updates.value[update.plugin_id] = update;
      }
      return allUpdates;
    } catch (e) {
      console.error("Failed to check all updates:", e);
      return [];
    }
  }

  function getPluginUiContainer(): HTMLElement {
    let container = document.getElementById("plugin-ui-container");
    if (!container) {
      container = document.createElement("div");
      container.id = "plugin-ui-container";
      container.style.cssText =
        "position: fixed; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; z-index: 9998;";
      document.body.appendChild(container);
    }
    return container;
  }

  async function handlePluginUiEvent(event: PluginUiEvent) {
    const { plugin_id, action, element_id, html } = event;

    const target = element_id;
    const fullElementId = `plugin-ui-${plugin_id}-${element_id}`;

    console.log(
      `[PluginUI] ${action} element: ${fullElementId}${target ? ` target: ${target}` : ""}`,
    );

    switch (action) {
      case "inject": {
        const existing = document.getElementById(fullElementId);
        if (existing) {
          existing.remove();
        }

        const container = getPluginUiContainer();
        const wrapper = document.createElement("div");
        wrapper.id = fullElementId;
        wrapper.setAttribute("data-plugin-id", plugin_id);
        wrapper.style.pointerEvents = "auto";
        wrapper.innerHTML = sanitizeHtml(html);
        container.appendChild(wrapper);
        executePluginScripts(wrapper, html);
        break;
      }

      case "remove": {
        const element = document.getElementById(fullElementId);
        if (element) {
          element.remove();
        }
        break;
      }

      case "update": {
        const element = document.getElementById(fullElementId);
        if (element) {
          element.innerHTML = sanitizeHtml(html);
          executePluginScripts(element, html);
        } else {
          handlePluginUiEvent({ ...event, action: "inject" });
        }
        break;
      }

      case "remove_all": {
        removePluginUiElements(plugin_id);

        cleanupPluginEventListeners(plugin_id);
        break;
      }

      case "hide": {
        if (!target) break;
        const elements = document.querySelectorAll(target);
        elements.forEach((el) => {
          (el as HTMLElement).style.display = "none";
          (el as HTMLElement).dataset.pluginHidden = plugin_id;
        });
        break;
      }

      case "show": {
        if (!target) break;
        const elements = document.querySelectorAll(target);
        elements.forEach((el) => {
          (el as HTMLElement).style.display = "";
          delete (el as HTMLElement).dataset.pluginHidden;
        });
        break;
      }

      case "disable": {
        if (!target) break;
        const elements = document.querySelectorAll(target);
        elements.forEach((el) => {
          (el as HTMLElement).setAttribute("disabled", "true");
          (el as HTMLElement).style.pointerEvents = "none";
          (el as HTMLElement).style.opacity = "0.5";
          (el as HTMLElement).dataset.pluginDisabled = plugin_id;
        });
        break;
      }

      case "enable": {
        if (!target) break;
        const elements = document.querySelectorAll(target);
        elements.forEach((el) => {
          (el as HTMLElement).removeAttribute("disabled");
          (el as HTMLElement).style.pointerEvents = "";
          (el as HTMLElement).style.opacity = "";
          delete (el as HTMLElement).dataset.pluginDisabled;
        });
        break;
      }

      case "insert": {
        if (!target) break;
        const [placement, selector] = target.split("|");
        const targetEl = document.querySelector(selector);
        if (targetEl) {
          const wrapper = document.createElement("div");
          wrapper.dataset.pluginInserted = plugin_id;
          wrapper.innerHTML = sanitizeHtml(html);
          switch (placement) {
            case "before":
              targetEl.parentNode?.insertBefore(wrapper, targetEl);
              break;
            case "after":
              targetEl.parentNode?.insertBefore(wrapper, targetEl.nextSibling);
              break;
            case "prepend":
              targetEl.prepend(wrapper);
              break;
            case "append":
              targetEl.append(wrapper);
              break;
          }
        }
        break;
      }

      case "remove_selector": {
        if (!target) break;
        const elements = document.querySelectorAll(target);
        elements.forEach((el) => {
          if ((el as HTMLElement).dataset.pluginInserted === plugin_id) {
            el.remove();
          }
        });

        document.querySelectorAll(`[data-plugin-inserted="${plugin_id}"]`).forEach((el) => {
          if (el.querySelector(target) || el.matches(target)) {
            el.remove();
          }
        });
        break;
      }

      case "set_style": {
        if (!target) break;
        const elements = document.querySelectorAll(target);
        try {
          const styles = JSON.parse(html);
          elements.forEach((el) => {
            Object.entries(styles).forEach(([prop, value]) => {
              (el as HTMLElement).style.setProperty(
                prop.replace(/([A-Z])/g, "-$1").toLowerCase(),
                value as string,
              );
            });
          });
        } catch (e) {
          console.error("[PluginUI] Invalid style JSON:", e);
        }
        break;
      }

      case "set_attribute": {
        if (!target) break;
        const elements = document.querySelectorAll(target);
        try {
          const { attribute, value } = JSON.parse(html);
          elements.forEach((el) => {
            el.setAttribute(attribute, value);
          });
        } catch (e) {
          console.error("[PluginUI] Invalid attribute JSON:", e);
        }
        break;
      }

      case "query": {
        if (!target) break;
        const elements = document.querySelectorAll(target);
        const results = Array.from(elements).map((el) => ({
          id: el.id || "",
          tag: el.tagName.toLowerCase(),
          classes: Array.from(el.classList),
          text: (el as HTMLElement).innerText?.substring(0, 500) || "",
          visible: (el as HTMLElement).offsetParent !== null,
          enabled: !(el as HTMLElement).hasAttribute("disabled"),
        }));
        emit("plugin-ui-query-result", { plugin_id, elements: results });
        break;
      }

      case "element_get_text": {
        if (!target) break;
        try {
          const parsed = JSON.parse(html || "{}");
          const requestId = parsed.request_id;
          const el = document.querySelector(target);
          const text = el ? (el as HTMLElement).innerText || "" : "";
          console.log(
            `[PluginUI] element_get_text: selector=${target}, found=${!!el}, text="${text}", req_id=${requestId}`,
          );
          emit("plugin-element-response", {
            plugin_id,
            request_id: requestId,
            data: text,
          });
        } catch (e) {
          console.error("[PluginUI] element_get_text error:", e);
        }
        break;
      }

      case "element_get_value": {
        if (!target) break;
        try {
          const parsed = JSON.parse(html || "{}");
          const requestId = parsed.request_id;
          const el = document.querySelector(target) as
            | HTMLInputElement
            | HTMLSelectElement
            | HTMLTextAreaElement
            | null;
          const value = el ? el.value || "" : "";
          emit("plugin-element-response", {
            plugin_id,
            request_id: requestId,
            data: value,
          });
        } catch (e) {
          console.error("[PluginUI] element_get_value error:", e);
        }
        break;
      }

      case "element_get_attribute": {
        if (!target) break;
        try {
          const parsed = JSON.parse(html || "{}");
          const requestId = parsed.request_id;
          const attr = parsed.attr;
          const el = document.querySelector(target);
          const val = el ? el.getAttribute(attr) || "" : "";
          emit("plugin-element-response", {
            plugin_id,
            request_id: requestId,
            data: val,
          });
        } catch (e) {
          console.error("[PluginUI] element_get_attribute error:", e);
        }
        break;
      }

      case "element_get_attributes": {
        if (!target) break;
        try {
          const parsed = JSON.parse(html || "{}");
          const requestId = parsed.request_id;
          const el = document.querySelector(target);
          const attrs: Record<string, string> = {};
          if (el) {
            Array.from(el.attributes).forEach((a) => {
              attrs[a.name] = a.value;
            });
          }
          emit("plugin-element-response", {
            plugin_id,
            request_id: requestId,
            data: JSON.stringify(attrs),
          });
        } catch (e) {
          console.error("[PluginUI] element_get_attributes error:", e);
        }
        break;
      }

      case "element_click": {
        if (!target) break;
        const el = document.querySelector(target) as HTMLElement | null;
        if (el) el.click();
        break;
      }

      case "element_set_value": {
        if (!target) break;
        const el = document.querySelector(target) as HTMLInputElement | null;
        if (el) {
          try {
            const { value } = JSON.parse(html || "{}");
            el.value = value;
            el.dispatchEvent(new Event("input", { bubbles: true }));
            el.dispatchEvent(new Event("change", { bubbles: true }));
          } catch (e) {
            console.error("[PluginUI] Invalid JSON:", e);
          }
        }
        break;
      }

      case "element_check": {
        if (!target) break;
        const el = document.querySelector(target) as HTMLInputElement | null;
        if (el) {
          try {
            const { checked } = JSON.parse(html || "{}");
            el.checked = checked;
            el.dispatchEvent(new Event("change", { bubbles: true }));
          } catch (e) {
            console.error("[PluginUI] Invalid JSON:", e);
          }
        }
        break;
      }

      case "element_select": {
        if (!target) break;
        const el = document.querySelector(target) as HTMLSelectElement | null;
        if (el) {
          try {
            const { value } = JSON.parse(html || "{}");
            el.value = value;
            el.dispatchEvent(new Event("change", { bubbles: true }));
          } catch (e) {
            console.error("[PluginUI] Invalid JSON:", e);
          }
        }
        break;
      }

      case "element_focus": {
        if (!target) break;
        const el = document.querySelector(target) as HTMLElement | null;
        if (el) el.focus();
        break;
      }

      case "element_blur": {
        if (!target) break;
        const el = document.querySelector(target) as HTMLElement | null;
        if (el) el.blur();
        break;
      }

      case "element_on_change": {
        if (!target) break;
        const el = document.querySelector(target) as HTMLElement | null;
        if (el) {
          const handler = (e: Event) => {
            const value = (e.target as HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement)
              .value;

            emit("plugin-element-change", {
              plugin_id,
              selector: target,
              value,
            });
          };
          el.addEventListener("change", handler);

          if (!eventListenerRegistry.has(plugin_id)) {
            eventListenerRegistry.set(plugin_id, []);
          }
          eventListenerRegistry.get(plugin_id)!.push({ element: el, eventType: "change", handler });
        }
        break;
      }

      case "inject_css": {
        const styleId = `plugin-css-${plugin_id}-${element_id}`;
        const css = sanitizeCss(html);

        const existingStyle = document.getElementById(styleId) as HTMLStyleElement | null;
        if (existingStyle) {
          existingStyle.textContent = css;
          existingStyle.setAttribute("data-plugin-id", plugin_id);
          existingStyle.setAttribute("data-plugin-source", "runtime");
          break;
        }

        const style = document.createElement("style");
        style.id = styleId;
        style.setAttribute("data-plugin-id", plugin_id);
        style.setAttribute("data-plugin-source", "runtime");
        style.textContent = css;
        document.head.appendChild(style);
        break;
      }

      case "remove_css": {
        const styleId = `plugin-css-${plugin_id}-${element_id}`;
        const styleEl = document.getElementById(styleId);
        if (styleEl) {
          styleEl.remove();
        }
        break;
      }

      case "toast": {
        try {
          const { type, message, duration } = JSON.parse(html || "{}");
          const validTypes = ["success", "error", "warning", "info"] as const;
          const toastType = validTypes.includes(type) ? type : "info";
          const toast = useToast();
          (
            toast[toastType as "success" | "error" | "warning" | "info"] as (
              msg: string,
              dur?: number,
            ) => void
          )(message || "", duration);
        } catch (e) {
          console.error("[PluginUI] toast error:", e);
        }
        break;
      }
    }
  }

  function cleanupPluginEventListeners(pluginId: string) {
    const listeners = eventListenerRegistry.get(pluginId);
    if (listeners) {
      for (const { element, eventType, handler } of listeners) {
        element.removeEventListener(eventType, handler);
      }
      eventListenerRegistry.delete(pluginId);
      console.log(`[PluginUI] Cleaned up event listeners for plugin: ${pluginId}`);
    }
  }

  function removePluginUiElements(pluginId: string) {
    const elements = document.querySelectorAll(`[data-plugin-id="${pluginId}"]`);
    console.log(`[PluginUI] Removing ${elements.length} UI elements for plugin: ${pluginId}`);
    elements.forEach((el) => el.remove());

    const insertedElements = document.querySelectorAll(`[data-plugin-inserted="${pluginId}"]`);
    insertedElements.forEach((el) => el.remove());

    const hiddenElements = document.querySelectorAll(`[data-plugin-hidden="${pluginId}"]`);
    hiddenElements.forEach((el) => {
      (el as HTMLElement).style.display = "";
      delete (el as HTMLElement).dataset.pluginHidden;
    });

    const disabledElements = document.querySelectorAll(`[data-plugin-disabled="${pluginId}"]`);
    disabledElements.forEach((el) => {
      (el as HTMLElement).removeAttribute("disabled");
      (el as HTMLElement).style.pointerEvents = "";
      (el as HTMLElement).style.opacity = "";
      delete (el as HTMLElement).dataset.pluginDisabled;
    });
  }

  let uiEventUnlisten: UnlistenFn | null = null;

  async function initUiEventListener() {
    if (uiEventUnlisten) {
      return;
    }

    try {
      uiEventUnlisten = await listen<PluginUiEvent>("plugin-ui-event", (event) => {
        console.log(`[PluginUI] Received: ${event.payload.action} for ${event.payload.element_id}`);
        handlePluginUiEvent(event.payload);
      });
      console.log("[PluginUI] Event listener initialized");
    } catch (e) {
      console.error("[PluginUI] Failed to initialize event listener:", e);
    }
  }

  function cleanupUiEventListener() {
    if (uiEventUnlisten) {
      uiEventUnlisten();
      uiEventUnlisten = null;
    }
  }

  let sidebarEventUnlisten: UnlistenFn | null = null;

  async function initSidebarEventListener() {
    // 已禁用插件动态注册侧栏按钮功能
    console.log("[PluginSidebar] Event listener disabled");
  }

  function cleanupSidebarEventListener() {
    if (sidebarEventUnlisten) {
      sidebarEventUnlisten();
      sidebarEventUnlisten = null;
    }
  }

  let permissionLogUnlisten: UnlistenFn | null = null;

  function addPermissionLog(log: PluginPermissionLog) {
    const logs = permissionLogs.value[log.plugin_id] || [];
    const newLogs = [...logs, log];

    if (newLogs.length > 200) newLogs.shift();
    permissionLogs.value = {
      ...permissionLogs.value,
      [log.plugin_id]: newLogs,
    };
  }

  function addPluginLog(log: PluginLogEvent) {
    const logs = pluginLogs.value[log.plugin_id] || [];
    const newLogs = [...logs, log];

    if (newLogs.length > 500) newLogs.splice(0, newLogs.length - 500);
    pluginLogs.value = {
      ...pluginLogs.value,
      [log.plugin_id]: newLogs,
    };
  }

  function getPluginLogs(pluginId: string): PluginLogEvent[] {
    return pluginLogs.value[pluginId] || [];
  }

  function getPermissionLogs(pluginId: string): PluginPermissionLog[] {
    return permissionLogs.value[pluginId] || [];
  }

  const HIGH_RISK_PERMISSIONS = [
    "network",
    "fs",
    "server",
    "console",
    "element",
    "system",
    "execute_program",
    "plugin_folder_access",
  ];

  function getHighRiskPermissions(pluginId: string): string[] {
    const plugin = plugins.value.find((p) => p.manifest.id === pluginId);
    if (!plugin || !plugin.manifest.permissions) return [];
    return plugin.manifest.permissions.filter((p) => HIGH_RISK_PERMISSIONS.includes(p));
  }

  function clearPermissionLogs(pluginId: string) {
    delete permissionLogs.value[pluginId];
  }

  async function initPermissionLogListener() {
    if (permissionLogUnlisten) {
      return;
    }

    try {
      permissionLogUnlisten = await listen<PluginPermissionLog>(
        "plugin-permission-log",
        (event) => {
          const log = event.payload;
          addPermissionLog(log);
          console.log(`[PluginPermission] ${log.log_type} from ${log.plugin_id}: ${log.action}`);
        },
      );
      console.log("[PluginPermission] Event listener initialized");
    } catch (e) {
      console.error("[PluginPermission] Failed to initialize event listener:", e);
    }
  }

  function cleanupPermissionLogListener() {
    if (permissionLogUnlisten) {
      permissionLogUnlisten();
      permissionLogUnlisten = null;
    }
  }

  let pluginLogUnlisten: UnlistenFn | null = null;

  async function initPluginLogListener() {
    if (pluginLogUnlisten) {
      return;
    }

    try {
      pluginLogUnlisten = await listen<PluginLogEvent>("plugin-log-event", (event) => {
        const log = event.payload;
        addPluginLog(log);
      });
    } catch (e) {
      console.error("[PluginLog] Failed to initialize event listener:", e);
    }
  }

  function cleanupPluginLogListener() {
    if (pluginLogUnlisten) {
      pluginLogUnlisten();
      pluginLogUnlisten = null;
    }
  }

  interface PluginComponentEvent {
    plugin_id: string;
    action: "get" | "set" | "call" | "on" | "proxy" | "list" | "remove_proxy" | "create";
    component_id: string;
    component_type?: string;
    props?: Record<string, any>;
    prop?: string;
    value?: any;
    method?: string;
    args?: any[];
    callback_id?: string;
    priority?: number;
    page_filter?: string;
  }

  function hasComponentPerm(pluginId: string, perm: string): boolean {
    const plugin = plugins.value.find((p) => p.manifest.id === pluginId);
    return plugin?.manifest.permissions?.includes(perm) ?? false;
  }

  async function handlePluginComponentEvent(event: PluginComponentEvent) {
    const { plugin_id, action, component_id, callback_id } = event;
    const reg = useComponentRegistry();

    const canRead = hasComponentPerm(plugin_id, "ui.component.read");
    const canWrite = hasComponentPerm(plugin_id, "ui.component.write");
    const canProxy = hasComponentPerm(plugin_id, "ui.component.proxy");
    const canCreate = hasComponentPerm(plugin_id, "ui.component.create");

    async function reply(data: any) {
      if (callback_id) {
        await emit(`plugin:component:callback:${callback_id}`, data);
      }
    }

    switch (action) {
      case "list": {
        if (!canRead) {
          await reply({ error: "permission denied" });
          return;
        }
        await reply(reg.list(event.page_filter));
        break;
      }
      case "get": {
        if (!canRead) {
          await reply({ error: "permission denied" });
          return;
        }
        const handle = reg.get(component_id);
        await reply(handle ? handle.get(event.prop!) : null);
        break;
      }
      case "set": {
        if (!canWrite) return;
        for (const { handle } of reg.getAll(component_id)) {
          handle.set(event.prop!, event.value);
        }
        trackPluginComponentSet(plugin_id, component_id, event.prop!);
        break;
      }
      case "call": {
        if (!canWrite) {
          await reply({ error: "permission denied" });
          return;
        }
        const handle = reg.get(component_id);
        const result = handle ? handle.call(event.method!, ...(event.args ?? [])) : null;
        await reply(result);
        break;
      }
      case "on": {
        if (!canRead) return;
        const handle = reg.get(component_id);
        if (handle && event.prop) {
          handle.on(event.prop, async (...args: any[]) => {
            await emit(`plugin:component:event:${plugin_id}:${component_id}:${event.prop}`, args);
          });
        }
        break;
      }
      case "proxy": {
        if (!canProxy) return;
        reg.addProxy(component_id, {
          pluginId: plugin_id,
          priority: event.priority ?? 0,
          handler: (e) => {
            if (callback_id) {
              void emit(`plugin:component:proxy:${callback_id}`, e);
            }
          },
        });
        break;
      }
      case "remove_proxy": {
        reg.removeProxy(component_id, plugin_id);
        break;
      }
      case "create": {
        if (!canCreate) {
          await reply({ error: "permission denied" });
          return;
        }
        const { component_type, props } = event;
        if (component_type && component_id) {
          if (!pendingComponentCreates.has(plugin_id)) {
            pendingComponentCreates.set(plugin_id, []);
          }
          pendingComponentCreates.get(plugin_id)!.push({
            component_type,
            component_id,
            props: props || {},
          });
          console.log(
            `[PluginComponent] Queued component create: ${component_type} (${component_id})`,
          );
          await reply({ ok: true });
        } else {
          await reply({ error: "invalid payload" });
        }
        break;
      }
    }
  }

  let componentEventUnlisten: UnlistenFn | null = null;

  async function initComponentEventListener() {
    if (componentEventUnlisten) return;
    try {
      componentEventUnlisten = await listen<PluginComponentEvent>("plugin:ui:component", (e) => {
        handlePluginComponentEvent(e.payload);
      });
      console.log("[PluginComponent] Event listener initialized");
    } catch (e) {
      console.error("[PluginComponent] Failed to initialize event listener:", e);
    }
  }

  function cleanupComponentEventListener() {
    if (componentEventUnlisten) {
      componentEventUnlisten();
      componentEventUnlisten = null;
    }
  }

  const pluginComponentSetMap = new Map<string, Array<{ componentId: string; prop: string }>>();

  function trackPluginComponentSet(pluginId: string, componentId: string, prop: string) {
    if (!pluginComponentSetMap.has(pluginId)) pluginComponentSetMap.set(pluginId, []);
    const list = pluginComponentSetMap.get(pluginId)!;
    if (!list.find((e) => e.componentId === componentId && e.prop === prop)) {
      list.push({ componentId, prop });
    }
  }

  function removePluginProxies(pluginId: string) {
    const reg = useComponentRegistry();
    for (const { id } of reg.list()) {
      reg.removeProxy(id, pluginId);
    }
    const sets = pluginComponentSetMap.get(pluginId) || [];
    for (const { componentId, prop } of sets) {
      for (const { handle } of reg.getAll(componentId)) {
        handle.set(prop, null);
      }
    }
    pluginComponentSetMap.delete(pluginId);
  }

  let i18nEventUnlisten: UnlistenFn | null = null;

  async function initI18nEventListener() {
    if (i18nEventUnlisten) return;
    try {
      i18nEventUnlisten = await listen<{
        plugin_id: string;
        action: string;
        locale: string;
        payload: string;
      }>("plugin-i18n-event", (e) => {
        const { plugin_id, action, locale, payload } = e.payload;
        if (action === "register_locale") {
          const { displayName } = JSON.parse(payload || "{}");
          registerPluginLocale(locale, displayName || locale);
        } else if (action === "add_translations") {
          const entries: Record<string, string> = JSON.parse(payload || "{}");
          addPluginTranslations(plugin_id, locale, entries);
        } else if (action === "remove_translations") {
          removePluginTranslations(plugin_id);
        }
      });
    } catch (e) {
      console.error("[PluginI18n] Failed to initialize event listener:", e);
    }
  }

  function cleanupI18nEventListener() {
    if (i18nEventUnlisten) {
      i18nEventUnlisten();
      i18nEventUnlisten = null;
    }
  }

  async function replayUiSnapshot() {
    try {
      const snapshot = await pluginApi.getPluginUiSnapshot();
      if (snapshot.length > 0) {
        console.log(`[PluginUI] Replaying ${snapshot.length} buffered UI events`);
        for (const event of snapshot) {
          handlePluginUiEvent({
            plugin_id: event.plugin_id,
            action: event.action as PluginUiAction,
            element_id: event.element_id,
            html: event.html,
          });
        }
      }
    } catch (e) {
      console.error("[PluginUI] Failed to replay UI snapshot:", e);
    }

    try {
      const sidebarSnapshot = await pluginApi.getPluginSidebarSnapshot();
      if (sidebarSnapshot.length > 0) {
        console.log(`[PluginSidebar] Replaying ${sidebarSnapshot.length} buffered sidebar events`);
        for (const event of sidebarSnapshot) {
          if (event.action === "register") {
            const sidebarMode: SidebarMode = "self";
            const filtered = sidebarItems.value.filter((item) => item.pluginId !== event.plugin_id);
            filtered.push({
              pluginId: event.plugin_id,
              label: event.label,
              icon: event.icon || undefined,
              mode: sidebarMode,
              showDependents: true,
              priority: 100,
            });
            filtered.sort((a, b) => a.priority - b.priority);
            sidebarItems.value = filtered;
          }
        }
      }
    } catch (e) {
      console.error("[PluginSidebar] Failed to replay sidebar snapshot:", e);
    }

    try {
      const componentSnapshot = await pluginApi.getPluginComponentSnapshot();
      if (componentSnapshot.length > 0) {
        console.log(
          `[PluginComponent] Replaying ${componentSnapshot.length} buffered component events`,
        );
        for (const event of componentSnapshot) {
          try {
            const payload = JSON.parse(event.payload_json);
            handlePluginComponentEvent({
              plugin_id: event.plugin_id,
              action: payload.action,
              component_id: payload.component_id,
              component_type: payload.component_type,
              props: payload.props,
              callback_id: payload.callback_id ?? "",
              prop: payload.prop,
              value: payload.value,
              method: payload.method,
              args: payload.args,
              priority: payload.priority,
              page_filter: payload.page_filter,
            });
          } catch (parseErr) {
            console.error("[PluginComponent] Failed to parse component event:", parseErr);
          }
        }
      }
    } catch (e) {
      console.error("[PluginComponent] Failed to replay component snapshot:", e);
    }

    try {
      const contextMenuSnapshot = await pluginApi.getPluginContextMenuSnapshot();
      if (contextMenuSnapshot.length > 0) {
        console.log(
          `[ContextMenu] Replaying ${contextMenuSnapshot.length} buffered context menu events`,
        );
        const { useContextMenuStore } = await import("@stores/contextMenuStore");
        const contextMenuStore = useContextMenuStore();
        for (const event of contextMenuSnapshot) {
          contextMenuStore.handleContextMenuEvent({
            action: event.action === "register" ? "register" : "unregister",
            plugin_id: event.plugin_id,
            context: event.context,
            items: event.items,
          });
        }
      }
    } catch (e) {
      console.error("[ContextMenu] Failed to replay context menu snapshot:", e);
    }
  }

  return {
    plugins,
    navItems,
    loading,
    error,
    icons,
    updates,
    pendingDependencies,
    sidebarItems,
    permissionLogs,
    pluginLogs,
    loadPlugins,
    refreshPlugins,
    togglePlugin,
    loadNavItems,
    collectSidebarItems,
    installFromZip,
    installBatch,
    loadPluginIcons,
    getPluginSettings,
    setPluginSettings,
    injectPluginCss,
    removePluginCss,
    injectAllPluginCss,
    playSound,
    deletePlugin,
    deletePlugins,
    checkUpdate,
    checkAllUpdates,
    applyThemeProviderSettings,
    applyThemeWidgetsProviderSettings,
    hasCapability,

    initUiEventListener,
    cleanupUiEventListener,
    removePluginUiElements,

    initSidebarEventListener,
    cleanupSidebarEventListener,

    getPermissionLogs,
    clearPermissionLogs,
    initPermissionLogListener,
    cleanupPermissionLogListener,

    initPluginLogListener,
    cleanupPluginLogListener,
    getPluginLogs,

    getHighRiskPermissions,

    initComponentEventListener,
    cleanupComponentEventListener,
    removePluginProxies,
    removePluginComponents,
    consumePendingComponentCreates,
    consumePendingComponentDeletes,

    initI18nEventListener,
    cleanupI18nEventListener,

    replayUiSnapshot,
  };
});
