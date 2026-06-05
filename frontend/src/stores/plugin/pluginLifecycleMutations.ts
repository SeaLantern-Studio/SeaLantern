import * as pluginApi from "@api/plugin";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { normalizeAppError } from "@utils/appError";
import type { BatchInstallResult, PluginInstallResult } from "@type/plugin";
import {
  clearPluginArtifacts,
  type CreatePluginLifecycleActionsOptions,
  type SetStoreError,
} from "./pluginLifecycleShared";

interface PluginLifecycleMutationsDependencies {
  loadPlugins: () => Promise<void>;
  setStoreError: SetStoreError;
}

export function createPluginLifecycleMutations(
  options: CreatePluginLifecycleActionsOptions,
  dependencies: PluginLifecycleMutationsDependencies,
) {
  async function installFromZip(zipPath: string): Promise<PluginInstallResult> {
    options.loading.value = true;
    try {
      const result = await pluginApi.installPlugin(zipPath);
      if (result.missing_dependencies.length > 0) {
        options.pendingDependencies.value = result.missing_dependencies;
      }
      await dependencies.loadPlugins();
      return result;
    } catch (errorCause) {
      pluginLogger.error("Store", "插件安装失败", normalizeAppError(errorCause));
      throw errorCause;
    } finally {
      options.loading.value = false;
    }
  }

  async function installBatch(paths: string[]): Promise<BatchInstallResult> {
    options.loading.value = true;
    try {
      const result = await pluginApi.installPluginsBatch(paths);
      if (result.failed.length > 0) {
        for (const item of result.failed) {
          pluginLogger.error("Store", `批量安装失败: ${item.path}`, item.error);
        }
      }
      await dependencies.loadPlugins();
      return result;
    } catch (errorCause) {
      pluginLogger.error("Store", "批量安装执行失败", normalizeAppError(errorCause));
      throw errorCause;
    } finally {
      options.loading.value = false;
    }
  }

  async function deletePlugin(pluginId: string, deleteData?: boolean) {
    try {
      await pluginApi.deletePlugin(pluginId, deleteData);
      clearPluginArtifacts(options, pluginId);
      await dependencies.loadPlugins();
    } catch (errorCause) {
      dependencies.setStoreError(`插件删除失败: ${pluginId}`, errorCause);
      throw errorCause;
    }
  }

  async function deletePlugins(pluginIds: string[], deleteData?: boolean) {
    try {
      await pluginApi.deletePlugins(pluginIds, deleteData);
      for (const pluginId of pluginIds) {
        clearPluginArtifacts(options, pluginId);
      }
      await dependencies.loadPlugins();
    } catch (errorCause) {
      dependencies.setStoreError("插件批量删除失败", errorCause);
      throw errorCause;
    }
  }

  async function checkUpdate(pluginId: string) {
    try {
      const update = await pluginApi.checkPluginUpdate(pluginId);
      if (update) {
        options.updates.value[pluginId] = update;
      }
      return update;
    } catch (errorCause) {
      pluginLogger.error("Store", `插件更新检查失败: ${pluginId}`, normalizeAppError(errorCause));
      return null;
    }
  }

  async function checkAllUpdates() {
    try {
      const allUpdates = await pluginApi.checkAllPluginUpdates();
      for (const update of allUpdates) {
        options.updates.value[update.plugin_id] = update;
      }
      return allUpdates;
    } catch (errorCause) {
      pluginLogger.error("Store", "插件批量更新检查失败", normalizeAppError(errorCause));
      return [];
    }
  }

  return {
    installFromZip,
    installBatch,
    deletePlugin,
    deletePlugins,
    checkUpdate,
    checkAllUpdates,
  };
}
