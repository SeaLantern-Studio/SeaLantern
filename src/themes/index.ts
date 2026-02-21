import type { ThemeDefinition, ThemeRegistry, ThemeColors, ColorPlan } from "./types";

let themes: ThemeRegistry = {};

let initialized = false;

async function initializeThemes(): Promise<void> {
  if (initialized) return;

  const themeModules = import.meta.glob("./*.ts", { eager: true });

  for (const [path, module] of Object.entries(themeModules)) {
    const fileName = path.replace("./", "").replace(".ts", "");
    if (fileName === "types" || fileName === "index") {
      continue;
    }

    const mod = module as Record<string, unknown>;

    let theme: ThemeDefinition | null = null;

    if (mod.default && isThemeDefinition(mod.default)) {
      theme = mod.default as ThemeDefinition;
    } else {
      for (const key of Object.keys(mod)) {
        if (key.endsWith("Theme") && isThemeDefinition(mod[key])) {
          theme = mod[key] as ThemeDefinition;
          break;
        }
      }
    }

    if (theme) {
      themes[theme.id] = theme;
    }
  }

  initialized = true;
}

function isThemeDefinition(obj: unknown): obj is ThemeDefinition {
  if (!obj || typeof obj !== "object") return false;
  const t = obj as Record<string, unknown>;
  return (
    typeof t.id === "string" &&
    typeof t.name === "string" &&
    typeof t.light === "object" &&
    typeof t.dark === "object" &&
    typeof t.lightAcrylic === "object" &&
    typeof t.darkAcrylic === "object"
  );
}

function ensureInitialized(): void {
  if (!initialized) {
    initializeThemes();
  }
}

export function getAllThemes(): ThemeRegistry {
  ensureInitialized();
  return { ...themes };
}

export function getThemeOptions(): Array<{ label: string; value: string }> {
  ensureInitialized();
  return Object.values(themes).map((theme) => ({
    label: theme.name,
    value: theme.id,
  }));
}

export function getThemeById(id: string): ThemeDefinition | undefined {
  ensureInitialized();
  return themes[id];
}

export function registerTheme(theme: ThemeDefinition): void {
  ensureInitialized();
  if (themes[theme.id]) {
    console.warn(`Theme "${theme.id}" already exists, it will be overwritten.`);
  }
  themes[theme.id] = theme;
}

export function unregisterTheme(id: string): boolean {
  ensureInitialized();
  if (themes[id]) {
    delete themes[id];
    return true;
  }
  return false;
}

export async function resetThemes(): Promise<void> {
  themes = {};
  initialized = false;
  await initializeThemes();
}

export function getThemeColors(themeId: string, plan: ColorPlan): ThemeColors | undefined {
  ensureInitialized();
  const theme = themes[themeId];
  if (!theme) return undefined;
  return theme[plan];
}

export function getThemeColorValue(
  themeId: string,
  plan: ColorPlan,
  colorType: keyof ThemeColors,
): string | undefined {
  const colors = getThemeColors(themeId, plan);
  return colors?.[colorType];
}

export function mapLegacyPlanName(plan: string): ColorPlan {
  const mapping: Record<string, ColorPlan> = {
    light: "light",
    dark: "dark",
    light_acrylic: "lightAcrylic",
    dark_acrylic: "darkAcrylic",
    lightAcrylic: "lightAcrylic",
    darkAcrylic: "darkAcrylic",
  };
  return mapping[plan] || "light";
}

export type { ThemeDefinition, ThemeColors, ColorPlan, ThemeRegistry };
