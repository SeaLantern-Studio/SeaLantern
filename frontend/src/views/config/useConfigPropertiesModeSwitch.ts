import { ref, type Ref } from "vue";
import { configApi } from "@api/config";
import type { ConfigEntry as ConfigEntryType } from "@api/config";
import { i18n } from "@language";

function applyParsedSourceToVisualState(sourceText: string) {
  return configApi.parseServerPropertiesSource(sourceText);
}

interface ModeSwitchCompareContext {
  compareMode: Ref<boolean>;
  compareTargetSourceDraftText: Ref<string>;
  prepareCompareTargetSourceDraftForSourceMode: () => Promise<void>;
  applyCompareTargetSourceDraftToVisualState: (sourceText: string) => Promise<void>;
}

interface UseConfigPropertiesModeSwitchOptions {
  serverPath: Ref<string>;
  editorMode: Ref<"visual" | "source">;
  sourceDraftText: Ref<string>;
  editValues: Ref<Record<string, string>>;
  visualModeBaseValues: Ref<Record<string, string>>;
  sourceParseError: Ref<string | null>;
  setError: (message: string | null) => void;
  buildVisualPreviewSource: () => Promise<string>;
  getCompareContext: () => ModeSwitchCompareContext | null;
}

export function useConfigPropertiesModeSwitch(options: UseConfigPropertiesModeSwitchOptions) {
  const modeSwitching = ref(false);
  const visualDraftDirty = ref(false);

  function markVisualDraftDirty() {
    visualDraftDirty.value = true;
  }

  function clearSourceParseError() {
    options.sourceParseError.value = null;
  }

  async function handleEditorModeChange(mode: string | null) {
    const targetMode = mode === "source" ? "source" : "visual";
    if (
      targetMode === options.editorMode.value ||
      modeSwitching.value ||
      !options.serverPath.value
    ) {
      return null;
    }

    modeSwitching.value = true;
    options.setError(null);

    try {
      if (targetMode === "source") {
        if (visualDraftDirty.value) {
          options.sourceDraftText.value = await options.buildVisualPreviewSource();
          visualDraftDirty.value = false;
        }

        const compareContext = options.getCompareContext();
        if (compareContext?.compareMode.value) {
          await compareContext.prepareCompareTargetSourceDraftForSourceMode();
        }

        clearSourceParseError();
        options.editorMode.value = "source";
        return null;
      }

      const parsed = await applyParsedSourceToVisualState(options.sourceDraftText.value);
      const compareContext = options.getCompareContext();
      if (compareContext?.compareMode.value) {
        await compareContext.applyCompareTargetSourceDraftToVisualState(
          compareContext.compareTargetSourceDraftText.value,
        );
      }

      options.editValues.value = { ...parsed.raw };
      options.visualModeBaseValues.value = { ...parsed.raw };
      visualDraftDirty.value = false;
      clearSourceParseError();
      options.editorMode.value = "visual";
      return parsed.entries as ConfigEntryType[];
    } catch (e) {
      options.sourceParseError.value = i18n.t("config.source_parse_failed");
      options.setError(String(e));
      return null;
    } finally {
      modeSwitching.value = false;
    }
  }

  return {
    modeSwitching,
    visualDraftDirty,
    markVisualDraftDirty,
    clearSourceParseError,
    handleEditorModeChange,
  };
}
