import { i18n } from "@language";
import type { MissingDependency, PluginInfo } from "@type/plugin";

export interface DependencyDetail {
  id: string;
  name: string;
  version?: string;
  status: "enabled" | "disabled" | "not-installed";
  statusLabel: string;
}

export interface DependentPlugin {
  id: string;
  name: string;
  required: boolean;
}

export interface PluginDependencyViewModel {
  permissions: string[];
  dependencies: DependencyDetail[];
  optionalDependencies: DependencyDetail[];
  dependents: DependentPlugin[];
}

type PluginSource = Readonly<PluginInfo[]> | (() => Readonly<PluginInfo[]>);

export function usePluginDependencies(plugins: PluginSource) {
  function getPlugins(): Readonly<PluginInfo[]> {
    return typeof plugins === "function" ? plugins() : plugins;
  }

  function findPlugin(pluginId: string) {
    return getPlugins().find((plugin) => plugin.manifest.id === pluginId);
  }

  function getDepDisplayName(depId: string): string {
    return findPlugin(depId)?.manifest.name || depId;
  }

  function hasMissingRequiredDependencies(plugin: PluginInfo): boolean {
    if (plugin.missing_dependencies) {
      const stillMissing = plugin.missing_dependencies.filter((dependency) => {
        if (!dependency.required) {
          return false;
        }
        const depPlugin = findPlugin(dependency.id);
        return !depPlugin || depPlugin.state !== "enabled";
      });
      if (stillMissing.length > 0) {
        return true;
      }
    }

    if (plugin.manifest.dependencies) {
      return plugin.manifest.dependencies.some((dependency) => {
        const depId = typeof dependency === "string" ? dependency : dependency.id;
        const depPlugin = findPlugin(depId);
        return !depPlugin || depPlugin.state !== "enabled";
      });
    }

    return false;
  }

  function getMissingRequiredDependencies(plugin: PluginInfo): MissingDependency[] {
    const missing: MissingDependency[] = [];

    if (plugin.missing_dependencies) {
      for (const dependency of plugin.missing_dependencies.filter((item) => item.required)) {
        const depPlugin = findPlugin(dependency.id);
        if (!depPlugin || depPlugin.state !== "enabled") {
          missing.push(dependency);
        }
      }
    }

    if (plugin.manifest.dependencies) {
      for (const dependency of plugin.manifest.dependencies) {
        const depId = typeof dependency === "string" ? dependency : dependency.id;
        if (missing.some((item) => item.id === depId)) {
          continue;
        }
        const depPlugin = findPlugin(depId);
        if (!depPlugin || depPlugin.state !== "enabled") {
          missing.push({ id: depId, required: true });
        }
      }
    }

    return missing;
  }

  function getMissingOptionalDependencies(plugin: PluginInfo): MissingDependency[] {
    const missing: MissingDependency[] = [];

    if (plugin.missing_dependencies) {
      for (const dependency of plugin.missing_dependencies.filter((item) => !item.required)) {
        const depPlugin = findPlugin(dependency.id);
        if (!depPlugin || depPlugin.state !== "enabled") {
          missing.push(dependency);
        }
      }
    }

    if (plugin.manifest.optional_dependencies) {
      for (const dependency of plugin.manifest.optional_dependencies) {
        const depId = typeof dependency === "string" ? dependency : dependency.id;
        if (missing.some((item) => item.id === depId)) {
          continue;
        }
        const depPlugin = findPlugin(depId);
        if (!depPlugin || depPlugin.state !== "enabled") {
          missing.push({ id: depId, required: false });
        }
      }
    }

    return missing;
  }

  function hasMissingOptionalDependencies(plugin: PluginInfo): boolean {
    return getMissingOptionalDependencies(plugin).length > 0;
  }

  function getDependencyTooltip(plugin: PluginInfo): string {
    const requiredMissing = getMissingRequiredDependencies(plugin);
    const optionalMissing = getMissingOptionalDependencies(plugin);
    const parts: string[] = [];

    if (requiredMissing.length > 0) {
      const names = requiredMissing
        .map((dependency) => getDepDisplayName(dependency.id))
        .join(", ");
      parts.push(i18n.t("plugins.dep_tooltip.required", { names }));
    }

    if (optionalMissing.length > 0) {
      const names = optionalMissing
        .map((dependency) => getDepDisplayName(dependency.id))
        .join(", ");
      parts.push(i18n.t("plugins.dep_tooltip.optional", { names }));
    }

    return parts.join("\n");
  }

  function mapDependencyDetails(
    dependencies:
      | PluginInfo["manifest"]["dependencies"]
      | PluginInfo["manifest"]["optional_dependencies"],
  ): DependencyDetail[] {
    if (!dependencies || dependencies.length === 0) {
      return [];
    }

    return dependencies.map((dependency) => {
      const depId = typeof dependency === "string" ? dependency : dependency.id;
      const version = typeof dependency === "object" ? dependency.version : undefined;
      const depPlugin = findPlugin(depId);

      let status: DependencyDetail["status"];
      let statusLabel: string;

      if (!depPlugin) {
        status = "not-installed";
        statusLabel = i18n.t("plugins.dep_status.not_installed");
      } else if (depPlugin.state !== "enabled") {
        status = "disabled";
        statusLabel = i18n.t("plugins.dep_status.installed_not_enabled");
      } else {
        status = "enabled";
        statusLabel = i18n.t("plugins.dep_status.enabled");
      }

      return {
        id: depId,
        name: depPlugin?.manifest.name || depId,
        version,
        status,
        statusLabel,
      };
    });
  }

  function getDependencyDetails(plugin: PluginInfo): DependencyDetail[] {
    return mapDependencyDetails(plugin.manifest.dependencies);
  }

  function getOptionalDependencyDetails(plugin: PluginInfo): DependencyDetail[] {
    return mapDependencyDetails(plugin.manifest.optional_dependencies);
  }

  function getDependentPlugins(plugin: PluginInfo): DependentPlugin[] {
    const dependents: DependentPlugin[] = [];
    const pluginId = plugin.manifest.id;

    for (const candidate of getPlugins()) {
      if (candidate.manifest.id === pluginId) {
        continue;
      }

      if (candidate.manifest.dependencies) {
        for (const dependency of candidate.manifest.dependencies) {
          const depId = typeof dependency === "string" ? dependency : dependency.id;
          if (depId === pluginId) {
            dependents.push({
              id: candidate.manifest.id,
              name: candidate.manifest.name,
              required: true,
            });
            break;
          }
        }
      }

      if (
        !dependents.find((item) => item.id === candidate.manifest.id) &&
        candidate.manifest.optional_dependencies
      ) {
        for (const dependency of candidate.manifest.optional_dependencies) {
          const depId = typeof dependency === "string" ? dependency : dependency.id;
          if (depId === pluginId) {
            dependents.push({
              id: candidate.manifest.id,
              name: candidate.manifest.name,
              required: false,
            });
            break;
          }
        }
      }
    }

    return dependents;
  }

  function getPluginDependencyViewModel(plugin: PluginInfo): PluginDependencyViewModel {
    return {
      permissions: plugin.manifest.permissions || [],
      dependencies: getDependencyDetails(plugin),
      optionalDependencies: getOptionalDependencyDetails(plugin),
      dependents: getDependentPlugins(plugin),
    };
  }

  return {
    getDepDisplayName,
    hasMissingRequiredDependencies,
    getMissingRequiredDependencies,
    getMissingOptionalDependencies,
    hasMissingOptionalDependencies,
    getDependencyTooltip,
    getDependencyDetails,
    getOptionalDependencyDetails,
    getDependentPlugins,
    getPluginDependencyViewModel,
  };
}
