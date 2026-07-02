import type {
  PluginDependency,
  PluginInfo,
  PluginSettingField,
  PluginSettingOption,
} from "@type/plugin";

export type PluginSettingValue = string | number | boolean | null;
export type PluginSettingsRecord = Record<string, PluginSettingValue>;

export function getPluginDependencyId(dependency: PluginDependency): string {
  return typeof dependency === "string" ? dependency : dependency.id;
}

export function clearSettingsRecord(record: Record<string, unknown>) {
  Object.keys(record).forEach((key) => delete record[key]);
}

export function serializeSettingsRecord(settings: Record<string, unknown>): string {
  return JSON.stringify(settings);
}

export function getDefaultPluginSettingValue(type: PluginSettingField["type"]): PluginSettingValue {
  switch (type) {
    case "number":
      return 0;
    case "boolean":
    case "checkbox":
      return false;
    default:
      return "";
  }
}

export function buildPluginSettingsForm(
  fields: PluginSettingField[] | undefined,
  savedSettings: Record<string, unknown> | undefined | null,
): PluginSettingsRecord {
  const form: PluginSettingsRecord = {
    ...((savedSettings ?? {}) as PluginSettingsRecord),
  };

  for (const field of fields || []) {
    if (form[field.key] === undefined) {
      form[field.key] = field.default ?? getDefaultPluginSettingValue(field.type);
    }
  }

  return form;
}

export function resetPluginSettingsForm(
  form: PluginSettingsRecord,
  fields: PluginSettingField[] | undefined,
) {
  for (const field of fields || []) {
    form[field.key] = field.default ?? getDefaultPluginSettingValue(field.type);
  }
}

export function getPluginFieldStringValue(value: unknown): string {
  return value == null ? "" : String(value);
}

export function getPluginFieldSelectValue(value: unknown): string | number | undefined {
  if (typeof value === "string" || typeof value === "number") {
    return value;
  }

  return undefined;
}

export function getPluginFieldOptions(options: PluginSettingOption[] | undefined) {
  return options ?? [];
}

export function updatePluginSettingsField(
  form: PluginSettingsRecord,
  key: string,
  value: string | number | boolean,
) {
  form[key] = value;
}

export function applyPluginPreset(
  form: PluginSettingsRecord,
  presetKey: string,
  presetData: Record<string, string>,
) {
  const payload: Record<string, unknown> = {};

  for (const [key, value] of Object.entries(presetData)) {
    if (key === "name") {
      continue;
    }

    form[key] = value;
    payload[key] = value;
  }

  form.preset = presetKey;
  payload.preset = presetKey;

  return payload;
}

export function findDependentPlugins(
  plugins: PluginInfo[],
  pluginId: string,
  includeOptionalDependencies = true,
) {
  return plugins.filter((candidate) => {
    if (candidate.state !== "enabled") {
      return false;
    }

    if (candidate.manifest.id === pluginId) {
      return false;
    }

    const dependencies = includeOptionalDependencies
      ? [
          ...(candidate.manifest.dependencies || []),
          ...(candidate.manifest.optional_dependencies || []),
        ]
      : candidate.manifest.dependencies || [];

    return dependencies.some((dependency) => getPluginDependencyId(dependency) === pluginId);
  });
}
